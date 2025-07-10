use std::env;
use std::path::Path;
use serde_json;
use chrono;

#[derive(Debug)]
pub enum CliCommand {
    Analyze { project_path: String },
    Export { project_path: String, format: String, output: Option<String> },
    Structure { project_path: String },
    Diagram { project_path: String, diagram_type: String, output: Option<String> },
    Help,
}

pub fn parse_args() -> std::result::Result<CliCommand, String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Ok(CliCommand::Help);
    }
    
    match args[1].as_str() {
        "analyze" => {
            if args.len() < 3 {
                return Err("Usage: archlens analyze <project_path>".to_string());
            }
            Ok(CliCommand::Analyze {
                project_path: args[2].clone()
            })
        },
        "export" => {
            if args.len() < 4 {
                return Err("Usage: archlens export <project_path> <format> [output_file]".to_string());
            }
            Ok(CliCommand::Export {
                project_path: args[2].clone(),
                format: args[3].clone(),
                output: args.get(4).cloned(),
            })
        },
        "structure" => {
            if args.len() < 3 {
                return Err("Usage: archlens structure <project_path>".to_string());
            }
            Ok(CliCommand::Structure {
                project_path: args[2].clone()
            })
        },
        "diagram" => {
            if args.len() < 4 {
                return Err("Usage: archlens diagram <project_path> <type> [output_file]".to_string());
            }
            Ok(CliCommand::Diagram {
                project_path: args[2].clone(),
                diagram_type: args[3].clone(),
                output: args.get(4).cloned(),
            })
        },
        "help" | "--help" | "-h" => Ok(CliCommand::Help),
        _ => Err(format!("Unknown command: {}", args[1])),
    }
}

pub async fn run_cli() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {}", err);
            print_help();
            std::process::exit(1);
        }
    };
    
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
            
            // Простой вывод статистики проекта
            match get_project_stats(&project_path) {
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
                    match generate_ai_compact(&project_path) {
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
            
            match get_project_structure(&project_path) {
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
                    match generate_mermaid_diagram(&project_path) {
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

// Простые функции для CLI
fn get_project_stats(project_path: &str) -> std::result::Result<ProjectStats, String> {
    use std::fs;
    use std::collections::HashMap;
    
    if !Path::new(project_path).exists() {
        return Err("Указанный путь не существует".to_string());
    }
    
    let mut file_types = HashMap::new();
    let mut total_files = 0;
    let mut total_lines = 0;
    
    fn scan_directory(dir: &Path, file_types: &mut HashMap<String, usize>, 
                      total_files: &mut usize, total_lines: &mut usize) -> std::result::Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if !name.to_string_lossy().starts_with('.') 
                       && name != "node_modules" 
                       && name != "target" {
                        scan_directory(&path, file_types, total_files, total_lines)?;
                    }
                }
            } else {
                *total_files += 1;
                
                let extension = path.extension()
                    .map(|ext| format!(".{}", ext.to_string_lossy()))
                    .unwrap_or_else(|| "".to_string());
                
                *file_types.entry(extension.clone()).or_insert(0) += 1;
                
                // Подсчитываем строки для текстовых файлов
                if matches!(extension.as_str(), ".rs" | ".js" | ".ts" | ".py" | ".java" | ".cpp" | ".c" | ".h") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        *total_lines += content.lines().count();
                    }
                }
            }
        }
        Ok(())
    }
    
    let root_path = Path::new(project_path);
    scan_directory(root_path, &mut file_types, &mut total_files, &mut total_lines)
        .map_err(|e| format!("Ошибка сканирования: {}", e))?;
    
    Ok(ProjectStats {
        total_files,
        total_lines,
        file_types,
        project_path: project_path.to_string(),
        scanned_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn get_project_structure(project_path: &str) -> std::result::Result<ProjectStructure, String> {
    use std::fs;
    use std::collections::HashMap;
    
    if !Path::new(project_path).exists() {
        return Err("Указанный путь не существует".to_string());
    }
    
    let mut file_types = HashMap::new();
    let mut total_files = 0;
    let mut files = Vec::new();
    
    fn scan_directory(dir: &Path, file_types: &mut HashMap<String, usize>, 
                      total_files: &mut usize, files: &mut Vec<FileInfo>, 
                      root_path: &Path, depth: usize) -> std::result::Result<(), std::io::Error> {
        if depth > 10 { return Ok(()); }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if !name.to_string_lossy().starts_with('.') 
                       && name != "node_modules" 
                       && name != "target" {
                        scan_directory(&path, file_types, total_files, files, root_path, depth + 1)?;
                    }
                }
            } else {
                *total_files += 1;
                
                let extension = path.extension()
                    .map(|ext| format!(".{}", ext.to_string_lossy()))
                    .unwrap_or_else(|| "".to_string());
                
                *file_types.entry(extension.clone()).or_insert(0) += 1;
                
                if files.len() < 100 {
                    let relative_path = path.strip_prefix(root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    
                    files.push(FileInfo {
                        path: relative_path,
                        name: path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        extension,
                        size: entry.metadata()?.len(),
                    });
                }
            }
        }
        Ok(())
    }
    
    let root_path = Path::new(project_path);
    scan_directory(root_path, &mut file_types, &mut total_files, &mut files, root_path, 0)
        .map_err(|e| format!("Ошибка сканирования: {}", e))?;
    
    let layers = vec!["src".to_string()];
    
    Ok(ProjectStructure {
        total_files,
        file_types,
        layers,
        files,
    })
}

