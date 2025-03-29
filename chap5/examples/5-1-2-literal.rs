#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
use std::fs::OpenOptions;
use std::io::{Write, self};
use std::time::Instant;
use rand::Rng;
use std::ops::Coroutine;
use std::pin::Pin;

fn main() -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let numbers: Vec<i32> = (0..200000).map(|_| rng.r#gen()).collect();
    
    let mut write_coroutine = #[coroutine] |_v| {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("numbers.txt")?;

        loop {
            let tmp = yield;
            writeln!(file, "{}", tmp)?;
        }
        #[allow(unreachable_code)]
        Ok::<(), std::io::Error>(())
    };

    let start = Instant::now();
    for &number in &numbers {
        Pin::new(&mut write_coroutine).resume(number);
    }

    let duration = start.elapsed();

    println!("Time elapsed in file operations is: {:?}", duration);
    Ok(())
}