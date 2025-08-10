use crate::types::Result;
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct CouplingValidator {
    threshold: f32,
}

impl CouplingValidator {
    pub fn new() -> Self {
        Self { threshold: 0.7 }
    }

    pub fn validate(
        &self,
        graph: &CapsuleGraph,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        if graph.metrics.coupling_index > self.threshold {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!("High coupling: {:.2}", graph.metrics.coupling_index),
                category: "coupling".to_string(),
                capsule_id: None,
                suggestion: Some("Use dependency inversion and interfaces".to_string()),
            });
        }

        // Analyze nodes with high coupling
        let mut coupling_counts: HashMap<Uuid, usize> = HashMap::new();
        for relation in &graph.relations {
            *coupling_counts.entry(relation.from_id).or_insert(0) += 1;
            *coupling_counts.entry(relation.to_id).or_insert(0) += 1;
        }

        for (capsule_id, count) in coupling_counts {
            if count > 10 {
                if let Some(capsule) = graph.capsules.get(&capsule_id) {
                    warnings.push(AnalysisWarning {
                        level: Priority::Medium,
                        message: format!(
                            "Component '{}' has too many connections: {}",
                            capsule.name, count
                        ),
                        category: "coupling".to_string(),
                        capsule_id: Some(capsule_id),
                        suggestion: Some("Consider applying Facade pattern".to_string()),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Default for CouplingValidator {
    fn default() -> Self {
        Self::new()
    }
}
