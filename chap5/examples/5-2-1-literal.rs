// 5-1-3 をコルーチンリテラルを用いて記述したもの

#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(iter_from_coroutine)]
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let path = "./data.txt";
    let coroutine = #[coroutine] || { 
        let file = File::open(path).unwrap();
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
    for number in std::iter::from_coroutine(coroutine) {
        println!("{:?}", number);
    }
    Ok(())
}
