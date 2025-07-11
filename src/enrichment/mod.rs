// Модуль для обогащения капсул - организует все подмодули анализа

pub mod semantic_analysis;
pub mod pattern_detection;
pub mod quality_metrics;
pub mod code_smells;
pub mod dependency_analysis;
pub mod content_analysis;

// Переэкспорт основных типов для удобства
pub use semantic_analysis::*;
pub use pattern_detection::*;
pub use quality_metrics::*;
pub use code_smells::*;
pub use dependency_analysis::*;
pub use content_analysis::*; 