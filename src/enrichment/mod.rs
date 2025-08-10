// Модуль для обогащения капсул - организует все подмодули анализа

pub mod code_smells;
pub mod content_analysis;
pub mod dependency_analysis;
pub mod pattern_detection;
pub mod quality_metrics;
pub mod semantic_analysis;

// Новые рефакторенные модули
pub mod enricher_core;
pub mod quality_analyzer;
pub mod semantic_analyzer;

// Переэкспорт основных типов для удобства
pub use code_smells::*;
pub use content_analysis::*;
pub use dependency_analysis::*;
pub use pattern_detection::*;
pub use quality_metrics::*;
pub use semantic_analysis::*;

// Переэкспорт новых модулей (избегаем конфликтов имен)
pub use enricher_core::{CapsuleEnricher, EnrichmentResult};
pub use quality_analyzer::{QualityAnalyzer, QualityAssessment};
pub use semantic_analyzer::{SemanticAnalyzer, SemanticEnricher};
