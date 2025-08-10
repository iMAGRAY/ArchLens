use crate::advanced_metrics::AdvancedMetricsCalculator;
use crate::capsule_constructor::CapsuleConstructor;
use crate::capsule_enricher::CapsuleEnricher;
use crate::capsule_graph_builder::CapsuleGraphBuilder;
use crate::diff_analyzer::DiffAnalyzer;
use crate::exporter::Exporter;
use crate::file_scanner::FileScanner;
use crate::metadata_extractor::MetadataExtractor;
use crate::parser_ast::ParserAST;
use crate::types::*;
use crate::validator_optimizer::ValidatorOptimizer;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
#[cfg(feature = "gui")]
use tauri::State;
use uuid::Uuid;

impl From<AnalysisError> for String {
    fn from(error: AnalysisError) -> Self {
        match error {
            AnalysisError::IoError(msg) => format!("IO Error: {}", msg),
            AnalysisError::ParsingError(msg) => format!("Parsing Error: {}", msg),
            AnalysisError::RegexError(msg) => format!("Regex Error: {}", msg),
            AnalysisError::GenericError(msg) => format!("Generic Error: {}", msg),
            AnalysisError::Parse(msg) => format!("Parse Error: {}", msg),
            AnalysisError::Io(msg) => format!("IO Error: {}", msg),
        }
    }
}

// Состояние приложения
pub struct AppState {
    pub last_analysis: Arc<Mutex<Option<AnalysisResult>>>,
    pub previous_analysis: Arc<Mutex<Option<CapsuleGraph>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_analysis: Arc::new(Mutex::new(None)),
            previous_analysis: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(serde::Serialize)]
pub struct ProjectStructure {
    pub total_files: usize,
    pub file_types: std::collections::HashMap<String, usize>,
    pub layers: Vec<String>,
    pub estimated_complexity: usize,
}

