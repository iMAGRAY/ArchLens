// Metrics calculation for capsule graphs
use crate::types::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Calculates various metrics for capsule graphs
pub struct MetricsCalculator {
    // Configuration for metric calculation
    pub complexity_weights: ComplexityWeights,
}

/// Weights for complexity calculations
#[derive(Debug, Clone)]
pub struct ComplexityWeights {
    pub cyclomatic_weight: f32,
    pub cognitive_weight: f32,
    pub coupling_weight: f32,
    pub cohesion_weight: f32,
}

impl MetricsCalculator {
    pub fn new() -> Self {
        Self {
            complexity_weights: ComplexityWeights::default(),
        }
    }

    /// Calculate advanced metrics for the graph
    pub fn calculate_advanced_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> Result<GraphMetrics> {
        let total_capsules = capsules.len();
        let total_relations = relations.len();

        // Average complexity
        let complexity_sum: u32 = capsules.values().map(|c| c.complexity).sum();
        let complexity_average = if total_capsules > 0 {
            complexity_sum as f32 / total_capsules as f32
        } else {
            0.0
        };

        // Coupling index - considers relation strength
        let coupling_sum: f32 = relations.iter().map(|r| r.strength).sum();
        let coupling_index = if total_capsules > 1 {
            coupling_sum / (total_capsules * (total_capsules - 1)) as f32
        } else {
            0.0
        };

        // Cohesion index - based on connected capsule groups
        let cohesion_index = self.calculate_cohesion_index(capsules, relations);

        // Cyclomatic complexity of the graph
        let cyclomatic_complexity = self.calculate_graph_complexity(capsules, relations);

        // Depth levels
        let depth_levels = self.calculate_depth_levels(capsules, relations);

        Ok(GraphMetrics {
            total_capsules,
            total_relations,
            complexity_average,
            coupling_index,
            cohesion_index,
            cyclomatic_complexity,
            depth_levels,
        })
    }

