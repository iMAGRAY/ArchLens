use crate::types::Result;
use crate::types::*;

/// Complexity validator
#[derive(Debug)]
pub struct ComplexityValidator {
    max_threshold: u32,
}

impl ComplexityValidator {
    pub fn new() -> Self {
        Self { max_threshold: 15 }
    }

    pub fn validate(
        &self,
        graph: &CapsuleGraph,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        // System complexity check
        if graph.metrics.complexity_average > self.max_threshold as f32 {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!(
                    "High system complexity: {:.2}. Consider breaking into simpler components.",
                    graph.metrics.complexity_average
                ),
                category: "complexity".to_string(),
                capsule_id: None,
                suggestion: Some("Extract common functionality into separate modules".to_string()),
            });
        }

        // Individual capsule complexity check
        for capsule in graph.capsules.values() {
            if capsule.complexity > self.max_threshold {
                warnings.push(AnalysisWarning {
                    level: Priority::Medium,
                    message: format!(
                        "Component '{}' has high complexity: {}",
                        capsule.name, capsule.complexity
                    ),
                    category: "complexity".to_string(),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Consider breaking into smaller functions".to_string()),
                });
            }
        }

        Ok(())
    }
}
