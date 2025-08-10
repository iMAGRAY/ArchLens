use crate::types::Result;
use crate::types::*;

#[derive(Debug)]
pub struct NamingValidator;

impl NamingValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        graph: &CapsuleGraph,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        for capsule in graph.capsules.values() {
            // Check for generic names
            if self.is_generic_name(&capsule.name) {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: format!("Generic name: {}", capsule.name),
                    category: "naming".to_string(),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Use more descriptive names".to_string()),
                });
            }

            // Check for inconsistent naming
            if self.has_inconsistent_naming(&capsule.name) {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: format!("Inconsistent naming: {}", capsule.name),
                    category: "naming".to_string(),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Follow consistent naming conventions".to_string()),
                });
            }
        }

        Ok(())
    }

    fn is_generic_name(&self, name: &str) -> bool {
        let generic_names = [
            "data", "info", "item", "object", "thing", "stuff", "temp", "test",
        ];
        generic_names.contains(&name.to_lowercase().as_str())
    }

    fn has_inconsistent_naming(&self, name: &str) -> bool {
        // Simple check for mixed case inconsistency
        name.chars().any(|c| c.is_uppercase()) && name.chars().any(|c| c == '_')
    }
}

impl Default for NamingValidator {
    fn default() -> Self { Self::new() }
}
