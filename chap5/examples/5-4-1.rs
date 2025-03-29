#![feature(coroutines, coroutine_trait)]
use rand::Rng;
use std::{
    ops::{Coroutine, CoroutineState},
    pin::Pin,
};

struct RandCoRoutine {
    pub value: u8,
    pub live: bool,
}
impl RandCoRoutine {
    fn new() -> Self {
        let mut coroutine = Self {
            value: 0,
            live: true,
        };
        coroutine.generate();
        coroutine
    }
    fn generate(&mut self) {
        let mut rng = rand::thread_rng();
        self.value = rng.gen_range(0..=10);
    }
}

impl Coroutine<()> for RandCoRoutine {
    type Yield = u8;
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, _: ()) -> CoroutineState<Self::Yield, Self::Return> {
        self.generate();
        CoroutineState::Yielded(self.value)
    }
}

fn main() {
    let mut coroutines = Vec::new();
    for _ in 0..10 {
        coroutines.push(RandCoRoutine::new());
    }
    let mut total: u32 = 0;

    loop {
        let mut all_dead = true;
        for mut coroutine in coroutines.iter_mut() {
            if coroutine.live {
                all_dead = false;
                match Pin::new(&mut coroutine).resume(()) {
                    CoroutineState::Yielded(result) => {
                        total += result as u32;
                    }
                    CoroutineState::Complete(_) => {
                        panic!("Coroutine should not complete");
                    }
                }
                if coroutine.value < 9 {
                    coroutine.live = false;
                }
            }
        }
        if all_dead {
            break;
        }
    }
    println!("Total: {}", total);
}

