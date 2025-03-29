#![feature(coroutines)]
#![feature(coroutine_trait)]
use std::fs::{OpenOptions, File};
use std::io::{Write, self};
use std::time::Instant;
use rand::Rng;

use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

struct WriteCoroutine {
    pub file_handle: File,
}

impl WriteCoroutine {
    fn new(path: &str) -> io::Result<Self> {
        let file_handle = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file_handle })
    }
}

impl Coroutine<i32> for WriteCoroutine {
    type Yield = ();
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, arg: i32) 
        -> CoroutineState<Self::Yield, Self::Return> {
        writeln!(self.file_handle, "{}", arg).unwrap();
        CoroutineState::Yielded(())
    }
}


fn main() -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let numbers: Vec<i32> = (0..200000).map(|_| rng.r#gen()).collect();
    
    let start = Instant::now();
    let mut coroutine = WriteCoroutine::new(
        "numbers.txt"
    )?;

    for &number in &numbers {
        Pin::new(&mut coroutine).resume(number);
    }
    
    let duration = start.elapsed();

    println!("Time elapsed in file operations is: {:?}", duration);
    Ok(())
}