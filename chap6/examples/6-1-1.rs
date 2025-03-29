use std::sync::Arc;
use std::sync::atomic::{AtomicI16, AtomicBool};
use core::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::future::Future;
use std::task::Poll;
use std::pin::Pin;
use std::task::Context;
use std::time::{Instant, Duration};

static TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| {
    Arc::new(AtomicI16::new(2090)) //<1>
});
static DESIRED_TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| {
    Arc::new(AtomicI16::new(2100)) //<2>
});
static HEAT_ON: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| {
    Arc::new(AtomicBool::new(false)) //<3>
});

pub struct DisplayFuture {
    pub temp_snapshot: i16,
}

impl DisplayFuture {
    pub fn new() -> Self {
        DisplayFuture {
            temp_snapshot: TEMP.load(Ordering::SeqCst)
        }
    }
}

impl Future for DisplayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output> {
            let current_snapshot = TEMP.load(Ordering::SeqCst); // <1>
            let desired_temp = DESIRED_TEMP.load(Ordering::SeqCst);
            let heat_on = HEAT_ON.load(Ordering::SeqCst);
    
            if current_snapshot == self.temp_snapshot { //<2>
                cx.waker().wake_by_ref();
                return Poll::Pending
            }
            if current_snapshot < desired_temp && heat_on == false { //<3>
                HEAT_ON.store(true, Ordering::SeqCst);
            }
            else if current_snapshot > desired_temp && heat_on == true { //<4>
                HEAT_ON.store(false, Ordering::SeqCst);
            }
            clearscreen::clear().unwrap(); // <5>
            println!("Temperature: {}\nDesired Temp: {}\nHeater On: {}", //<6>
                current_snapshot as f32 / 100.0, 
                desired_temp as f32 / 100.0, 
                heat_on);
            self.temp_snapshot = current_snapshot; //<7>
            cx.waker().wake_by_ref();
            return Poll::Pending
    }
}


pub struct HeaterFuture {
    pub time_snapshot: Instant,
}

impl HeaterFuture {
    pub fn new() -> Self {
        HeaterFuture {
            time_snapshot: Instant::now()
        }
    }
}

impl Future for HeaterFuture {
    type Output = ();

    fn poll( 
	    mut self: Pin<&mut Self>, 
	    cx: &mut Context<'_> 
    ) -> Poll<Self::Output> {        
        if HEAT_ON.load(Ordering::SeqCst) == false { // <1>
            self.time_snapshot = Instant::now();
            cx.waker().wake_by_ref();   
            return Poll::Pending
        }
        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(self.time_snapshot) < Duration::from_secs(3) { // <2>
            cx.waker().wake_by_ref();
            return Poll::Pending
        }
        TEMP.fetch_add(3, Ordering::SeqCst); // <3>
        self.time_snapshot = Instant::now();
        cx.waker().wake_by_ref();
        return Poll::Pending
    }
}

pub struct HeatLossFuture {
    pub time_snapshot: Instant,
}
impl HeatLossFuture {
    pub fn new() -> Self {
        HeatLossFuture {
            time_snapshot: Instant::now()
        }
    }
}

impl Future for HeatLossFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) ->
                                          Poll<Self::Output> {
        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(self.time_snapshot) >
                                        Duration::from_secs(3) {
            TEMP.fetch_sub(1, Ordering::SeqCst);
            self.time_snapshot = Instant::now();
        }
        cx.waker().wake_by_ref();
        return Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let display = tokio::spawn(async {
        DisplayFuture::new().await;
    });
    let heat_loss = tokio::spawn(async {
        HeatLossFuture::new().await;
    });
    let heater = tokio::spawn(async {
        HeaterFuture::new().await;
    });
    display.await.unwrap();
    heat_loss.await.unwrap();
    heater.await.unwrap();
}
