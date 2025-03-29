use tokio::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender},
    oneshot
};

struct Message {
    value: i64
}

async fn basic_actor(mut rx: Receiver<Message>) {
    let mut state = 0;

    while let Some(msg) = rx.recv().await {
        state += msg.value;
        println!("Received: {}", msg.value);
        println!("State: {}", state);
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = channel::<Message>(100);

    let _actor_handle = tokio::spawn(
        basic_actor(rx)
    );
    for i in 0..10 {
        let msg = Message { value: i };
        tx.send(msg).await.unwrap();
    }
}