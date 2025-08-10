use crate::types::Result;
use crate::types::*;

#[derive(Debug)]
pub struct CohesionValidator {
    threshold: f32,
}

impl CohesionValidator {
    pub fn new() -> Self {
        Self { threshold: 0.3 }
    }

    pub fn validate(
        &self,
        graph: &CapsuleGraph,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        if graph.metrics.cohesion_index < self.threshold {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!("Low cohesion: {:.2}", graph.metrics.cohesion_index),
                category: "cohesion".to_string(),
                capsule_id: None,
                suggestion: Some("Group related functionality into modules".to_string()),
            });
        }
        Ok(())
    }
}

impl Default for CohesionValidator {
    fn default() -> Self { Self::new() }
}
