use std::time::Instant;
use reqwest::Error;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://jsonplaceholder.typicode.com/posts/1";
    let start_time = Instant::now();
    
    // let _ = reqwest::get(url).await?;

    /* 
    let first = reqwest::get(url);
    let second = reqwest::get(url);
    let third = reqwest::get(url);
    let fourth = reqwest::get(url);
    
    let first = first.await?;
    let second = second.await?;
    let third = third.await?;
    let fourth = fourth.await?;
    */
  j
    let (_, _, _, _) = tokio::join!(
        reqwest::get(url),
        reqwest::get(url),
        reqwest::get(url),
        reqwest::get(url),
    );
    

    let elapsed_time = start_time.elapsed();
    println!("Request took {} ms", elapsed_time.as_millis());

    Ok(())
}
