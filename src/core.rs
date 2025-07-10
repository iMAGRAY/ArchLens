use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Основные типы файлов для анализа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Java,
    Go,
    Cpp,
    C,
    Other(String),
}

/// Тип капсулы (структурной единицы)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum CapsuleType {
    Module,
    Struct,
    Enum,
    Function,
    Method,
    Interface,
    Class,
    Variable,
    Constant,
    Import,
    Export,
    Other,
}

/// Уровень важности/приоритета
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Статус капсулы
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum CapsuleStatus {
    Active,
    Deprecated,
    Experimental,
    Internal,
    Public,
    Unstable,
}

/// Метаданные файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub lines_count: usize,
    pub last_modified: DateTime<Utc>,
    pub layer: Option<String>,      // архитектурный слой (domain, infrastructure, etc.)
    pub slogan: Option<String>,     // краткое описание назначения
    pub status: CapsuleStatus,
    pub dependencies: Vec<PathBuf>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

/// Основная структурная единица - капсула
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capsule {
    pub id: Uuid,
    pub name: String,
    pub capsule_type: CapsuleType,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub complexity: u32,
    pub priority: Priority,
    pub status: CapsuleStatus,
    pub layer: Option<String>,
    pub slogan: Option<String>,
    pub summary: Option<String>,
    pub warnings: Vec<String>,
    pub dependencies: Vec<Uuid>,    // ID других капсул
    pub dependents: Vec<Uuid>,      // ID капсул, которые зависят от этой
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

/// Связь между капсулами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleRelation {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relation_type: RelationType,
    pub strength: f32,             // сила связи 0.0-1.0
    pub description: Option<String>,
}

/// Типы связей между капсулами
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    Depends,       // зависимость
    Uses,          // использование
    Implements,    // реализация
    Extends,       // наследование
    Aggregates,    // агрегация
    Composes,      // композиция
    Calls,         // вызов
    References,    // ссылка
}

/// Граф капсул
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleGraph {
    pub capsules: HashMap<Uuid, Capsule>,
    pub relations: Vec<CapsuleRelation>,
    pub layers: HashMap<String, Vec<Uuid>>,
    pub metrics: GraphMetrics,
    pub created_at: DateTime<Utc>,
}

/// Метрики графа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_capsules: usize,
    pub total_relations: usize,
    pub complexity_average: f32,
    pub coupling_index: f32,
    pub cohesion_index: f32,
    pub cyclomatic_complexity: u32,
    pub depth_levels: u32,
}

/// Результат анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub graph: CapsuleGraph,
    pub warnings: Vec<AnalysisWarning>,
    pub recommendations: Vec<String>,
    pub export_formats: Vec<ExportFormat>,
}

/// Предупреждение анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWarning {
    pub level: Priority,
    pub message: String,
    pub capsule_id: Option<Uuid>,
    pub suggestion: Option<String>,
}

/// Форматы экспорта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Yaml,
    Json,
    GraphML,
    DOT,
    Mermaid,
    ChainOfThought,
    LLMPrompt,
}

/// Настройки анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub project_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<u32>,
    pub analyze_dependencies: bool,
    pub extract_comments: bool,
    pub generate_summaries: bool,
    pub languages: Vec<FileType>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from("."),
            include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string()],
            exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
            max_depth: Some(10),
            analyze_dependencies: true,
            extract_comments: true,
            generate_summaries: true,
            languages: vec![FileType::Rust, FileType::TypeScript, FileType::JavaScript],
        }
    }
}

/// Ошибки анализа
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("IO ошибка: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Ошибка парсинга: {0}")]
    Parse(String),
    
    #[error("Неподдерживаемый тип файла: {0:?}")]
    UnsupportedFileType(FileType),
    
    #[error("Конфигурация невалидна: {0}")]
    InvalidConfig(String),
    
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

impl From<regex::Error> for AnalysisError {
    fn from(err: regex::Error) -> Self {
        AnalysisError::Parse(err.to_string())
    }
}

impl From<String> for AnalysisError {
    fn from(err: String) -> Self {
        AnalysisError::Internal(err)
    }
}

pub type Result<T> = std::result::Result<T, AnalysisError>; 