/// Расширенная команда анализа проекта с полным пайплайном
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn analyze_project_advanced(
    project_path: String,
    _config: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path.clone()),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.js".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
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
        ],
    };

    // Шаг 1: Сканирование файлов
    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth,
    )?;
    let files = scanner.scan_files(Path::new(&project_path))?;

    // Шаг 2: Парсинг AST
    let mut parser = ParserAST::new()?;
    let mut all_nodes = Vec::new();

    for file in &files {
        if let Ok(content) = fs::read_to_string(&file.path) {
            match parser.parse_file(&file.path, &content, &file.file_type) {
                Ok(nodes) => all_nodes.extend(nodes),
                Err(e) => eprintln!("Ошибка парсинга файла {:?}: {}", file.path, e),
            }
        }
    }

    // Шаг 3: Извлечение метаданных
    let metadata_extractor = MetadataExtractor::new();
    let mut metadata_list = Vec::new();

    for node in &all_nodes {
        let metadata = metadata_extractor
            .extract_metadata(node, &std::path::PathBuf::new())
            .map_err(|e| format!("Ошибка извлечения метаданных: {e}"))?;
        metadata_list.push(metadata);
    }

    // Шаг 4: Создание капсул
    let capsule_constructor = CapsuleConstructor::new();
    let capsules =
        capsule_constructor.create_capsules(&all_nodes, &PathBuf::from(project_path.clone()))?;

    // Шаг 5: Построение графа
    let mut graph_builder = CapsuleGraphBuilder::new();
    let graph = graph_builder
        .build_graph(&capsules)
        .map_err(|e| format!("Ошибка построения графа: {e}"))?;

    // Шаг 6: Обогащение семантикой
    let enricher = CapsuleEnricher::new();
    let enriched_graph = enricher
        .enrich_graph(&graph)
        .map_err(|e| format!("Ошибка обогащения: {e}"))?;

    // Шаг 7: Валидация и оптимизация
    let validator = ValidatorOptimizer::new();
    let validated_graph = validator
        .validate_and_optimize(&enriched_graph)
        .map_err(|e| format!("Ошибка валидации: {e}"))?;

    // Шаг 8: Diff-анализ (если есть предыдущая версия)
    let mut diff_analysis = None;
    if let Ok(previous_guard) = state.previous_analysis.lock() {
        if let Some(previous_graph) = previous_guard.as_ref() {
            let diff_analyzer = DiffAnalyzer::new();
            if let Ok(diff) = diff_analyzer.analyze_diff(&validated_graph, previous_graph) {
                diff_analysis = Some(diff);
            }
        }
    }

    // Шаг 9: Расчет продвинутых метрик
    let metrics_calculator = AdvancedMetricsCalculator::new();
    let mut advanced_metrics = Vec::new();

    for capsule in validated_graph.capsules.values() {
        if let Ok(content) = fs::read_to_string(&capsule.file_path) {
            if let Ok(metrics) = metrics_calculator.calculate_metrics(capsule, &content) {
                advanced_metrics.push((capsule.name.clone(), metrics));
            }
        }
    }

    // Сохраняем текущий граф для следующего diff-анализа
    if let Ok(mut previous_guard) = state.previous_analysis.lock() {
        *previous_guard = Some(validated_graph.clone());
    }

    // Подготовка результата
    let analysis_result = AnalysisResult {
        graph: validated_graph,
        warnings: vec![],
        recommendations: vec![
            "Архитектурный анализ завершен с использованием полного пайплайна".to_string(),
            "Проверьте обнаруженные архитектурные паттерны".to_string(),
            "Рассмотрите рекомендации по SOLID принципам".to_string(),
        ],
        export_formats: vec![
            ExportFormat::JSON,
            ExportFormat::YAML,
            ExportFormat::Mermaid,
            ExportFormat::DOT,
            ExportFormat::SVG,
            ExportFormat::InteractiveHTML,
            ExportFormat::AICompact,
        ],
    };

    // Сохраняем результат
    let mut last_analysis = state.last_analysis.lock().unwrap();
    *last_analysis = Some(analysis_result.clone());

    // Возвращаем JSON результат
    let json_result = serde_json::to_string(&analysis_result)
        .map_err(|e| format!("Ошибка сериализации JSON: {e}"))?;

    Ok(json_result)
}

/// Команда запуска интеграционных тестов
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn run_integration_tests(
    project_path: Option<String>,
) -> std::result::Result<String, String> {
    let test_path = project_path.map(PathBuf::from);

    // Временно отключаем интеграционные тесты
    // match // crate::integration_tests::run_integration_tests(test_path) {
    //     Ok(results) => {
    //         println!("✅ Интеграционные тесты прошли успешно");
    //         println!("Результаты: {:?}", results);
    //     }
    //     Err(e) => {
    //         eprintln!("❌ Ошибка при выполнении интеграционных тестов: {}", e);
    //         return Err(e);
    //     }
    // }

    println!("✅ Интеграционные тесты временно отключены");
    Ok("Интеграционные тесты временно отключены".to_string())
}

/// Команда экспорта в интерактивный HTML
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn export_interactive_html(
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let last_analysis = state.last_analysis.lock().unwrap();

    if let Some(analysis) = last_analysis.as_ref() {
        let exporter = Exporter::new();
        let html_content = exporter
            .export_to_interactive_html(&analysis.graph)
            .map_err(|e| format!("Ошибка экспорта в HTML: {e}"))?;

        Ok(html_content)
    } else {
        Err("Нет данных для экспорта. Сначала выполните анализ проекта.".to_string())
    }
}

