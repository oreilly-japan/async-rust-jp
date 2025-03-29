use tokio;

async fn cleanup() {
    println!("cleanup background task started");
    let mut count = 0;
    loop {
        // std::thread::sleep(std::time::Duration::from_secs(5)); 
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl-c received!");
        count += 1;
        if count > 2 {
            std::process::exit(0);
        }
    }
}

#[tokio::main]
async fn main() {
    tokio::spawn(cleanup());
    loop {
    }
}
