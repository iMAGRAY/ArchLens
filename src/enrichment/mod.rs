// Модуль для обогащения капсул - организует все подмодули анализа

pub mod semantic_analysis;
pub mod pattern_detection;
pub mod quality_metrics;
pub mod code_smells;
pub mod dependency_analysis;
pub mod content_analysis;

// Новые рефакторенные модули
pub mod enricher_core;
pub mod semantic_analyzer;
pub mod quality_analyzer;

// Переэкспорт основных типов для удобства
pub use semantic_analysis::*;
pub use pattern_detection::*;
pub use quality_metrics::*;
pub use code_smells::*;
pub use dependency_analysis::*;
pub use content_analysis::*;

// Переэкспорт новых модулей (избегаем конфликтов имен)
pub use enricher_core::{CapsuleEnricher, EnrichmentResult};
pub use semantic_analyzer::{SemanticEnricher, SemanticAnalyzer};
pub use quality_analyzer::{QualityAnalyzer, QualityAssessment}; 