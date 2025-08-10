// Cycle detection for dependency graphs
use crate::types::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Cycle detector for dependency analysis
#[derive(Debug)]
pub struct CycleDetector {
    visited: HashSet<Uuid>,
    recursion_stack: HashSet<Uuid>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            recursion_stack: HashSet::new(),
        }
    }

    /// Finds cycles in the dependency graph
    pub fn find_cycles(&mut self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        self.visited.clear();
        self.recursion_stack.clear();

        for capsule_id in graph.capsules.keys() {
            if !self.visited.contains(capsule_id) {
                if let Some(cycle) = self.dfs_cycle_detection(*capsule_id, graph, &mut Vec::new()) {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &mut self,
        capsule_id: Uuid,
        graph: &CapsuleGraph,
        path: &mut Vec<Uuid>,
    ) -> Option<Vec<Uuid>> {
        self.visited.insert(capsule_id);
        self.recursion_stack.insert(capsule_id);
        path.push(capsule_id);

        // Check all dependencies of current capsule
        if let Some(capsule) = graph.capsules.get(&capsule_id) {
            for &dependency_id in &capsule.dependencies {
                if !self.visited.contains(&dependency_id) {
                    if let Some(cycle) = self.dfs_cycle_detection(dependency_id, graph, path) {
                        return Some(cycle);
                    }
                } else if self.recursion_stack.contains(&dependency_id) {
                    // Cycle found
                    if let Some(cycle_start_pos) = path.iter().position(|&id| id == dependency_id) {
                        return Some(path[cycle_start_pos..].to_vec());
                    } else {
                        // If we can't find cycle start, return entire path
                        return Some(path.clone());
                    }
                }
            }
        }

        path.pop();
        self.recursion_stack.remove(&capsule_id);
        None
    }

    /// Add cycle warnings to graph
    pub fn add_cycle_warnings(&self, graph: &mut CapsuleGraph, cycles: &[Vec<Uuid>]) -> Result<()> {
        for cycle in cycles {
            for &capsule_id in cycle {
                if let Some(capsule) = graph.capsules.get_mut(&capsule_id) {
                    capsule.warnings.push(AnalysisWarning {
                        message: format!("Circular dependency detected in cycle of {} capsules", cycle.len()),
                        level: Priority::High,
                        category: "architecture".to_string(),
                        capsule_id: None,
                        suggestion: Some("Break the circular dependency through abstraction or dependency inversion".to_string()),
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if graph has any cycles
    pub fn has_cycles(&mut self, graph: &CapsuleGraph) -> bool {
        !self.find_cycles(graph).is_empty()
    }

    /// Get strongly connected components
    pub fn get_strongly_connected_components(&mut self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();

        // First pass: collect finish times
        let mut finish_order = Vec::new();
        for &capsule_id in graph.capsules.keys() {
            if !visited.contains(&capsule_id) {
                self.dfs_finish_time(capsule_id, graph, &mut visited, &mut finish_order);
            }
        }

        // Second pass: find components in reverse finish order
        visited.clear();
        finish_order.reverse();

        for &capsule_id in &finish_order {
            if !visited.contains(&capsule_id) {
                let mut component = Vec::new();
                self.dfs_component(capsule_id, graph, &mut visited, &mut component);
                if component.len() > 1 {
                    components.push(component);
                }
            }
        }

        components
    }

    /// DFS for finish time calculation
    fn dfs_finish_time(
        &self,
        capsule_id: Uuid,
        graph: &CapsuleGraph,
        visited: &mut HashSet<Uuid>,
        finish_order: &mut Vec<Uuid>,
    ) {
        visited.insert(capsule_id);

        if let Some(capsule) = graph.capsules.get(&capsule_id) {
            for &dependency_id in &capsule.dependencies {
                if !visited.contains(&dependency_id) {
                    self.dfs_finish_time(dependency_id, graph, visited, finish_order);
                }
            }
        }

        finish_order.push(capsule_id);
    }

    /// DFS for component collection
    fn dfs_component(
        &self,
        capsule_id: Uuid,
        graph: &CapsuleGraph,
        visited: &mut HashSet<Uuid>,
        component: &mut Vec<Uuid>,
    ) {
        visited.insert(capsule_id);
        component.push(capsule_id);

        // Follow reverse dependencies (dependents)
        if let Some(capsule) = graph.capsules.get(&capsule_id) {
            for &dependent_id in &capsule.dependents {
                if !visited.contains(&dependent_id) {
                    self.dfs_component(dependent_id, graph, visited, component);
                }
            }
        }
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}
