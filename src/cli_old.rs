use std::env;
use std::path::Path;
use serde_json;
use chrono;
use uuid;
use crate::types::*;

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
    // Используем полный архитектурный анализ вместо простой статистики
    use crate::types::*;
    use crate::file_scanner::FileScanner;
    use crate::exporter::Exporter;
    use crate::capsule_graph_builder::CapsuleGraphBuilder;
    use crate::capsule_enricher::CapsuleEnricher;
    use crate::validator_optimizer::ValidatorOptimizer;
    use std::path::PathBuf;
    
    // Создаем конфигурацию анализа
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path),
        include_patterns: vec![
            "**/*.rs".to_string(), 
            "**/*.ts".to_string(), 
            "**/*.js".to_string(), 
            "**/*.py".to_string(),
            "**/*.tsx".to_string(),
            "**/*.jsx".to_string(),
            "**/*.java".to_string(),
            "**/*.cpp".to_string(),
            "**/*.c".to_string(),
            "**/*.go".to_string(),
        ],
        exclude_patterns: vec![
            "**/target/**".to_string(), 
            "**/node_modules/**".to_string(), 
            "**/.git/**".to_string(),
            "**/dist/**".to_string(),
            "**/build/**".to_string(),
            "**/.next/**".to_string(),
            "**/coverage/**".to_string(),
        ],
        max_depth: Some(10),
        follow_symlinks: false,
        analyze_dependencies: true,
        extract_comments: true,
        parse_tests: false,
        experimental_features: false,
        generate_summaries: true,
        languages: vec![
            FileType::Rust, 
            FileType::TypeScript, 
            FileType::JavaScript, 
            FileType::Python,
            FileType::Java,
            FileType::Go,
            FileType::Cpp,
            FileType::C,
        ],
    };

    // Сканируем проект
    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth.map(|d| d as usize),
    ).map_err(|e| format!("Ошибка создания сканера: {e}"))?;

    let files = scanner.scan_project(&config.project_path)
        .map_err(|e| format!("Ошибка сканирования: {e}"))?;

    // Создаем капсулы из найденных файлов
    let mut capsules = std::collections::HashMap::new();
    let mut layers = std::collections::HashMap::new();
    
    if files.is_empty() {
        // Возвращаем информацию о том, что проект пуст
        return Ok(format!(
            "# 🏗️ ArchLens AI Compact Analysis\n\n📁 Проект: {}\n⚠️ Статус: Проект пуст или не содержит поддерживаемых файлов\n\n🔍 Поддерживаемые файлы: .rs, .ts, .js, .py, .java, .cpp, .c, .go\n📅 Анализ выполнен: {}\n🔢 Токенов: ~100\n",
            project_path,
            chrono::Utc::now().to_rfc3339()
        ));
    }

    // Создаем капсулы из найденных файлов  
    for file in files.iter() {
        let capsule_id = uuid::Uuid::new_v4();
        let layer_name = determine_layer(&file.path);
        
        // Анализируем сложность файла
        let complexity = calculate_file_complexity(&file.path, &file.file_type);
        
        let capsule = Capsule {
            id: capsule_id,
            name: file.path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            capsule_type: determine_capsule_type(&file.file_type),
            file_path: file.path.clone(),
            line_start: 1,
            line_end: file.lines_count,
            size: file.lines_count,
            complexity,
            dependencies: vec![],
            layer: Some(layer_name.clone()),
            summary: None,
            description: Some(format!("Компонент {}", file.path.file_name().unwrap_or_default().to_string_lossy())),
            warnings: convert_warnings_to_analysis_warnings(generate_file_warnings(&file.path, complexity)),
            status: CapsuleStatus::Active,
            priority: if complexity > 50 { Priority::High } else if complexity > 20 { Priority::Medium } else { Priority::Low },
            tags: vec![layer_name.to_lowercase()],
            metadata: std::collections::HashMap::new(),
            quality_score: if complexity > 50 { 0.4 } else if complexity > 20 { 0.7 } else { 0.9 },
            slogan: Some(format!("Компонент {}", file.path.file_name().unwrap_or_default().to_string_lossy())),
            dependents: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        
        capsules.insert(capsule_id, capsule);
        layers.entry(layer_name).or_insert_with(Vec::new).push(capsule_id);
    }

    // Строим граф
    let mut graph_builder = CapsuleGraphBuilder::new();
    let capsules_vec: Vec<Capsule> = capsules.into_values().collect();
    let graph = graph_builder.build_graph(&capsules_vec)
        .map_err(|e| format!("Ошибка построения графа: {e}"))?;

    // Обогащаем граф
    let enricher = CapsuleEnricher::new();
    let enriched_graph = enricher.enrich_graph(&graph)
        .map_err(|e| format!("Ошибка обогащения графа: {e}"))?;

    // Валидируем и оптимизируем
    let validator = ValidatorOptimizer::new();
    let validated_graph = validator.validate_and_optimize(&enriched_graph)
        .map_err(|e| format!("Ошибка валидации: {e}"))?;

    // Экспортируем в AI Compact формат
    let exporter = Exporter::new();
    let ai_compact = exporter.export_to_ai_compact(&validated_graph)
        .map_err(|e| format!("Ошибка экспорта AI Compact: {e}"))?;

    Ok(ai_compact)
}

