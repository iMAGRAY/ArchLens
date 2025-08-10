use crate::types::Result;
use crate::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LayerValidator;

impl LayerValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        graph: &CapsuleGraph,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        let hierarchy = self.get_layer_hierarchy();

        for relation in &graph.relations {
            if let (Some(from_capsule), Some(to_capsule)) = (
                graph.capsules.get(&relation.from_id),
                graph.capsules.get(&relation.to_id),
            ) {
                if let (Some(from_layer), Some(to_layer)) = (&from_capsule.layer, &to_capsule.layer)
                {
                    if self.violates_layer_hierarchy(from_layer, to_layer, &hierarchy) {
                        warnings.push(AnalysisWarning {
                            level: Priority::Medium,
                            message: format!(
                                "Layer violation: {} -> {} (from {} to {})",
                                from_capsule.name, to_capsule.name, from_layer, to_layer
                            ),
                            category: "layers".to_string(),
                            capsule_id: Some(from_capsule.id),
                            suggestion: Some("Respect architectural layers".to_string()),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn get_layer_hierarchy(&self) -> HashMap<String, usize> {
        let mut hierarchy = HashMap::new();
        hierarchy.insert("UI".to_string(), 0);
        hierarchy.insert("API".to_string(), 1);
        hierarchy.insert("Business".to_string(), 2);
        hierarchy.insert("Data".to_string(), 3);
        hierarchy.insert("Core".to_string(), 4);
        hierarchy
    }

    fn violates_layer_hierarchy(
        &self,
        from_layer: &str,
        to_layer: &str,
        hierarchy: &HashMap<String, usize>,
    ) -> bool {
        if let (Some(&from_level), Some(&to_level)) =
            (hierarchy.get(from_layer), hierarchy.get(to_layer))
        {
            from_level > to_level
        } else {
            false
        }
    }
}
