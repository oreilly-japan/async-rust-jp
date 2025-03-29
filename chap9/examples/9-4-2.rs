use std::task::{Poll,Context};
use std::pin::Pin;
use std::future::Future;

#[derive(Clone)]
enum State {
    On,
    Off,
}

enum Event {
    SwitchOn,
    SwitchOff,
}

impl State {
    async fn transition(self, event: Event) -> Self {
        match (&self, event) {
            (State::On, Event::SwitchOff) => {
                println!("Transitioning to the Off state");
                State::Off
            },
            (State::Off, Event::SwitchOn) => {
                println!("Transitioning to the On state");
                State::On
            },
            _ => {
                println!(
                    "No transition possible, 
                    staying in the current state"
                );
                self
            },
        }
    }
}

struct StateFuture<F: Future, X: Future> {
    pub state: State,
    pub on_future: F,
    pub off_future: X,
}


impl<F: Future, X: Future> StateFuture<F, X> {
    async fn off(&mut self) {
        self.state = self.state.clone().transition(Event::SwitchOff).await;
    }
    async fn on(&mut self) {
        self.state = self.state.clone().transition(Event::SwitchOn).await;
    }
}

impl<F: Future, X: Future> Future for StateFuture<F, X> {
    type Output = State;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
        match self.state {
            State::On => {
                let inner = unsafe { 
                    self.map_unchecked_mut(|s| &mut s.on_future) 
                };
                let _ = inner.poll(cx);
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            State::Off => {
                let inner = unsafe { 
                    self.map_unchecked_mut(|s| &mut s.off_future)
                };
                let _ = inner.poll(cx);
                cx.waker().wake_by_ref();
                Poll::Pending
            },
        }
    }
}

async fn print_a() {
    loop {
        tokio::task::yield_now().await;
        println!("a");
    }
}

async fn print_b() {
    loop {
        tokio::task::yield_now().await;
        println!("b");
    }
}

#[tokio::main]
async fn main() {
    let mut state_future = StateFuture{
        state: State::On, 
        on_future: print_a(), 
        off_future: print_b(),
    };
    state_future.off().await;
    //state_future.on().await;
    state_future.await;
}
