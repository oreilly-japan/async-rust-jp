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
    let (tx, rx) = channel::<RespMessage>(1000);

    let _resp_actor_handle = tokio::spawn(async {
        resp_actor(rx).await;
    });
    for i in 0..100000000 {
        let (resp_tx, resp_rx) = oneshot::channel::<i64>();
        let msg = RespMessage {
            value: i,
            responder: resp_tx
        };
        tx.send(msg).await.unwrap();
        println!("Response: {}", resp_rx.await.unwrap());
    }
}