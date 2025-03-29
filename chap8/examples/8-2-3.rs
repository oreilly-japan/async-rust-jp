use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::error::TryRecvError;

async fn actor_replacement(state: Arc<Mutex<i64>>, value: i64) -> i64 {
    let mut state = state.lock().await;
    *state += value;
    return *state;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    let now = tokio::time::Instant::now();

    for i in 0..10000000 {
        let state_ref = state.clone();
        let future = async move {
            actor_replacement(state_ref, i).await;
        };
        handles.push(tokio::spawn(future));
    }
    for handle in handles {
        let _ = handle.await.unwrap();
    }
    println!("Elapsed: {:?}", now.elapsed());
}
