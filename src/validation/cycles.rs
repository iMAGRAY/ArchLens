use crate::types::*;
use crate::types::Result;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub struct CycleValidator;

impl CycleValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        let cycles = self.find_dependency_cycles(graph);
        
        for cycle in cycles {
            if cycle.len() > 1 {
                let cycle_names: Vec<String> = cycle.iter()
                    .filter_map(|id| graph.capsules.get(id).map(|c| c.name.clone()))
                    .collect();
                
                warnings.push(AnalysisWarning {
                    level: Priority::High,
                    message: format!("Circular dependency detected: {}", cycle_names.join(" -> ")),
                    category: "cycles".to_string(),
                    capsule_id: cycle.first().copied(),
                    suggestion: Some("Break circular dependencies using interfaces".to_string()),
                });
            }
        }
        
        Ok(())
    }
    
    fn find_dependency_cycles(&self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        
        for capsule_id in graph.capsules.keys() {
            if !visited.contains(capsule_id) {
                let mut rec_stack = HashSet::new();
                let mut path = Vec::new();
                
                if self.has_cycle_dfs(*capsule_id, graph, &mut visited, &mut rec_stack, &mut path) {
                    cycles.push(path);
                }
            }
        }
        
        cycles
    }
    
    fn has_cycle_dfs(
        &self,
        capsule_id: Uuid,
        graph: &CapsuleGraph,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> bool {
        visited.insert(capsule_id);
        rec_stack.insert(capsule_id);
        path.push(capsule_id);
        
        for relation in &graph.relations {
            if relation.from_id == capsule_id {
                let next_id = relation.to_id;
                
                if !visited.contains(&next_id) {
                    if self.has_cycle_dfs(next_id, graph, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(&next_id) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(&capsule_id);
        path.pop();
        false
    }
} 