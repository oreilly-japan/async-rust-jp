use tokio::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender},
    oneshot
};

struct RespMessage {
    value: i32,
    responder: oneshot::Sender<i64>
}

async fn resp_actor(mut rx: Receiver<RespMessage>) {
    let mut state= 0;

    while let Some(msg) = rx.recv().await {
        state += msg.value;
        if msg.responder.send(state.into()).is_err() {
            eprintln!("Failed to send response");
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (tx, rx) = channel::<RespMessage>(100000000);
    let _resp_actor_handle = tokio::spawn(async {
        resp_actor(rx).await;
    });
    let mut handles = Vec::new();
    
    let now = tokio::time::Instant::now();
//    for i in 0..100000000 {
    for i in 0..10000000 {
        let tx_ref = tx.clone();
    
        let future = async move {
            let (resp_tx, resp_rx) = oneshot::channel::<i64>();
            let msg = RespMessage {
                value: i,
                responder: resp_tx
            };
            tx_ref.send(msg).await.unwrap();
            let _ = resp_rx.await.unwrap();
        };
        handles.push(tokio::spawn(future));
    }
    for handle in handles {
        let _ = handle.await.unwrap();
    }
    println!("Elapsed: {:?}", now.elapsed());
    
}