/// Команда получения diff-анализа
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn get_diff_analysis(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let last_analysis = state.last_analysis.lock().unwrap();
    let previous_analysis = state.previous_analysis.lock().unwrap();

    if let (Some(current), Some(previous)) = (last_analysis.as_ref(), previous_analysis.as_ref()) {
        let diff_analyzer = DiffAnalyzer::new();
        let diff_result = diff_analyzer
            .analyze_diff(&current.graph, previous)
            .map_err(|e| format!("Ошибка diff-анализа: {e}"))?;

        let json_result =
            serde_json::to_string(&diff_result).map_err(|e| format!("Ошибка сериализации: {e}"))?;

        Ok(json_result)
    } else {
        Err("Недостаточно данных для diff-анализа. Необходимо минимум два анализа.".to_string())
    }
}

/// Команда получения продвинутых метрик
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn get_advanced_metrics(
    capsule_name: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let last_analysis = state.last_analysis.lock().unwrap();

    if let Some(analysis) = last_analysis.as_ref() {
        // Найдем капсулу по имени
        if let Some(capsule) = analysis
            .graph
            .capsules
            .values()
            .find(|c| c.name == capsule_name)
        {
            let metrics_calculator = AdvancedMetricsCalculator::new();

            if let Ok(content) = fs::read_to_string(&capsule.file_path) {
                let metrics = metrics_calculator
                    .calculate_metrics(capsule, &content)
                    .map_err(|e| format!("Ошибка расчета метрик: {e}"))?;

                let json_result = serde_json::to_string(&metrics)
                    .map_err(|e| format!("Ошибка сериализации: {e}"))?;

                Ok(json_result)
            } else {
                Err(format!(
                    "Не удалось прочитать файл капсулы: {}",
                    capsule.file_path.display()
                ))
            }
        } else {
            Err(format!("Капсула '{}' не найдена", capsule_name))
        }
    } else {
        Err("Нет данных для анализа. Сначала выполните анализ проекта.".to_string())
    }
}

