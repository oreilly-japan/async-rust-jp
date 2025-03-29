use std::process::{Command, Output};

fn main() {
    let output: Output = Command::new("./connection_bin")
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", stderr);
    }
}

