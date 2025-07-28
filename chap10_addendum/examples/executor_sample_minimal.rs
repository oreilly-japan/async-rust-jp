use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    task::{Context, Poll, Waker},
    time::Duration,
};
use waker_fn::waker_fn;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    task_id: usize,
    waker: Arc<Waker>,
}
pub struct Executor {
    sender: mpsc::Sender<usize>,
    receiver: mpsc::Receiver<usize>,
    count: AtomicUsize,
    pub waiting: Mutex<HashMap<usize, Task>>,
    pub polling: Mutex<VecDeque<Task>>,
}

impl Executor {
    pub fn new() -> Arc<Self> {
        let (sender, receiver) = mpsc::channel();
        Arc::new(Executor {
            sender: sender,                       // ウェイカへ渡す送信端
            receiver: receiver,                   // ウェイカからの通信を受け取る受信端
            count: AtomicUsize::new(0),           // タスクにユニークな番号を与えるためのカウンタ
            waiting: Mutex::new(HashMap::new()),  // ウェイカを待機するタスクのマップ
            polling: Mutex::new(VecDeque::new()), // ポーリングしてよいタスクのキュー
        })
    }

    pub fn spawn_rcv<F, T>(&self, future: F) -> mpsc::Receiver<T>
    where
        F: Future<Output = T> + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async move {
            let result = future.await;
            let _ = tx.send(result);
        });

        let task_id = self.count.fetch_add(1, Ordering::Relaxed); // task にユニークな番号を付番
        let sender = self.sender.clone();
        let task = Task {
            future,
            task_id,               // waker_fn を用いてウェイカを作成
            waker: waker_fn(move || sender.send(task_id).expect("send")).into(),
        };
        self.polling.lock().unwrap().push_back(task);
        return rx;
    }

    pub fn run(&self) {
        loop {
            self.poll(); // polling キューにあるタスクをすべて実行

            // 実行すべきタスクも待機すべきウェイカもなければ終了
            if self.polling.lock().unwrap().is_empty() && self.waiting.lock().unwrap().is_empty() {
                break;
            }
            let task_id = self.receiver.recv().unwrap(); // ウェイカを待機

            // ウェイカから受け取ったIDに対応するタスクを取り出してpollingに追加
            if let Some(task) = self.waiting.lock().unwrap().remove(&task_id) {
                self.polling.lock().unwrap().push_back(task);
            }

        }
    }

    pub fn poll(&self) {
        loop {
            // ポーリング対象のタスクを取得
            let mut task = match self.polling.lock().unwrap().pop_front() {
                Some(task) => task,
                None => break,   // タスクがなければ終了
            };
            let waker: Arc<Waker> = task.waker.clone();
            let context = &mut Context::from_waker(&waker);
            match task.future.as_mut().poll(context) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    // Pendig が返された場合には、タスクをwaitingに登録
                    self.waiting.lock().unwrap().insert(task.task_id, task);
                }
            }
        }
    }
}


fn main() {
    let executor = Executor::new();
    let rx = executor.spawn_rcv(async {
        async_std::task::sleep(Duration::from_secs(1)).await;
        10
    });
    executor.run();
    println!("got value {:}", rx.recv().unwrap());
}
