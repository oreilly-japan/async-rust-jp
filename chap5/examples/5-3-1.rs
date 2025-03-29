#![feature(coroutines, coroutine_trait)]
use std::{
    collections::VecDeque,
    future::Future,
    ops::{Coroutine, CoroutineState},
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

struct SleepCoroutine {
    pub start: Instant,
    pub duration: std::time::Duration,
}
impl SleepCoroutine {
    fn new(duration: std::time::Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
}
impl Coroutine<()> for SleepCoroutine {
    type Yield = ();
    type Return = ();

    fn resume(self: Pin<&mut Self>, _: ()) 
    -> CoroutineState<Self::Yield, Self::Return> {
        if self.start.elapsed() >= self.duration {
            CoroutineState::Complete(())
        } else {
            CoroutineState::Yielded(())
        }
    }
}

fn main() {
    let mut sleep_coroutines = VecDeque::new();
    sleep_coroutines.push_back(
        SleepCoroutine::new(std::time::Duration::from_secs(1))
    );
    sleep_coroutines.push_back(
        SleepCoroutine::new(std::time::Duration::from_secs(1))
    );
    sleep_coroutines.push_back(
        SleepCoroutine::new(std::time::Duration::from_secs(1))
    );

    let mut counter = 0;
    let start = Instant::now();

    while counter < sleep_coroutines.len() {
        let mut coroutine = sleep_coroutines.pop_front().unwrap();
        match Pin::new(&mut coroutine).resume(()) {
            CoroutineState::Yielded(_) => {
                sleep_coroutines.push_back(coroutine);
            },
            CoroutineState::Complete(_) => {
                counter += 1;
            },
        }
    }
    println!("Took {:?}", start.elapsed());
}

