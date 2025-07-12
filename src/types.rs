use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fmt;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapsuleStatus {
    Pending,
    Active,
    Deprecated,
    Archived,
    Hidden,
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

/// Основная структура компонента (капсулы)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capsule {
    pub id: Uuid,
    pub name: String,
    pub capsule_type: CapsuleType,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub size: usize,
    pub complexity: u32,
    pub dependencies: Vec<Uuid>,
    pub layer: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub warnings: Vec<AnalysisWarning>,
    pub status: CapsuleStatus,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub quality_score: f64,
    pub slogan: Option<String>,
    pub dependents: Vec<Uuid>,
    pub created_at: Option<String>,
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
    pub previous_analysis: Option<Box<ComparisonSnapshot>>, // Для дифф-анализа
}

/// Снимок предыдущего анализа для сравнения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSnapshot {
    pub metrics: GraphMetrics,
    pub total_capsules: usize,
    pub total_relations: usize,
    pub max_complexity: u32,
    pub max_complexity_module: String,
    pub orphan_count: usize,
    pub cycle_count: usize,
    pub analyzed_at: DateTime<Utc>,
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
    pub message: String,
    pub level: Priority,
    pub category: String,
    pub capsule_id: Option<Uuid>,
    pub suggestion: Option<String>,
}

/// Форматы экспорта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    YAML,
    Mermaid,
    DOT,
    GraphML,
    SVG,
    InteractiveHTML,
    ChainOfThought,
    LLMPrompt,
    AICompact,
}

/// Конфигурация анализа
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub project_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub analyze_dependencies: bool,
    pub extract_comments: bool,
    pub parse_tests: bool,
    pub experimental_features: bool,
    pub generate_summaries: bool,
    pub languages: Vec<FileType>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        AnalysisConfig {
            project_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            include_patterns: vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string()],
            exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
            max_depth: Some(10),
            follow_symlinks: false,
            analyze_dependencies: true,
            extract_comments: true,
            parse_tests: false,
            experimental_features: false,
            generate_summaries: true,
            languages: vec![FileType::Rust, FileType::TypeScript, FileType::JavaScript],
        }
    }
}

/// Типы ошибок анализа
#[derive(Debug, Clone)]
pub enum AnalysisError {
    IoError(String),
    ParsingError(String),
    RegexError(String),
    GenericError(String),
    Parse(String),
    Io(String),  // Изменяю на String для Clone
}

impl From<regex::Error> for AnalysisError {
    fn from(err: regex::Error) -> Self {
        AnalysisError::RegexError(err.to_string())
    }
}

impl From<String> for AnalysisError {
    fn from(err: String) -> Self {
        AnalysisError::GenericError(err)
    }
}

impl From<&str> for AnalysisError {
    fn from(s: &str) -> Self {
        AnalysisError::GenericError(s.to_string())
    }
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::IoError(msg) => write!(f, "IO Error: {}", msg),
            AnalysisError::ParsingError(msg) => write!(f, "Parsing Error: {}", msg),
            AnalysisError::RegexError(msg) => write!(f, "Regex Error: {}", msg),
            AnalysisError::GenericError(msg) => write!(f, "Generic Error: {}", msg),
            AnalysisError::Parse(msg) => write!(f, "Parse Error: {}", msg),
            AnalysisError::Io(msg) => write!(f, "IO Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for AnalysisError {
    fn from(err: std::io::Error) -> Self {
        AnalysisError::IoError(err.to_string())
    }
}

/// Наш собственный Result тип для всего проекта
pub type Result<T> = std::result::Result<T, AnalysisError>;

/// Результат diff-анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffAnalysis {
    pub changes: Vec<ArchitectureChange>,
    pub metrics_diff: MetricsDiff,
    pub quality_trend: QualityTrend,
    pub recommendations: Vec<String>,
    pub summary: String,
}

/// Тип изменения в архитектуре
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureChange {
    pub change_type: ChangeType,
    pub component: String,
    pub description: String,
    pub impact: ChangeImpact,
    pub related_components: Vec<String>,
}

/// Типы изменений
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Moved,
    Renamed,
    ComplexityIncrease,
    ComplexityDecrease,
    NewDependency,
    RemovedDependency,
}

/// Влияние изменения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeImpact {
    Breaking,
    Major,
    Minor,
    Refactoring,
    Performance,
    Quality,
}

/// Разница метрик
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDiff {
    pub complexity_delta: f32,
    pub coupling_delta: f32,
    pub cohesion_delta: f32,
    pub component_count_delta: i32,
    pub relation_count_delta: i32,
    pub new_warnings: usize,
    pub resolved_warnings: usize,
}

/// Тренд качества
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTrend {
    Improving,
    Degrading,
    Stable,
    Mixed,
} 