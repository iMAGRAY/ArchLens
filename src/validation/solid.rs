use crate::types::Result;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct SolidAnalyzer {
    pub principle: SolidPrinciple,
    pub detection_patterns: Vec<String>,
    pub violation_threshold: f32,
}

#[derive(Debug, Clone)]
pub enum SolidPrinciple {
    SingleResponsibility,
    OpenClosed,
    LiskovSubstitution,
    InterfaceSegregation,
    DependencyInversion,
}

impl SolidAnalyzer {
    pub fn new(principle: SolidPrinciple) -> Self {
        Self {
            principle,
            detection_patterns: vec![],
            violation_threshold: 0.5,
        }
    }

    pub fn analyze(&self, capsule: &Capsule) -> Result<Vec<AnalysisWarning>> {
        let mut warnings = Vec::new();

        if let SolidPrinciple::SingleResponsibility = self.principle {
            if capsule.complexity > 15 {
                warnings.push(AnalysisWarning {
                    level: Priority::Medium,
                    message: format!("Possible SRP violation in {}", capsule.name),
                    category: "solid".to_string(),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Consider splitting responsibilities".to_string()),
                });
            }
        }

        Ok(warnings)
    }
}