fn generate_ai_compact(project_path: &str) -> std::result::Result<String, String> {
    let stats = get_project_stats(project_path)?;
    let structure = get_project_structure(project_path)?;
    
    let mut result = String::new();
    result.push_str("# 🏗️ ArchLens AI Compact Analysis\n\n");
    result.push_str(&format!("📁 Проект: {}\n", project_path));
    result.push_str(&format!("📊 Файлов: {}, Строк: {}\n\n", stats.total_files, stats.total_lines));
    
    result.push_str("## 📋 Типы файлов:\n");
    for (ext, count) in &stats.file_types {
        if !ext.is_empty() {
            result.push_str(&format!("- {}: {} файлов\n", ext, count));
        }
    }
    
    result.push_str("\n## 🗂️ Структура:\n");
    for layer in &structure.layers {
        result.push_str(&format!("- {}\n", layer));
    }
    
    result.push_str(&format!("\n📅 Анализ выполнен: {}\n", stats.scanned_at));
    result.push_str(&format!("🔢 Токенов: ~{}\n", result.len() / 4));
    
    // #ДОДЕЛАТЬ: Добавить полный архитектурный анализ
    result.push_str("\n#ДОДЕЛАТЬ: Требуется интеграция с полным анализатором архитектуры\n");
    
    Ok(result)
}

fn generate_mermaid_diagram(project_path: &str) -> std::result::Result<String, String> {
    let structure = get_project_structure(project_path)?;
    
    let mut result = String::new();
    result.push_str("graph TD\n");
    result.push_str(&format!("    A[{}] --> B[src]\n", 
        Path::new(project_path).file_name()
            .unwrap_or_default()
            .to_string_lossy()));
    
    for (ext, count) in &structure.file_types {
        if !ext.is_empty() && count > &0 {
            let ext_clean = ext.replace(".", "");
            result.push_str(&format!("    B --> {}[{} files: {}]\n", ext_clean, ext, count));
        }
    }
    
    // #ДОДЕЛАТЬ: Добавить связи между модулями
    result.push_str("\n    %% #ДОДЕЛАТЬ: Требуется анализ зависимостей между модулями\n");
    
    Ok(result)
}

// Структуры для CLI результатов
#[derive(Debug, serde::Serialize)]
pub struct ProjectStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub file_types: std::collections::HashMap<String, usize>,
    pub project_path: String,
    pub scanned_at: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ProjectStructure {
    pub total_files: usize,
    pub file_types: std::collections::HashMap<String, usize>,
    pub layers: Vec<String>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size: u64,
} 