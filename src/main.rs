// Обеспечиваем работу как CLI, так и GUI
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use archlens::cli;
use std::env;

#[tokio::main]
async fn main() {
    let _args: Vec<String> = env::args().collect();
    // Always run CLI
    match cli::run().await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("❌ CLI error: {}", e);
            std::process::exit(1);
        }
    }
}
