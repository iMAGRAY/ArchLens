// Модуль командной строки - организует все CLI подмодули

pub mod parser;
pub mod handlers;
pub mod stats;
pub mod export;
pub mod diagram;

pub use parser::*;
pub use handlers::*;
pub use stats::*;
pub use export::*;
pub use diagram::*;

use std::env;

/// Основная функция CLI для запуска всех команд
pub async fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let command = match parser::parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {}", err);
            handlers::print_help();
            std::process::exit(1);
        }
    };
    
    handlers::handle_command(command).await
} 