    /// Calculate cohesion index based on layer grouping
    fn calculate_cohesion_index(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> f32 {
        if capsules.is_empty() {
            return 0.0;
        }

        // Group capsules by layers
        let mut layer_groups: HashMap<String, Vec<Uuid>> = HashMap::new();

        for capsule in capsules.values() {
            if let Some(layer) = &capsule.layer {
                layer_groups
                    .entry(layer.clone())
                    .or_insert_with(Vec::new)
                    .push(capsule.id);
            }
        }

        // Calculate intra-layer connections
        let mut total_internal_connections = 0;
        let mut total_possible_connections = 0;

        for group in layer_groups.values() {
            let group_size = group.len();
            if group_size > 1 {
                total_possible_connections += group_size * (group_size - 1);

                // Count actual connections within the group
                for relation in relations {
                    if group.contains(&relation.from_id) && group.contains(&relation.to_id) {
                        total_internal_connections += 1;
                    }
                }
            }
        }

        if total_possible_connections == 0 {
            return 0.0;
        }

        total_internal_connections as f32 / total_possible_connections as f32
    }

    /// Calculate graph cyclomatic complexity
    fn calculate_graph_complexity(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> u32 {
        let nodes = capsules.len() as u32;
        let edges = relations.len() as u32;

        // Approximate number of connected components
        let mut components = 0;
        let mut visited: HashSet<Uuid> = HashSet::new();

        for capsule_id in capsules.keys() {
            if !visited.contains(capsule_id) {
                self.dfs_component_visit(*capsule_id, relations, &mut visited);
                components += 1;
            }
        }

        // Formula: E - N + 2P (where E = edges, N = nodes, P = components)
        if nodes > 0 {
            edges.saturating_sub(nodes) + 2 * components
        } else {
            0
        }
    }

    /// Calculate depth levels in the graph
    fn calculate_depth_levels(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> u32 {
        let mut max_depth = 0;

        for capsule_id in capsules.keys() {
            let depth = self.calculate_dependency_depth(*capsule_id, relations, &mut Vec::new());
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// DFS for component connectivity
    fn dfs_component_visit(
        &self,
        capsule_id: Uuid,
        relations: &[CapsuleRelation],
        visited: &mut HashSet<Uuid>,
    ) {
        visited.insert(capsule_id);

        // Find all connected capsules
        for relation in relations {
            let connected_id = if relation.from_id == capsule_id {
                relation.to_id
            } else if relation.to_id == capsule_id {
                relation.from_id
            } else {
                continue;
            };

            if !visited.contains(&connected_id) {
                self.dfs_component_visit(connected_id, relations, visited);
            }
        }
    }

    /// Calculate dependency depth for a capsule
    pub fn calculate_dependency_depth(
        &self,
        capsule_id: Uuid,
        relations: &[CapsuleRelation],
        visited: &mut Vec<Uuid>,
    ) -> u32 {
        if visited.contains(&capsule_id) {
            return 0; // Avoid infinite recursion
        }

        visited.push(capsule_id);

        let mut max_depth = 0;

        // Find all dependencies
        for relation in relations {
            if relation.from_id == capsule_id {
                let depth = 1 + self.calculate_dependency_depth(relation.to_id, relations, visited);
                max_depth = max_depth.max(depth);
            }
        }

        visited.pop();
        max_depth
    }

    /// Calculate coupling metrics
    pub fn calculate_coupling_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> CouplingMetrics {
        let total_capsules = capsules.len();

        // Afferent coupling (Ca) - incoming dependencies
        let mut afferent_coupling: HashMap<Uuid, u32> = HashMap::new();

        // Efferent coupling (Ce) - outgoing dependencies
        let mut efferent_coupling: HashMap<Uuid, u32> = HashMap::new();

        for relation in relations {
            *efferent_coupling.entry(relation.from_id).or_insert(0) += 1;
            *afferent_coupling.entry(relation.to_id).or_insert(0) += 1;
        }

        // Calculate instability (I = Ce / (Ca + Ce))
        let mut instabilities = Vec::new();
        for capsule_id in capsules.keys() {
            let ca = afferent_coupling.get(capsule_id).unwrap_or(&0);
            let ce = efferent_coupling.get(capsule_id).unwrap_or(&0);

            let instability = if ca + ce > 0 {
                *ce as f32 / (ca + ce) as f32
            } else {
                0.0
            };

            instabilities.push(instability);
        }

        let average_instability = if !instabilities.is_empty() {
            instabilities.iter().sum::<f32>() / instabilities.len() as f32
        } else {
            0.0
        };

        CouplingMetrics {
            average_afferent_coupling: afferent_coupling.values().sum::<u32>() as f32
                / total_capsules as f32,
            average_efferent_coupling: efferent_coupling.values().sum::<u32>() as f32
                / total_capsules as f32,
            average_instability,
            total_coupling: relations.len(),
        }
    }

    /// Calculate cohesion metrics
    pub fn calculate_cohesion_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> CohesionMetrics {
        let mut layer_cohesion_scores: Vec<f32> = Vec::new();

        // Group by layers
        let mut layer_groups: HashMap<String, Vec<Uuid>> = HashMap::new();
        for capsule in capsules.values() {
            if let Some(layer) = &capsule.layer {
                layer_groups
                    .entry(layer.clone())
                    .or_insert_with(Vec::new)
                    .push(capsule.id);
            }
        }

        // Calculate layer cohesion
        let mut layer_cohesion_sum = 0.0;
        let mut layer_count = 0;

        for (_layer_name, capsule_ids) in &layer_groups {
            let mut internal_relations = 0;
            let mut external_relations = 0;

            for relation in relations {
                let from_in_layer = capsule_ids.contains(&relation.from_id);
                let to_in_layer = capsule_ids.contains(&relation.to_id);

                if from_in_layer && to_in_layer {
                    internal_relations += 1;
                } else if from_in_layer || to_in_layer {
                    external_relations += 1;
                }
            }

            let total_relations = internal_relations + external_relations;
            if total_relations > 0 {
                layer_cohesion_sum += internal_relations as f32 / total_relations as f32;
            }
            layer_count += 1;
        }

        CohesionMetrics {
            average_layer_cohesion: if layer_count > 0 {
                layer_cohesion_sum / layer_count as f32
            } else {
                0.0
            },
            layer_count: layer_groups.len(),
            total_internal_relations: layer_cohesion_sum as usize,
        }
    }

    /// Calculate complexity distribution
    pub fn calculate_complexity_distribution(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
    ) -> ComplexityDistribution {
        let complexities: Vec<u32> = capsules.values().map(|c| c.complexity).collect();

        if complexities.is_empty() {
            return ComplexityDistribution::default();
        }

        let min_complexity = *complexities.iter().min().unwrap();
        let max_complexity = *complexities.iter().max().unwrap();
        let avg_complexity = complexities.iter().sum::<u32>() as f32 / complexities.len() as f32;

        // Standard deviation
        let variance = complexities
            .iter()
            .map(|&c| (c as f32 - avg_complexity).powi(2))
            .sum::<f32>()
            / complexities.len() as f32;
        let std_deviation = variance.sqrt();

        // Complexity categories
        let low_complexity = complexities.iter().filter(|&&c| c <= 5).count();
        let medium_complexity = complexities.iter().filter(|&&c| c > 5 && c <= 15).count();
        let high_complexity = complexities.iter().filter(|&&c| c > 15).count();

        ComplexityDistribution {
            min_complexity,
            max_complexity,
            avg_complexity,
            std_deviation,
            low_complexity_count: low_complexity,
            medium_complexity_count: medium_complexity,
            high_complexity_count: high_complexity,
        }
    }
}

/// Coupling metrics
#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    pub average_afferent_coupling: f32,
    pub average_efferent_coupling: f32,
    pub average_instability: f32,
    pub total_coupling: usize,
}

/// Cohesion metrics
#[derive(Debug, Clone)]
pub struct CohesionMetrics {
    pub average_layer_cohesion: f32,
    pub layer_count: usize,
    pub total_internal_relations: usize,
}

/// Complexity distribution
#[derive(Debug, Clone)]
pub struct ComplexityDistribution {
    pub min_complexity: u32,
    pub max_complexity: u32,
    pub avg_complexity: f32,
    pub std_deviation: f32,
    pub low_complexity_count: usize,
    pub medium_complexity_count: usize,
    pub high_complexity_count: usize,
}

impl Default for ComplexityWeights {
    fn default() -> Self {
        Self {
            cyclomatic_weight: 0.3,
            cognitive_weight: 0.3,
            coupling_weight: 0.2,
            cohesion_weight: 0.2,
        }
    }
}

impl Default for ComplexityDistribution {
    fn default() -> Self {
        Self {
            min_complexity: 0,
            max_complexity: 0,
            avg_complexity: 0.0,
            std_deviation: 0.0,
            low_complexity_count: 0,
            medium_complexity_count: 0,
            high_complexity_count: 0,
        }
    }
}

impl Default for MetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
}
