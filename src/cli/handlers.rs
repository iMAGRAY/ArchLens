use std::path::Path;
use crate::types::*;

#[derive(Debug)]
pub enum CliCommand {
    Analyze { project_path: String },
    Export { project_path: String, format: String, output: Option<String> },
    Structure { project_path: String },
    Diagram { project_path: String, diagram_type: String, output: Option<String> },
    Help,
}

pub async fn handle_command(command: CliCommand) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use super::{stats, export, diagram};
    
    match command {
        CliCommand::Help => {
            print_help();
        },
        CliCommand::Analyze { project_path } => {
            println!("🔍 Анализ проекта: {}", project_path);
            
            if !Path::new(&project_path).exists() {
                eprintln!("❌ Путь не существует: {}", project_path);
                std::process::exit(1);
            }
            
            match stats::get_project_stats(&project_path) {
                Ok(stats) => {
                    println!("✅ Анализ завершен успешно");
                    println!("{}", serde_json::to_string_pretty(&stats)?);
                },
                Err(err) => {
                    eprintln!("❌ Ошибка анализа: {}", err);
                    std::process::exit(1);
                }
            }
        },
        CliCommand::Export { project_path, format, output } => {
            println!("📤 Экспорт проекта: {} в формат: {}", project_path, format);
            
            match format.as_str() {
                "ai_compact" => {
                    match export::generate_ai_compact(&project_path) {
                        Ok(content) => {
                            if let Some(output_file) = output {
                                std::fs::write(&output_file, &content)?;
                                println!("✅ AI Compact анализ сохранен в: {}", output_file);
                            } else {
                                println!("{}", content);
                            }
                        },
                        Err(err) => {
                            eprintln!("❌ Ошибка экспорта: {}", err);
                            std::process::exit(1);
                        }
                    }
                },
                _ => {
                    eprintln!("❌ Неподдерживаемый формат: {}", format);
                    eprintln!("Доступные форматы: ai_compact");
                    std::process::exit(1);
                }
            }
        },
        CliCommand::Structure { project_path } => {
            println!("📊 Структура проекта: {}", project_path);
            
            match stats::get_project_structure(&project_path) {
                Ok(structure) => {
                    println!("{}", serde_json::to_string_pretty(&structure)?);
                },
                Err(err) => {
                    eprintln!("❌ Ошибка получения структуры: {}", err);
                    std::process::exit(1);
                }
            }
        },
        CliCommand::Diagram { project_path, diagram_type, output } => {
            println!("📈 Генерация диаграммы: {} типа: {}", project_path, diagram_type);
            
            match diagram_type.as_str() {
                "mermaid" => {
                    match diagram::generate_mermaid_diagram(&project_path) {
                        Ok(content) => {
                            let output_path = output.unwrap_or_else(|| "diagram.mmd".to_string());
                            std::fs::write(&output_path, content)?;
                            println!("✅ Mermaid диаграмма сохранена в: {}", output_path);
                        },
                        Err(err) => {
                            eprintln!("❌ Ошибка генерации диаграммы: {}", err);
                            std::process::exit(1);
                        }
                    }
                },
                _ => {
                    eprintln!("❌ Неподдерживаемый тип диаграммы: {}", diagram_type);
                    eprintln!("Доступные типы: mermaid");
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("🏗️ ArchLens - Анализатор архитектуры кода");
    println!();
    println!("ИСПОЛЬЗОВАНИЕ:");
    println!("  archlens <КОМАНДА> [ОПЦИИ]");
    println!();
    println!("КОМАНДЫ:");
    println!("  analyze <path>                 Анализ архитектуры проекта");
    println!("  export <path> <format> [out]   Экспорт анализа в указанный формат");
    println!("  structure <path>               Получение структуры проекта");
    println!("  diagram <path> <type> [out]    Генерация диаграммы архитектуры");
    println!("  help                           Показать эту справку");
    println!();
    println!("ФОРМАТЫ ЭКСПОРТА:");
    println!("  ai_compact    Компактный формат для AI (~2800 токенов)");
    println!();
    println!("ТИПЫ ДИАГРАММ:");
    println!("  mermaid       Mermaid диаграмма");
    println!();
    println!("ПРИМЕРЫ:");
    println!("  archlens analyze /path/to/project");
    println!("  archlens export /path/to/project ai_compact analysis.txt");
    println!("  archlens diagram /path/to/project mermaid diagram.mmd");
    println!("  archlens structure /path/to/project");
} 