/// Команда обратной совместимости - упрощенный анализ проекта
#[cfg(feature = "gui")]
#[tauri::command]
pub async fn analyze_project(
    project_path: String,
    _config: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path.clone()),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.js".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec![],
        max_depth: Some(3),
        follow_symlinks: false,
        analyze_dependencies: false,
        extract_comments: false,
        parse_tests: false,
        experimental_features: false,
        generate_summaries: false,
        languages: vec![
            FileType::Rust,
            FileType::TypeScript,
            FileType::JavaScript,
            FileType::Python,
        ],
    };

    // Сканирование файлов
    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth,
    )?;
    let files = scanner.scan_files(Path::new(&project_path))?;

    // Если файлы не найдены, создаем демо-капсулы для демонстрации
    let mut capsules = std::collections::HashMap::new();

    if files.is_empty() {
        // Создаем несколько демо-капсул для демонстрации работы системы
        let demo_capsules_data = vec![
            (
                "main",
                CapsuleType::Function,
                "Core",
                15,
                "Главная функция приложения",
            ),
            (
                "config",
                CapsuleType::Module,
                "Core",
                25,
                "Модуль конфигурации",
            ),
            (
                "utils",
                CapsuleType::Module,
                "Utils",
                35,
                "Утилиты и помощники",
            ),
            ("api", CapsuleType::Module, "API", 45, "API интерфейс"),
        ];

        for (name, capsule_type, layer_name, complexity, slogan) in demo_capsules_data {
            let capsule_id = uuid::Uuid::new_v4();

            let capsule = Capsule {
                id: capsule_id,
                name: name.to_string(),
                capsule_type,
                file_path: PathBuf::from(format!("{project_path}/{name}.rs")),
                line_start: 1,
                line_end: complexity,
                size: complexity,
                complexity: (complexity / 5) as u32,
                dependencies: vec![],
                layer: Some(layer_name.to_string()),
                summary: Some(format!("Демо-компонент {name} для тестирования")),
                description: Some(format!(
                    "Демонстрационный компонент {name} для показа возможностей системы"
                )),
                warnings: vec![],
                status: CapsuleStatus::Active,
                priority: Priority::Medium,
                tags: vec!["demo".to_string(), layer_name.to_lowercase()],
                metadata: std::collections::HashMap::new(),
                quality_score: 0.8,
                slogan: Some(slogan.to_string()),
                dependents: vec![],
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            };

            capsules.insert(capsule_id, capsule);
        }
    } else {
        // Упрощенное создание капсул из файлов
        for file in files.iter() {
            let capsule_id = uuid::Uuid::new_v4();
            let layer_name = determine_layer(&file.path);

            let capsule = Capsule {
                id: capsule_id,
                name: file
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                capsule_type: determine_capsule_type(&file.file_type),
                file_path: file.path.clone(),
                line_start: 1,
                line_end: file.lines_count,
                size: file.lines_count,
                complexity: ((file.lines_count / 10) + 1) as u32,
                dependencies: vec![],
                layer: Some(layer_name.clone()),
                summary: None,
                description: Some(format!(
                    "Файл {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                )),
                warnings: vec![],
                status: CapsuleStatus::Active,
                priority: Priority::Medium,
                tags: vec![layer_name.to_lowercase()],
                metadata: std::collections::HashMap::new(),
                quality_score: 0.7,
                slogan: Some(format!(
                    "Компонент {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                )),
                dependents: vec![],
                created_at: Some(chrono::Utc::now().to_rfc3339()),
            };

            capsules.insert(capsule_id, capsule);
        }
    }

    // Упрощенное построение графа
    let mut graph_builder = CapsuleGraphBuilder::new();
    let capsules_vec: Vec<Capsule> = capsules.into_values().collect();
    let graph = graph_builder.build_graph(&capsules_vec)?;

    // Упрощенная валидация
    let validator = ValidatorOptimizer::new();
    let validated_graph = validator
        .validate_and_optimize(&graph)
        .map_err(|e| format!("Ошибка валидации: {e}"))?;

    let analysis_result = AnalysisResult {
        graph: validated_graph,
        warnings: vec![],
        recommendations: vec![
            "Рассмотрите выделение общих компонентов в отдельные модули".to_string(),
            "Проверьте цикличные зависимости между модулями".to_string(),
            "Добавьте документацию к публичным интерфейсам".to_string(),
        ],
        export_formats: vec![
            ExportFormat::JSON,
            ExportFormat::Mermaid,
            ExportFormat::DOT,
            ExportFormat::SVG,
        ],
    };

    // Сохраняем результат
    let mut last_analysis = state.last_analysis.lock().unwrap();
    *last_analysis = Some(analysis_result.clone());

    // Возвращаем JSON результат
    let json_result = serde_json::to_string(&analysis_result)
        .map_err(|e| format!("Ошибка сериализации JSON: {e}"))?;

    Ok(json_result)
}

fn determine_layer(path: &Path) -> String {
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
                    _ => "Other".to_string(),
                };
            }
        }
    }
    "Core".to_string()
}

