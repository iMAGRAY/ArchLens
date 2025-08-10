// Семантический обогатитель капсул с анализом связей и метаданных
// Рефакторенная версия - использует модульную архитектуру

use crate::enrichment::quality_analyzer::QualityCategory;
use crate::enrichment::{CapsuleEnricher as CoreEnricher, QualityAnalyzer, SemanticEnricher};
use crate::types::*;
// use std::collections::HashMap;
// use uuid::Uuid;

/// Главный обогатитель капсул - композитный класс, использующий специализированные анализаторы
pub struct CapsuleEnricher {
    core_enricher: CoreEnricher,
    semantic_enricher: SemanticEnricher,
    quality_analyzer: QualityAnalyzer,
}

// Переэкспорт типов из модулей для обратной совместимости
pub use crate::enrichment::enricher_core::{CodeSmell, CodeSmellType, QualityMetrics};
pub use crate::enrichment::{
    ArchitecturalPattern, EnrichmentResult, PatternType, SemanticLink, SemanticLinkType,
};

impl CapsuleEnricher {
    /// Create new enricher with all specialized analyzers
    pub fn new() -> Self {
        Self {
            core_enricher: CoreEnricher::new(),
            semantic_enricher: SemanticEnricher::new(),
            quality_analyzer: QualityAnalyzer::new(),
        }
    }

    /// Main enrichment function - delegates to core enricher
    pub fn enrich_graph(&self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        self.core_enricher.enrich_graph(graph)
    }

    /// Perform comprehensive semantic analysis using specialized analyzers
    pub fn perform_semantic_analysis(
        &self,
        capsule: &Capsule,
        content: &str,
    ) -> Result<EnrichmentResult> {
        // Use semantic enricher for detailed analysis
        let mut result = self
            .semantic_enricher
            .perform_semantic_analysis(capsule, content)?;

        // Enhance with quality analysis
        if let Ok(quality_assessment) = self.quality_analyzer.analyze_quality(capsule, content) {
            // Update quality metrics with detailed assessment
            result.quality_metrics = crate::enrichment::enricher_core::QualityMetrics {
                maintainability_index: quality_assessment.maintainability_index,
                cognitive_complexity: quality_assessment.complexity_score as u32,
                technical_debt_ratio: (100.0 - quality_assessment.technical_debt_score) / 100.0,
                test_coverage_estimate: quality_assessment.test_coverage_score / 100.0,
                documentation_completeness: quality_assessment.documentation_score / 100.0,
            };

            // Add quality-based code smells
            for recommendation in quality_assessment.recommendations {
                let smell_type = match recommendation.category {
                    QualityCategory::Complexity => {
                        crate::enrichment::enricher_core::CodeSmellType::LongMethod
                    }
                    QualityCategory::Documentation => {
                        crate::enrichment::enricher_core::CodeSmellType::DeadCode
                    }
                    QualityCategory::Testing => {
                        crate::enrichment::enricher_core::CodeSmellType::DeadCode
                    }
                    QualityCategory::Maintainability => {
                        crate::enrichment::enricher_core::CodeSmellType::GodObject
                    }
                    QualityCategory::Architecture => {
                        crate::enrichment::enricher_core::CodeSmellType::GodObject
                    }
                    _ => crate::enrichment::enricher_core::CodeSmellType::DeadCode,
                };

                result
                    .code_smells
                    .push(crate::enrichment::enricher_core::CodeSmell {
                        smell_type,
                        severity: recommendation.priority,
                        description: recommendation.description,
                        suggestion: recommendation.suggestion,
                    });
            }
        }

        Ok(result)
    }

    /// Calculate quality index for a capsule
    pub fn calculate_quality_index(&self, capsule: &Capsule) -> f64 {
        if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
            if let Ok(assessment) = self.quality_analyzer.analyze_quality(capsule, &content) {
                return assessment.overall_score as f64;
            }
        }

        // Fallback calculation
        let mut score: f64 = 50.0;

        if capsule.complexity <= 10 {
            score += 20.0;
        } else if capsule.complexity <= 20 {
            score += 10.0;
        } else {
            score -= 10.0;
        }

        if capsule.size <= 100 {
            score += 10.0;
        } else if capsule.size > 1000 {
            score -= 15.0;
        }

        score.clamp(0.0, 100.0)
    }
}

impl Default for CapsuleEnricher {
    fn default() -> Self {
        Self::new()
    }
}
