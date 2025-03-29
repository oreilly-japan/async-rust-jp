#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(iter_from_coroutine)]
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::ops::Coroutine;
use std::pin::Pin;

fn main() {
    let read_path = "numbers.txt";
    let write_path =  "output.txt";

    let mut reader = #[coroutine] || {
        let file = File::open(read_path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(Ok(line)) = lines.next() {
            if let Ok(number) = line.parse::<i32>() {
                yield number;
            } else {
                break;
            }
        }
    };

    let mut writer =  #[coroutine] |_v| {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(write_path)
            .unwrap();

        loop {
            let tmp = yield ();
            writeln!(file, "{}", tmp).unwrap();
        }
    };

    let mut write_pin = Pin::new(&mut writer);

    for number in std::iter::from_coroutine(reader) {
        write_pin.as_mut().resume(number);
    }
}