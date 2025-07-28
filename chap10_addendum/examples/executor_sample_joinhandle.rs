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

pub struct JoinHandle<T>
where
    T: 'static,
{
    receiver: Option<mpsc::Receiver<T>>,
    cache: Arc<Mutex<Option<T>>>,
}

impl<T> Future for JoinHandle<T>
where
    T: 'static + Send,
{
    type Output = Result<T, mpsc::RecvError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        {   // すでにキャッシュされていたらその値を返す
            let mut guard = self.cache.lock().unwrap();
            if guard.is_some() {
                let value = guard.take().unwrap();
                return Poll::Ready(Ok(value));
            }
        }
        if self.receiver.is_none() {  // すでにreceiverをスレッドに渡しているのでそのまま待機
            return Poll::Pending;
        }

        match self.receiver.as_ref().unwrap().try_recv() { // 受信を試みる
            Ok(value) => Poll::Ready(Ok(value)),   // 受信できたらそのまま値を返す
            Err(mpsc::TryRecvError::Empty) => {       // 受信できなければ受信スレッドを起動
                let waker = cx.waker().clone();
                let receiver = self.receiver.take().unwrap();
                let cache = self.cache.clone();
                std::thread::spawn(move || {
                    let value = receiver.recv().unwrap();
                    cache.lock().unwrap().replace(value);  // cacheに値をセット
                    waker.wake();             // ウェイカを呼び出す
                });
                Poll::Pending
            }
            _ => Poll::Ready(Err(mpsc::RecvError)),
        }
    }
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

    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: Send + 'static,
    {
        let rx = self.spawn_rcv(future);
        let handle = JoinHandle {
            receiver: Some(rx),
            cache: Arc::new(None.into()),
        };
        handle
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


fn _main() {
    let executor = Executor::new();
    let rx = executor.spawn_rcv(async {
        async_std::task::sleep(Duration::from_secs(1)).await;
        10
    });
    executor.run();
    println!("got value {:}", rx.recv().unwrap());
}

use async_std::task::sleep;
async fn f() -> i64 {
    println!("in f sleeping 1");
    sleep(Duration::from_secs(1)).await;
    println!("in f calling g");
    let v = g().await;
    println!("in f sleeping 2");
    sleep(Duration::from_secs(1)).await;
    println!("in f return v");
    v
}

async fn g() -> i64 {
    println!("in g sleeping");
    sleep(Duration::from_secs(1)).await;
    println!("in g return 10");
    10
}

fn test1() {
    let executor = Executor::new();
    let executor_clone = executor.clone();
    executor.clone().spawn(async move {
        let handle_f = executor.spawn(f());
        let handle_g = executor.spawn(g());
        println!(
            "got value {:}",
            handle_f.await.unwrap() + handle_g.await.unwrap()
        );
    }); 
    executor_clone.run();
}

fn test2() {
    let executor = Executor::new();
    let executor_clone = executor.clone();
    executor.clone().spawn(async move {
        let handle_f = executor.spawn(async {10});
        let handle_g = executor.spawn(async {20});
        println!(
            "got value {:}",
            handle_f.await.unwrap() + handle_g.await.unwrap()
        );
    }); 
    executor_clone.run();
}

fn main() {
    test1();
    test2();
}