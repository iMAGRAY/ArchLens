use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::State;
use std::sync::Arc;
use std::sync::Mutex;
use crate::core::*;
use crate::file_scanner::FileScanner;
use crate::exporter::Exporter;
use crate::capsule_graph_builder::CapsuleGraphBuilder;
use crate::capsule_enricher::CapsuleEnricher;
use crate::validator_optimizer::ValidatorOptimizer;

// Состояние приложения
pub struct AppState {
    pub last_analysis: Arc<Mutex<Option<AnalysisResult>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_analysis: Arc::new(Mutex::new(None)),
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

#[tauri::command]
pub async fn analyze_project(
    project_path: String,
    _config: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let config = AnalysisConfig {
        project_path: PathBuf::from(project_path.clone()),
        include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string(), "**/*.py".to_string()],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string(), "**/.git/**".to_string()],
        max_depth: Some(10),
        analyze_dependencies: true,
        extract_comments: true,
        generate_summaries: true,
        languages: vec![FileType::Rust, FileType::TypeScript, FileType::JavaScript, FileType::Python],
    };

    let scanner = FileScanner::new(
        config.include_patterns.clone(),
        config.exclude_patterns.clone(),
        config.max_depth.map(|d| d as usize),
    ).map_err(|e| format!("Ошибка создания сканера: {e}"))?;

    let files = scanner.scan_project(&config.project_path)
        .map_err(|e| format!("Ошибка сканирования: {e}"))?;

    // Если файлы не найдены, создаем демо-капсулы для демонстрации
    let mut capsules = std::collections::HashMap::new();
    let mut layers = std::collections::HashMap::new();
    
    if files.is_empty() {
        // Создаем несколько демо-капсул для демонстрации работы системы
        let demo_capsules_data = vec![
            ("main", CapsuleType::Function, "Core", 15, "Главная функция приложения"),
            ("config", CapsuleType::Module, "Core", 25, "Модуль конфигурации"),
            ("utils", CapsuleType::Module, "Utils", 35, "Утилиты и помощники"),
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
                complexity: (complexity / 5) as u32,
                priority: Priority::Medium,
                status: CapsuleStatus::Active,
                layer: Some(layer_name.to_string()),
                slogan: Some(slogan.to_string()),
                summary: Some(format!("Демо-компонент {name} для тестирования")),
                warnings: vec![],
                dependencies: vec![],
                dependents: vec![],
                metadata: std::collections::HashMap::new(),
                created_at: chrono::Utc::now(),
            };
            
            capsules.insert(capsule_id, capsule);
            layers.entry(layer_name.to_string()).or_insert_with(Vec::new).push(capsule_id);
        }
    } else {
        // Создаем капсулы из найденных файлов
        for file in files.iter() {
            let capsule_id = uuid::Uuid::new_v4();
            let layer_name = determine_layer(&file.path);
            
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
                complexity: ((file.lines_count / 10) + 1) as u32,
                priority: Priority::Medium,
                status: CapsuleStatus::Active,
                layer: Some(layer_name.clone()),
                slogan: Some(format!("Компонент {}", file.path.file_name().unwrap_or_default().to_string_lossy())),
                summary: None,
                warnings: vec![],
                dependencies: vec![],
                dependents: vec![],
                metadata: std::collections::HashMap::new(),
                created_at: chrono::Utc::now(),
            };
            
            capsules.insert(capsule_id, capsule);
            layers.entry(layer_name).or_insert_with(Vec::new).push(capsule_id);
        }
    }

    let graph_builder = CapsuleGraphBuilder::new();
    let capsules_vec: Vec<Capsule> = capsules.into_values().collect();
    let graph = graph_builder.build_graph(&capsules_vec)
        .map_err(|e| format!("Ошибка построения графа: {e}"))?;

    let enricher = CapsuleEnricher::new();
    let enriched_graph = enricher.enrich_graph(&graph)
        .map_err(|e| format!("Ошибка обогащения графа: {e}"))?;

    let validator = ValidatorOptimizer::new();
    let validated_graph = validator.validate_and_optimize(&enriched_graph)
        .map_err(|e| format!("Ошибка валидации: {e}"))?;

    let analysis_result = AnalysisResult {
        graph: validated_graph,
        warnings: vec![],
        recommendations: vec![
            "Рассмотрите выделение общих компонентов в отдельные модули".to_string(),
            "Проверьте цикличные зависимости между модулями".to_string(),
            "Добавьте документацию к публичным интерфейсам".to_string(),
        ],
        export_formats: vec![ExportFormat::Json, ExportFormat::Mermaid, ExportFormat::DOT],
    };

    // Сохраняем результат
    let mut last_analysis = state.last_analysis.lock().unwrap();
    *last_analysis = Some(analysis_result.clone());

    // Возвращаем JSON результат вместо форматированной строки
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
                    let json_data = exporter.export_to_json(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта JSON: {e}"))?;
                    Ok(json_data)
                }
                "yaml" => {
                    let yaml_data = exporter.export_to_yaml(&result.graph)
                        .map_err(|e| format!("Ошибка экспорта YAML: {e}"))?;
                    Ok(yaml_data)
                }
                _ => Err(format!("Неподдерживаемый формат: {format}"))
            }
        }
        None => Err("Нет данных для экспорта".to_string()),
    }
}

#[tauri::command]
pub async fn generate_architecture_diagram(
    state: State<'_, AppState>,
) -> std::result::Result<String, String> {
    let analysis = state.last_analysis.lock().unwrap();
    match analysis.as_ref() {
        Some(result) => {
            let exporter = Exporter::new();
            let diagram = exporter.export_to_mermaid(&result.graph)
                .map_err(|e| format!("Ошибка генерации диаграммы: {e}"))?;
            Ok(diagram)
        }
        None => Err("Нет данных для генерации диаграммы".to_string()),
    }
}

#[tauri::command]
pub async fn get_project_structure(
    project_path: String,
) -> std::result::Result<ProjectStructure, String> {
    let scanner = FileScanner::new(
        vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string(), "**/*.py".to_string()],
        vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        Some(5),
    ).map_err(|e| format!("Ошибка создания сканера: {e}"))?;

    let files = scanner.scan_project(std::path::Path::new(&project_path))
        .map_err(|e| format!("Ошибка сканирования: {e}"))?;

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

#[tauri::command]
pub async fn validate_project_path(
    project_path: String,
) -> std::result::Result<bool, String> {
    let scanner = FileScanner::new(
        vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string(), "**/*.py".to_string()],
        vec![],
        Some(3),
    ).map_err(|e| format!("Ошибка создания сканера: {e}"))?;

    let files = scanner.scan_project(std::path::Path::new(&project_path))
        .map_err(|e| format!("Ошибка сканирования: {e}"))?;

    Ok(!files.is_empty())
} 