// Определяем архитектурный слой по пути файла
fn determine_layer(path: &std::path::Path) -> String {
    if let Some(parent) = path.parent() {
        if let Some(dir_name) = parent.file_name() {
            if let Some(dir_str) = dir_name.to_str() {
                return match dir_str {
                    "src" | "lib" => "Core".to_string(),
                    "api" | "controllers" | "routes" => "API".to_string(),
                    "ui" | "components" | "views" => "UI".to_string(),
                    "utils" | "helpers" | "tools" => "Utils".to_string(),
                    "models" | "entities" | "domain" => "Business".to_string(),
                    "services" | "business" => "Business".to_string(),
                    "data" | "database" | "db" => "Data".to_string(),
                    "tests" | "test" => "Tests".to_string(),
                    "mcp" | "server" => "Infrastructure".to_string(),
                    _ => "Other".to_string(),
                };
            }
        }
    }
    "Core".to_string()
}

// Определяем тип капсулы по типу файла
fn determine_capsule_type(file_type: &crate::types::FileType) -> CapsuleType {
    match file_type {
        crate::types::FileType::Rust => CapsuleType::Module,
        crate::types::FileType::JavaScript | crate::types::FileType::TypeScript => CapsuleType::Module,
        crate::types::FileType::Python => CapsuleType::Module,
        crate::types::FileType::Java => CapsuleType::Class,
        crate::types::FileType::Go => CapsuleType::Module,
        crate::types::FileType::Cpp | crate::types::FileType::C => CapsuleType::Module,
        _ => CapsuleType::Module,
    }
}

// Вычисляем сложность файла на основе его содержимого
fn calculate_file_complexity(path: &std::path::Path, file_type: &crate::types::FileType) -> u32 {
    if let Ok(content) = std::fs::read_to_string(path) {
        let lines = content.lines().count();
        let functions = content.matches("fn ").count() + content.matches("function ").count() + content.matches("def ").count();
        let classes = content.matches("class ").count() + content.matches("struct ").count() + content.matches("impl ").count();
        let imports = content.matches("use ").count() + content.matches("import ").count() + content.matches("require(").count();
        let conditions = content.matches("if ").count() + content.matches("match ").count() + content.matches("switch ").count();
        let loops = content.matches("for ").count() + content.matches("while ").count() + content.matches("loop ").count();
        
        // Алгоритм подсчета сложности
        let base_complexity = (lines / 10) as u32;
        let functional_complexity = (functions * 3 + classes * 5 + imports) as u32;
        let logical_complexity = (conditions * 2 + loops * 2) as u32;
        
        base_complexity + functional_complexity + logical_complexity
    } else {
        1 // Минимальная сложность для недоступных файлов
    }
}

