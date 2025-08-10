use crate::types::*;
use std::path::Path;

use super::parser;

pub async fn handle_command(
    command: parser::CliCommand,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use super::{diagram, export, stats};

    match command {
        parser::CliCommand::Help => {
            print_help();
        }
        parser::CliCommand::Version => {
            println!("archlens v{}", env!("CARGO_PKG_VERSION"));
        }
        parser::CliCommand::Analyze {
            project_path,
            verbose: _verbose,
            include_tests: _include_tests,
            deep,
        } => {
            eprintln!(
                "🔍 Анализ проекта: {}{}",
                project_path,
                if deep { " (deep)" } else { "" }
            );
            if !Path::new(&project_path).exists() {
                eprintln!("❌ Путь не существует: {}", project_path);
                std::process::exit(1);
            }
            if deep {
                match run_deep_pipeline(&project_path) {
                    Ok(json) => println!("{}", json),
                    Err(err) => {
                        eprintln!(
                            "⚠️ Ошибка deep-анализа: {}. Переход к базовой статистике.",
                            err
                        );
                        match stats::get_project_stats(&project_path) {
                            Ok(s) => println!("{}", serde_json::to_string_pretty(&s)?),
                            Err(e) => {
                                eprintln!("❌ Ошибка анализа: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            } else {
                match stats::get_project_stats(&project_path) {
                    Ok(stats) => {
                        eprintln!("✅ Анализ завершен успешно");
                        println!("{}", serde_json::to_string_pretty(&stats)?);
                    }
                    Err(err) => {
                        eprintln!("❌ Ошибка анализа: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        parser::CliCommand::Export {
            project_path,
            format,
            output,
            options: _options,
        } => {
            eprintln!(
                "📤 Экспорт проекта: {} в формат: {:?}",
                project_path, format
            );
            match format {
                parser::ExportFormat::AiCompact => {
                    match export::generate_ai_compact(&project_path) {
                        Ok(content) => {
                            if let Some(output_file) = output {
                                std::fs::write(&output_file, &content)?;
                                eprintln!("✅ AI Compact анализ сохранен в: {}", output_file);
                            } else {
                                println!("{}", content);
                            }
                        }
                        Err(err) => {
                            eprintln!("❌ Ошибка экспорта: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                parser::ExportFormat::Json
                | parser::ExportFormat::Markdown
                | parser::ExportFormat::Html => {
                    eprintln!("❌ Неподдерживаемый формат: {:?}", format);
                    eprintln!("Доступные форматы: ai_compact");
                    std::process::exit(1);
                }
            }
        }
        parser::CliCommand::Structure { project_path, .. } => {
            eprintln!("📊 Структура проекта: {}", project_path);
            match stats::get_project_structure(&project_path) {
                Ok(structure) => {
                    println!("{}", serde_json::to_string_pretty(&structure)?);
                }
                Err(err) => {
                    eprintln!("❌ Ошибка получения структуры: {}", err);
                    std::process::exit(1);
                }
            }
        }
        parser::CliCommand::Diagram {
            project_path,
            diagram_type,
            output,
            include_metrics: _,
        } => {
            eprintln!(
                "📈 Генерация диаграммы: {} типа: {:?}",
                project_path, diagram_type
            );
            let diag_type = match diagram_type {
                parser::DiagramType::Mermaid => "mermaid",
                parser::DiagramType::Dot => "dot",
                parser::DiagramType::Svg => "svg",
            };
            match diag_type {
                "mermaid" => {
                    // Сначала попробуем построить граф и отдать мермайд на его основе
                    match build_graph_mermaid(&project_path) {
                        Ok(content) => {
                            if let Some(out) = output {
                                std::fs::write(&out, &content)?;
                                eprintln!("✅ Mermaid диаграмма (graph) сохранена в: {}", out);
                            } else {
                                println!("{}", content);
                            }
                        }
                        Err(_) => {
                            // Фоллбек на старый генератор по импортам
                            match diagram::generate_mermaid_diagram(&project_path) {
                                Ok(content) => {
                                    if let Some(out) = output {
                                        std::fs::write(&out, &content)?;
                                        eprintln!("✅ Mermaid диаграмма сохранена в: {}", out);
                                    } else {
                                        println!("{}", content);
                                    }
                                }
                                Err(err) => {
                                    eprintln!("❌ Ошибка генерации диаграммы: {}", err);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
                _ => {
                    eprintln!("❌ Неподдерживаемый тип диаграммы: {}", diag_type);
                    eprintln!("Доступные типы: mermaid");
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

pub fn build_graph_mermaid(project_path: &str) -> std::result::Result<String, String> {
    use crate::capsule_constructor::CapsuleConstructor;
    use crate::capsule_graph_builder::CapsuleGraphBuilder;
    use crate::exporter::Exporter;
    use crate::file_scanner::FileScanner;
    use crate::parser_ast::ParserAST;
    use crate::validator_optimizer::ValidatorOptimizer;

    let scanner = FileScanner::new(
        vec![
            "**/*.rs".into(),
            "**/*.ts".into(),
            "**/*.js".into(),
            "**/*.py".into(),
            "**/*.java".into(),
            "**/*.go".into(),
            "**/*.cpp".into(),
            "**/*.c".into(),
        ],
        vec![
            "**/target/**".into(),
            "**/node_modules/**".into(),
            "**/.git/**".into(),
            "**/dist/**".into(),
            "**/build/**".into(),
        ],
        Some(6),
    )
    .map_err(|e| e.to_string())?;
    let files = scanner
        .scan_files(Path::new(project_path))
        .map_err(|e| e.to_string())?;

    let mut parser = ParserAST::new().map_err(|e| e.to_string())?;
    let constructor = CapsuleConstructor::new();
    let mut capsules: Vec<Capsule> = Vec::new();

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(&file.path) {
            if let Ok(nodes) = parser.parse_file(&file.path, &content, &file.file_type) {
                let mut caps = constructor
                    .create_capsules(&nodes, &file.path.clone())
                    .map_err(|e| e.to_string())?;
                capsules.append(&mut caps);
            }
        }
    }
    if capsules.is_empty() {
        return Err("No capsules".into());
    }
    let mut builder = CapsuleGraphBuilder::new();
    let graph = builder.build_graph(&capsules).map_err(|e| e.to_string())?;
    let validator = ValidatorOptimizer::new();
    let graph = validator
        .validate_and_optimize(&graph)
        .map_err(|e| e.to_string())?;
    let exporter = Exporter::new();
    exporter
        .export_to_mermaid(&graph)
        .map_err(|e| e.to_string())
}

pub fn run_deep_pipeline(project_path: &str) -> std::result::Result<String, String> {
    use crate::capsule_constructor::CapsuleConstructor;
    use crate::capsule_graph_builder::CapsuleGraphBuilder;
    use crate::file_scanner::FileScanner;
    use crate::parser_ast::ParserAST;
    use crate::validator_optimizer::ValidatorOptimizer;

    let scanner = FileScanner::new(
        vec![
            "**/*.rs".into(),
            "**/*.ts".into(),
            "**/*.js".into(),
            "**/*.py".into(),
            "**/*.java".into(),
            "**/*.go".into(),
            "**/*.cpp".into(),
            "**/*.c".into(),
        ],
        vec![
            "**/target/**".into(),
            "**/node_modules/**".into(),
            "**/.git/**".into(),
            "**/dist/**".into(),
            "**/build/**".into(),
        ],
        Some(10),
    )
    .map_err(|e| e.to_string())?;
    let files = scanner
        .scan_files(Path::new(project_path))
        .map_err(|e| e.to_string())?;

    let mut parser = ParserAST::new().map_err(|e| e.to_string())?;
    let constructor = CapsuleConstructor::new();
    let mut capsules: Vec<Capsule> = Vec::new();

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(&file.path) {
            if let Ok(nodes) = parser.parse_file(&file.path, &content, &file.file_type) {
                let mut caps = constructor
                    .create_capsules(&nodes, &file.path.clone())
                    .map_err(|e| e.to_string())?;
                capsules.append(&mut caps);
            }
        }
    }

    let mut builder = CapsuleGraphBuilder::new();
    let graph = builder.build_graph(&capsules).map_err(|e| e.to_string())?;
    let validator = ValidatorOptimizer::new();
    let validated_graph = validator
        .validate_and_optimize(&graph)
        .map_err(|e| e.to_string())?;

    let result = AnalysisResult {
        graph: validated_graph,
        warnings: Vec::new(),
        recommendations: vec!["Граф построен с использованием полного пайплайна".to_string()],
        export_formats: vec![
            ExportFormat::JSON,
            ExportFormat::Mermaid,
            ExportFormat::DOT,
            ExportFormat::SVG,
            ExportFormat::AICompact,
        ],
    };

    Ok(serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?)
}

pub fn print_help() {
    println!("🏗️ ArchLens - Анализатор архитектуры кода");
    println!();
    println!("ИСПОЛЬЗОВАНИЕ:");
    println!("  archlens <КОМАНДА> [ОПЦИИ]");
    println!();
    println!("КОМАНДЫ:");
    println!(
        "  analyze <path> [--verbose] [--include-tests] [--deep]  Анализ (deep — полный пайплайн)"
    );
    println!("  export <path> <format> [--output <file>]               Экспорт (ai_compact)");
    println!("  structure <path> [--max-depth N] [--show-metrics]      Структура проекта");
    println!("  diagram <path> <type> [--output <file>]               Диаграмма архитектуры");
    println!("  version                                               Печать версии");
    println!("  help                                                  Показать эту справку");
}
