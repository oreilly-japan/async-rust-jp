use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

trait Logging {
    fn log(&self);
}

struct LoggingFuture<F: Future + Logging> {
    inner: F,
}

impl<F: Future + Logging> Future for LoggingFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.log();
        inner.poll(cx)
    }
}

impl<F: Future> Logging for F {
    fn log(&self) {
        println!("Polling the future!");
    }
}

async fn my_async_function() -> String {
    "Result of async computation".to_string()
}

#[tokio::main]
async fn main() {
    let logged_future = LoggingFuture { inner: my_async_function() };
    let result = logged_future.await;
    println!("{}", result);
}