// Генерируем предупреждения для файла
fn generate_file_warnings(path: &std::path::Path, complexity: u32) -> Vec<String> {
    let mut warnings = Vec::new();
    
    if complexity > 100 {
        warnings.push("Очень высокая сложность файла - рассмотрите разбиение на модули".to_string());
    } else if complexity > 50 {
        warnings.push("Высокая сложность файла - требуется рефакторинг".to_string());
    }
    
    if let Ok(content) = std::fs::read_to_string(path) {
        let lines = content.lines().count();
        if lines > 1000 {
            warnings.push("Очень большой файл - рассмотрите разбиение".to_string());
        }
        
        // Проверяем на отсутствие документации
        if !content.contains("///") && !content.contains("/**") && !content.contains("\"\"\"") {
            warnings.push("Отсутствует документация".to_string());
        }
        
        // Проверяем на todo/fixme
        if content.contains("TODO") || content.contains("FIXME") || content.contains("#ДОДЕЛАТЬ") {
            warnings.push("Содержит незавершенный код".to_string());
        }
    }
    
    warnings
}

fn generate_mermaid_diagram(project_path: &str) -> std::result::Result<String, String> {
    let structure = get_project_structure(project_path)?;
    
    let mut result = String::new();
    result.push_str("graph TD\n");
    result.push_str(&format!("    A[{}] --> B[src]\n", 
        Path::new(project_path).file_name()
            .unwrap_or_default()
            .to_string_lossy()));
    
    // Собираем информацию о модулях и их зависимостях
    let mut module_dependencies = analyze_module_dependencies(project_path)?;
    let mut node_counter = 0;
    let mut file_nodes = std::collections::HashMap::new();
    
    // Создаем узлы для каждого типа файлов
    for (ext, count) in &structure.file_types {
        if !ext.is_empty() && count > &0 {
            let ext_clean = ext.replace(".", "");
            result.push_str(&format!("    B --> {}[{} files: {}]\n", ext_clean, ext, count));
            file_nodes.insert(ext.clone(), ext_clean);
        }
    }
    
    // Добавляем связи между модулями на основе анализа зависимостей
    result.push_str("\n    %% Связи между модулями\n");
    
    for dependency in &module_dependencies {
        let from_node = sanitize_node_name(&dependency.from_module);
        let to_node = sanitize_node_name(&dependency.to_module);
        
        // Создаем узлы для модулей если они еще не созданы
        if !file_nodes.values().any(|v| v == &from_node) {
            result.push_str(&format!("    {from_node}[{from_module}]\n", 
                from_module = dependency.from_module));
        }
        if !file_nodes.values().any(|v| v == &to_node) {
            result.push_str(&format!("    {to_node}[{to_module}]\n", 
                to_module = dependency.to_module));
        }
        
        // Добавляем связь между модулями
        let arrow_type = match dependency.dependency_type.as_str() {
            "import" => "-->",
            "use" => "-.->",
            "include" => "==>",
            _ => "--->"
        };
        
        result.push_str(&format!("    {from_node} {arrow_type} {to_node}\n"));
    }
    
    // Добавляем стилизацию
    result.push_str("\n    %% Стилизация\n");
    result.push_str("    classDef rustFile fill:#dea584,stroke:#8b4513,stroke-width:2px\n");
    result.push_str("    classDef jsFile fill:#f7df1e,stroke:#323330,stroke-width:2px\n");
    result.push_str("    classDef pyFile fill:#3776ab,stroke:#ffde57,stroke-width:2px\n");
    result.push_str("    classDef configFile fill:#e6f3ff,stroke:#1e88e5,stroke-width:2px\n");
    
    Ok(result)
}

/// Анализирует зависимости между модулями
fn analyze_module_dependencies(project_path: &str) -> std::result::Result<Vec<ModuleDependency>, String> {
    let mut dependencies = Vec::new();
    let path = Path::new(project_path);
    
    if !path.exists() {
        return Err("Путь к проекту не существует".to_string());
    }
    
    // Сканируем все файлы проекта
    scan_for_dependencies(path, &mut dependencies, "")?;
    
    Ok(dependencies)
}

