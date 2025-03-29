use std::path::PathBuf;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncReadExt;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

async fn read_file(filename: &str) -> Result<String, std::io::Error> {
    let mut file = AsyncFile::open(filename).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

async fn watch_file_changes(tx: watch::Sender<bool>) {
    let path = PathBuf::from("data.txt"); //<1>

    let mut last_modified = None; // <2>
    loop { //<3>
        if let Ok(metadata) = path.metadata() { 
            let modified = metadata.modified().unwrap(); //<4>

            if last_modified != Some(modified) { //<5>
                last_modified = Some(modified);
                let _ = tx.send(true);
            }
        }
        sleep(Duration::from_millis(100)).await; //<6>
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = watch::channel(false); //<1>

    tokio::spawn(watch_file_changes(tx)); //<2>

    loop { //<3>
        // Wait for a change in the file  // ファイルの更新を待つ
        let _ = rx.changed().await; //<4>
        
        // ファイルを読んで内容をコンソールに表示
        // Read the file and print its contents to the console
        if let Ok(contents) = read_file("data.txt").await { //<5>
            println!("{}", contents);
        }
    }
}