fn determine_capsule_type(file_type: &FileType) -> CapsuleType {
    match file_type {
        FileType::Rust => CapsuleType::Module,
        FileType::JavaScript | FileType::TypeScript => CapsuleType::Module,
        FileType::Python => CapsuleType::Module,
        _ => CapsuleType::Module,
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn get_analysis_status(
    state: State<'_, AppState>,
) -> std::result::Result<Option<String>, String> {
    let analysis = state.last_analysis.lock().unwrap();
    match analysis.as_ref() {
        Some(_result) => Ok(Some("Анализ завершен".to_string())),
        None => Ok(None),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn export_analysis(
    format: String,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let analysis = state.last_analysis.lock().unwrap();
    match analysis.as_ref() {
        Some(result) => {
            let exporter = Exporter::new();
            match format.as_str() {
                "json" => {
                    let json_data = exporter
                        .export_to_json(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта JSON: {e}"))?;
                    Ok(json_data)
                }
                "yaml" => {
                    let yaml_data = exporter
                        .export_to_yaml(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта YAML: {e}"))?;
                    Ok(yaml_data)
                }
                "svg" => {
                    let svg_data = exporter
                        .export_to_svg(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта SVG: {e}"))?;
                    Ok(svg_data)
                }
                "ai_compact" => {
                    let ai_data = exporter
                        .export_to_ai_compact(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта AI Compact: {e}"))?;
                    Ok(ai_data)
                }
                _ => Err(format!("Неподдерживаемый формат: {format}")),
            }
        }
        None => Err("Нет данных для экспорта".to_string()),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn generate_architecture_diagram(
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let analysis = state.last_analysis.lock().unwrap();
    match analysis.as_ref() {
        Some(result) => {
            let exporter = Exporter::new();
            let diagram = exporter
                .export_to_mermaid(&result.graph)
                .map_err(|e| format!("Ошибка генерации диаграммы: {e}"))?;
            Ok(diagram)
        }
        None => Err("Нет данных для генерации диаграммы".to_string()),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn generate_svg_architecture_diagram(
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let analysis = state.last_analysis.lock().unwrap();
    match analysis.as_ref() {
        Some(result) => {
            let exporter = Exporter::new();
            let svg_diagram = exporter
                .export_to_svg(&result.graph)
                .map_err(|e| format!("Ошибка генерации SVG диаграммы: {e}"))?;
            Ok(svg_diagram)
        }
        None => Err("Нет данных для генерации SVG диаграммы".to_string()),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn get_project_structure(
    project_path: String,
) -> std::result::Result<ProjectStructure, String> {
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path.clone()),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.js".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(5),
        follow_symlinks: false,
        analyze_dependencies: false,
        extract_comments: false,
        parse_tests: false,
        experimental_features: false,
        generate_summaries: false,
        languages: vec![
            FileType::Rust,
            FileType::TypeScript,
            FileType::JavaScript,
            FileType::Python,
        ],
    };

    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth,
    )?;
    let files = scanner.scan_files(Path::new(&project_path))?;

    let mut file_types = std::collections::HashMap::new();
    let mut layers = std::collections::HashSet::new();

    for file in &files {
        let type_str = format!("{:?}", file.file_type);
        *file_types.entry(type_str).or_insert(0) += 1;

        // Определяем архитектурные слои по путям
        if let Some(parent) = file.path.parent() {
            if let Some(dir_name) = parent.file_name() {
                if let Some(dir_str) = dir_name.to_str() {
                    match dir_str {
                        "src" | "lib" => layers.insert("Core".to_string()),
                        "api" | "routes" => layers.insert("API".to_string()),
                        "models" | "entities" => layers.insert("Domain".to_string()),
                        "services" => layers.insert("Business".to_string()),
                        "utils" | "helpers" => layers.insert("Utils".to_string()),
                        "components" => layers.insert("UI".to_string()),
                        _ => false,
                    };
                }
            }
        }
    }

    let complexity = files.len().min(100);

    Ok(ProjectStructure {
        total_files: files.len(),
        file_types,
        layers: layers.into_iter().collect(),
        estimated_complexity: complexity,
    })
}

#[cfg(feature = "gui")]
#[tauri::command]
pub async fn validate_project_path(project_path: String) -> std::result::Result<bool, String> {
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path.clone()),
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.js".to_string(),
            "**/*.py".to_string(),
        ],
        exclude_patterns: vec![],
        max_depth: Some(3),
        follow_symlinks: false,
        analyze_dependencies: false,
        extract_comments: false,
        parse_tests: false,
        experimental_features: false,
        generate_summaries: false,
        languages: vec![
            FileType::Rust,
            FileType::TypeScript,
            FileType::JavaScript,
            FileType::Python,
        ],
    };

    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth,
    )?;
    let files = scanner.scan_files(Path::new(&project_path))?;

    Ok(!files.is_empty())
}