/// Рекурсивно сканирует файлы на предмет зависимостей
fn scan_for_dependencies(dir: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_dir() {
                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    
                    if !dir_name.starts_with('.') && dir_name != "target" && dir_name != "node_modules" {
                        scan_for_dependencies(&path, dependencies, dir_name)?;
                    }
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            match ext_str {
                                "rs" => analyze_rust_file(&path, dependencies, current_module)?,
                                "js" | "ts" => analyze_js_file(&path, dependencies, current_module)?,
                                "py" => analyze_python_file(&path, dependencies, current_module)?,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Анализирует Rust файл на предмет зависимостей
fn analyze_rust_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let file_name = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let from_module = if current_module.is_empty() { 
            file_name.to_string() 
        } else { 
            format!("{}::{}", current_module, file_name) 
        };
        
        // Ищем use statements
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                if let Some(imported_module) = extract_rust_import(trimmed) {
                    dependencies.push(ModuleDependency {
                        from_module: from_module.clone(),
                        to_module: imported_module,
                        dependency_type: "use".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(())
}

/// Анализирует JavaScript/TypeScript файл на предмет зависимостей
fn analyze_js_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let file_name = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let from_module = if current_module.is_empty() { 
            file_name.to_string() 
        } else { 
            format!("{}/{}", current_module, file_name) 
        };
        
        // Ищем import statements
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(imported_module) = extract_js_import(trimmed) {
                    dependencies.push(ModuleDependency {
                        from_module: from_module.clone(),
                        to_module: imported_module,
                        dependency_type: "import".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(())
}

/// Анализирует Python файл на предмет зависимостей
fn analyze_python_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let file_name = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let from_module = if current_module.is_empty() { 
            file_name.to_string() 
        } else { 
            format!("{}.{}", current_module, file_name) 
        };
        
        // Ищем import statements
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                if let Some(imported_module) = extract_python_import(trimmed) {
                    dependencies.push(ModuleDependency {
                        from_module: from_module.clone(),
                        to_module: imported_module,
                        dependency_type: "import".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(())
}

/// Извлекает имя модуля из Rust use statement
fn extract_rust_import(line: &str) -> Option<String> {
    // Упрощенная логика - можно расширить для более сложных случаев
    if let Some(start) = line.find("use ") {
        let import_part = &line[start + 4..];
        if let Some(end) = import_part.find(';') {
            let module_path = import_part[..end].trim();
            // Берем только первую часть пути (основной модуль)
            if let Some(first_part) = module_path.split("::").next() {
                return Some(first_part.to_string());
            }
        }
    }
    None
}

/// Извлекает имя модуля из JavaScript/TypeScript import statement
fn extract_js_import(line: &str) -> Option<String> {
    // Ищем 'from "module"' или 'from 'module''
    if let Some(from_pos) = line.find("from ") {
        let from_part = &line[from_pos + 5..];
        if let Some(quote_start) = from_part.find(&['"', '\''][..]) {
            let quote_char = from_part.chars().nth(quote_start).unwrap();
            if let Some(quote_end) = from_part[quote_start + 1..].find(quote_char) {
                let module_name = &from_part[quote_start + 1..quote_start + 1 + quote_end];
                return Some(module_name.to_string());
            }
        }
    }
    None
}

/// Извлекает имя модуля из Python import statement
fn extract_python_import(line: &str) -> Option<String> {
    if line.starts_with("import ") {
        let import_part = &line[7..];
        if let Some(module_name) = import_part.split_whitespace().next() {
            return Some(module_name.to_string());
        }
    } else if line.starts_with("from ") {
        let from_part = &line[5..];
        if let Some(module_name) = from_part.split_whitespace().next() {
            return Some(module_name.to_string());
        }
    }
    None
}

/// Санитизация имен узлов для Mermaid
fn sanitize_node_name(name: &str) -> String {
    name.replace("::", "_")
        .replace("/", "_")
        .replace(".", "_")
        .replace("-", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Структура для хранения зависимостей между модулями
#[derive(Debug)]
struct ModuleDependency {
    pub from_module: String,
    pub to_module: String,
    pub dependency_type: String,
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

// Конвертирует строковые предупреждения в AnalysisWarning
fn convert_warnings_to_analysis_warnings(string_warnings: Vec<String>) -> Vec<crate::types::AnalysisWarning> {
    string_warnings.into_iter().map(|warning| {
        let level = if warning.contains("Очень высокая") || warning.contains("критич") {
            crate::types::Priority::Critical
        } else if warning.contains("высокая") || warning.contains("Высокая") {
            crate::types::Priority::High
        } else {
            crate::types::Priority::Medium
        };
        
        crate::types::AnalysisWarning {
            message: warning.clone(),
            level,
            category: "file_analysis".to_string(),
            capsule_id: None,
            suggestion: Some(format!("Рекомендация для: {}", warning)),
        }
    }).collect()
} 