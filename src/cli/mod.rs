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
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {}", err);
            handlers::handle_command(handlers::CliCommand::Help).await?;
            std::process::exit(1);
        }
    };
    
    handlers::handle_command(command).await
}

/// Парсинг аргументов командной строки
pub fn parse_args() -> std::result::Result<handlers::CliCommand, String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Ok(handlers::CliCommand::Help);
    }
    
    match args[1].as_str() {
        "analyze" => {
            if args.len() < 3 {
                return Err("Usage: archlens analyze <project_path>".to_string());
            }
            Ok(handlers::CliCommand::Analyze {
                project_path: args[2].clone()
            })
        },
        "export" => {
            if args.len() < 4 {
                return Err("Usage: archlens export <project_path> <format> [output_file]".to_string());
            }
            Ok(handlers::CliCommand::Export {
                project_path: args[2].clone(),
                format: args[3].clone(),
                output: args.get(4).cloned(),
            })
        },
        "structure" => {
            if args.len() < 3 {
                return Err("Usage: archlens structure <project_path>".to_string());
            }
            Ok(handlers::CliCommand::Structure {
                project_path: args[2].clone()
            })
        },
        "diagram" => {
            if args.len() < 4 {
                return Err("Usage: archlens diagram <project_path> <type> [output_file]".to_string());
            }
            Ok(handlers::CliCommand::Diagram {
                project_path: args[2].clone(),
                diagram_type: args[3].clone(),
                output: args.get(4).cloned(),
            })
        },
        "help" | "--help" | "-h" => Ok(handlers::CliCommand::Help),
        _ => Err(format!("Unknown command: {}", args[1])),
    }
} 