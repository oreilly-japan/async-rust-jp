use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll as MioPoll, Token};
use std::io::{Read, Write};
use std::time::Duration;
use std::error::Error;

use std::sync::LazyLock;
use std::pin::Pin;
use std::{future::Future, panic::catch_unwind, thread};
use std::task::{Context, Poll};
use async_task::{Runnable, Task};
use flume::{Receiver, Sender};
use futures_lite::future;



#[derive(Debug, Clone, Copy)]
enum FutureType {
    High,
    Low,
}

// Multiple Threads , Multiple Queues, with Task Stealing, refactored, with join macro, background task

fn spawn_task<F, T>(future: F, order: FutureType) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    static HIGH_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> =
        LazyLock::new(|| flume::unbounded::<Runnable>());
    static LOW_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> =
        LazyLock::new(|| flume::unbounded::<Runnable>());

    static HIGH_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
        let high_num = std::env::var("HIGH_NUM").unwrap().parse::<usize>()
                                                             .unwrap();
        for _ in 0..high_num {
            let high_receiver = HIGH_CHANNEL.1.clone();
            let low_receiver = LOW_CHANNEL.1.clone();
            thread::spawn(move || {
                loop {
                    match high_receiver.try_recv() {
                        Ok(runnable) => {
                            let _ = catch_unwind(|| runnable.run());
                        }
                        Err(_) => match low_receiver.try_recv() {
                            Ok(runnable) => {
                                let _ = catch_unwind(|| runnable.run());
                            }
                            Err(_) => {
                                thread::sleep(Duration::from_millis(100));
                            }
                        },
                    };
                }
            });
        }
        HIGH_CHANNEL.0.clone()
    });
    static LOW_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
        let low_num = std::env::var("LOW_NUM").unwrap().parse::<usize>()
                                                             .unwrap();
        for _ in 0..low_num {
            let high_receiver = HIGH_CHANNEL.1.clone();
            let low_receiver = LOW_CHANNEL.1.clone();
            thread::spawn(move || {
                loop {
                    match low_receiver.try_recv() {
                        Ok(runnable) => {
                            let _ = catch_unwind(|| runnable.run());
                        }
                        Err(_) => match high_receiver.try_recv() {
                            Ok(runnable) => {
                                let _ = catch_unwind(|| runnable.run());
                            }
                            Err(_) => {
                                thread::sleep(Duration::from_millis(100));
                            }
                        },
                    };
                }
            });
        }
        LOW_CHANNEL.0.clone()
    });

    let schedule_high = |runnable| HIGH_QUEUE.send(runnable).unwrap();
    let schedule_low = |runnable| LOW_QUEUE.send(runnable).unwrap();

    let schedule = match order {
        FutureType::High => schedule_high,
        FutureType::Low => schedule_low,
    };
    let (runnable, task) = async_task::spawn(future, schedule);
    runnable.schedule();
    return task;
}


macro_rules! spawn_task {
    ($future:expr) => {
        spawn_task!($future, FutureType::Low)
    };
    ($future:expr, $order:expr) => {
        spawn_task($future, $order)
    };
}

macro_rules! join {
    ($($future:expr),*) => {
        {
            let mut results = Vec::new();
            $(
                results.push(future::block_on($future));
            )*
            results
        }
    };
}



struct Runtime {
    high_num: usize,
    low_num: usize,
}


impl Runtime {
    pub fn new() -> Self {
        let num_cores = std::thread::available_parallelism().unwrap()
                                                            .get();
        Self {
            high_num: num_cores - 2,
            low_num: 1,
        }
    }
    pub fn with_high_num(mut self, num: usize) -> Self {
        self.high_num = num;
        self
    }
    pub fn with_low_num(mut self, num: usize) -> Self {
        self.low_num = num;
        self
    }
    pub fn run(&self) {
        unsafe {
            std::env::set_var("HIGH_NUM", self.high_num.to_string());
            std::env::set_var("LOW_NUM", self.low_num.to_string());
        }
        let high = spawn_task!(async {}, FutureType::High);
        let low = spawn_task!(async {}, FutureType::Low);
        join!(high, low);
    
    }
}

/////////////////////////////////


const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);


struct ServerFuture {
    server: TcpListener,
    poll: MioPoll,
}
impl Future for ServerFuture {

    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
            let mut events = Events::with_capacity(1);
            let _ = self.poll.poll(
                &mut events,
                Some(Duration::from_millis(200))
            ).unwrap();
            
            
            for event in events.iter() {
                if event.token() == SERVER && event.is_readable() {
                    let (mut stream, _) = self.server.accept().unwrap();
                    let mut buffer = [0u8; 1024];
                    let mut received_data = Vec::new();
                    
                    loop {
                        match stream.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                received_data.extend_from_slice(&buffer[..n]);
                            }
                            Ok(_) => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error reading from stream: {}", e);
                                break;
                            }
                        }
                    }
                    if !received_data.is_empty() {
                        let received_str = String::from_utf8_lossy(&received_data);
                        return Poll::Ready(received_str.to_string())
                    }
                    cx.waker().wake_by_ref();
                    return Poll::Pending
                }

            }
            cx.waker().wake_by_ref();
            return Poll::Pending
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Runtime::new().with_low_num(2).with_high_num(4).run();
    let addr = "127.0.0.1:13265".parse()?;
    let mut server = TcpListener::bind(addr)?;
    let mut stream = TcpStream::connect(server.local_addr()?)?;
    let poll: MioPoll = MioPoll::new()?;
    poll.registry().register(&mut server, SERVER, Interest::READABLE)?;
    
    let server_worker = ServerFuture{
        server,
        poll,
    };
    let test = spawn_task!(server_worker);

    let mut client_poll: MioPoll = MioPoll::new()?;
    client_poll.registry().register(&mut stream, CLIENT, Interest::WRITABLE)?;

    let mut events = Events::with_capacity(128);
    let _ = client_poll.poll(
        &mut events,
        None
    ).unwrap();

    for event in events.iter() {
        if event.token() == CLIENT && event.is_writable() {
            let message = "that's so dingo!\n";
            let _ = stream.write_all(message.as_bytes());
        }
    }
    
    let outcome = future::block_on(test);
    println!("outcome: {}", outcome);
    

    Ok(())
}
