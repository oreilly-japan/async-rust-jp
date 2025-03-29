use std::future::ready;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut vec: Vec<String> = vec![];

    let mut closure = async || {
        vec.push(ready(String::from("")).await);
    };

    closure().await;

    /* 以下はコンパイルできない
    let mut closure2 = || async {
        vec.push(ready(String::from("")).await);
    };
    closure2().await;
    */    

}