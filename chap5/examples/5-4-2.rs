#![feature(coroutines, coroutine_trait)]
use rand::Rng;
use std::{
    ops::{Coroutine, CoroutineState},
    pin::Pin,
    time::Duration,
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
    let (sender, receiver) = std::sync::mpsc::channel::<RandCoRoutine>();
    let _thread = std::thread::spawn(move || {
        loop {
            let mut coroutine = match receiver.recv() {
                Ok(coroutine) => coroutine,
                Err(_) => break,
            };
            match Pin::new(&mut coroutine).resume(()) {
                CoroutineState::Yielded(result) => {
                    println!("Coroutine yielded: {}", result);
                },
                CoroutineState::Complete(_) => {
                    panic!("Coroutine should not complete");
                },
            }
        }
    });
    std::thread::sleep(Duration::from_secs(1));
    sender.send(RandCoRoutine::new()).unwrap();
    sender.send(RandCoRoutine::new()).unwrap();
    std::thread::sleep(Duration::from_secs(1));
}