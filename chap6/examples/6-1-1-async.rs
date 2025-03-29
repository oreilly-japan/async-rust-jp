// 6-1-1 を async fn を用いて書き直したもの

use std::sync::Arc;
use std::sync::atomic::{AtomicI16, AtomicBool};
use core::sync::atomic::Ordering;
use std::sync::LazyLock;
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


async fn display() {
    let mut temp_snapshot: i16 = TEMP.load(Ordering::SeqCst);

    loop {
        let current_snapshot = TEMP.load(Ordering::SeqCst); // <1>
        let desired_temp = DESIRED_TEMP.load(Ordering::SeqCst);
        let heat_on = HEAT_ON.load(Ordering::SeqCst);

        if current_snapshot == temp_snapshot { //<2>
            tokio::task::yield_now().await;
            continue;
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
        temp_snapshot = current_snapshot; 
        tokio::task::yield_now().await;
    }
}

async fn heater() {
    let mut time_snapshot: Instant = Instant::now();
    loop {
        if HEAT_ON.load(Ordering::SeqCst) == false { // <1>
            time_snapshot = Instant::now();
            tokio::task::yield_now().await;
            continue;
        }
        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(time_snapshot) < Duration::from_secs(3) { // <2>
            tokio::task::yield_now().await;
            continue;
        }
        TEMP.fetch_add(3, Ordering::SeqCst); // <3>
        time_snapshot = Instant::now();
        tokio::task::yield_now().await;
    }
}

async fn heat_loss() {
    let mut time_snapshot: Instant = Instant::now();
    loop {
        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(time_snapshot) >
                                        Duration::from_secs(3) {
            TEMP.fetch_sub(1, Ordering::SeqCst);
            time_snapshot = Instant::now();
        }
        tokio::task::yield_now().await;
    }
}
 
#[tokio::main]
async fn main() {
    let display = tokio::spawn(display());      
    let heat_loss = tokio::spawn(heat_loss());
    let heater = tokio::spawn(heater());
    display.await.unwrap();
    heat_loss.await.unwrap();
    heater.await.unwrap();
}
