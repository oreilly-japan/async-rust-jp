use std::{future::Future, panic::catch_unwind, thread};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::sync::LazyLock;

use async_task::{Runnable, Task};
use futures_lite::future;

fn spawn_task<F, T>(future: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    static QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
        let (tx, rx) = flume::unbounded::<Runnable>();
    
        thread::spawn(move || {
            while let Ok(runnable) = rx.recv() {
                println!("runnable accepted");
                let _ = catch_unwind(|| runnable.run());
            }
        });
        tx
    }); 

    let schedule = |runnable| QUEUE.send(runnable).unwrap();
    let (runnable, task) = async_task::spawn(future, schedule);

    runnable.schedule();
    println!("Here is the queue count: {:?}", QUEUE.len());
    return task
}

use std::time::Instant;

struct AsyncSleep {
    start_time: Instant,
    duration: Duration,
}
impl AsyncSleep {
    fn new(duration: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
        }
    }
}

impl Future for AsyncSleep {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)  
    -> Poll<Self::Output> {
        let elapsed_time = self.start_time.elapsed();
        if elapsed_time >= self.duration {
            Poll::Ready(true)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}


fn main() {
    let async_sleep = AsyncSleep::new(Duration::from_secs(5));
    let async_sleep_handle = spawn_task(async {
        async_sleep.await;
        println!("after sleep")
    });
    future::block_on(async_sleep_handle);
}

