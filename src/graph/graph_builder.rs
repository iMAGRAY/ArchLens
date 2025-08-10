// Main graph builder orchestrating all components
use crate::graph::{CycleDetector, MetricsCalculator, RelationAnalyzer};
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Main capsule graph builder
pub struct CapsuleGraphBuilder {
    pub cycle_detector: CycleDetector,
    pub relation_analyzer: RelationAnalyzer,
    pub metrics_calculator: MetricsCalculator,
}

impl CapsuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            cycle_detector: CycleDetector::new(),
            relation_analyzer: RelationAnalyzer::new(),
            metrics_calculator: MetricsCalculator::new(),
        }
    }

    /// Build complete capsule graph
    pub fn build_graph(&mut self, capsules: &[Capsule]) -> Result<CapsuleGraph> {
        let mut capsule_map = HashMap::new();
        let mut layers: HashMap<String, Vec<Uuid>> = HashMap::new();

        // Add capsules to graph
        for capsule in capsules {
            capsule_map.insert(capsule.id, capsule.clone());

            // Group by layers
            if let Some(layer) = &capsule.layer {
                layers
                    .entry(layer.clone())
                    .or_insert_with(Vec::new)
                    .push(capsule.id);
            }
        }

        // Build relations between capsules using advanced analysis
        let relations = self.relation_analyzer.build_advanced_relations(capsules)?;

        // Update dependencies in capsules
        let updated_capsules = self
            .relation_analyzer
            .update_capsule_dependencies(&capsule_map, &relations)?;

        // Calculate graph metrics
        let metrics = self
            .metrics_calculator
            .calculate_advanced_metrics(&updated_capsules, &relations)?;

        // Create graph
        let mut graph = CapsuleGraph {
            capsules: updated_capsules,
            relations,
            layers,
            metrics,
            created_at: chrono::Utc::now(),
            previous_analysis: None,
        };

        // Detect cycles
        let cycles = self.cycle_detector.find_cycles(&graph);
        if !cycles.is_empty() {
            self.cycle_detector
                .add_cycle_warnings(&mut graph, &cycles)?;
        }

        Ok(graph)
    }

    /// Get detailed graph analysis
    pub fn analyze_graph(&mut self, graph: &CapsuleGraph) -> Result<GraphAnalysis> {
        let cycles = self.cycle_detector.find_cycles(graph);
        let strongly_connected = self.cycle_detector.get_strongly_connected_components(graph);

        let coupling_metrics = self
            .metrics_calculator
            .calculate_coupling_metrics(&graph.capsules, &graph.relations);
        let cohesion_metrics = self
            .metrics_calculator
            .calculate_cohesion_metrics(&graph.capsules, &graph.relations);
        let complexity_distribution = self
            .metrics_calculator
            .calculate_complexity_distribution(&graph.capsules);

        Ok(GraphAnalysis {
            cycles,
            strongly_connected_components: strongly_connected,
            coupling_metrics,
            cohesion_metrics,
            complexity_distribution,
            total_capsules: graph.capsules.len(),
            total_relations: graph.relations.len(),
            layer_count: graph.layers.len(),
        })
    }

    /// Validate graph structure
    pub fn validate_graph(&mut self, graph: &CapsuleGraph) -> Result<Vec<GraphValidationIssue>> {
        let mut issues = Vec::new();

        // Check for cycles
        let cycles = self.cycle_detector.find_cycles(graph);
        for cycle in cycles {
            issues.push(GraphValidationIssue {
                issue_type: GraphIssueType::CircularDependency,
                severity: Priority::High,
                description: format!("Circular dependency detected with {} capsules", cycle.len()),
                affected_capsules: cycle,
                suggestion: "Break the cycle through dependency inversion or interface extraction"
                    .to_string(),
            });
        }

        // Check for orphaned capsules
        let orphaned = self.find_orphaned_capsules(graph);
        if !orphaned.is_empty() {
            issues.push(GraphValidationIssue {
                issue_type: GraphIssueType::OrphanedCapsules,
                severity: Priority::Medium,
                description: format!("Found {} orphaned capsules", orphaned.len()),
                affected_capsules: orphaned,
                suggestion: "Consider connecting these capsules or removing them if unused"
                    .to_string(),
            });
        }

        // Check for high coupling
        let coupling_metrics = self
            .metrics_calculator
            .calculate_coupling_metrics(&graph.capsules, &graph.relations);
        if coupling_metrics.average_instability > 0.8 {
            issues.push(GraphValidationIssue {
                issue_type: GraphIssueType::HighCoupling,
                severity: Priority::Medium,
                description: format!(
                    "High coupling detected (instability: {:.2})",
                    coupling_metrics.average_instability
                ),
                affected_capsules: Vec::new(),
                suggestion: "Reduce coupling by introducing abstractions and interfaces"
                    .to_string(),
            });
        }

        // Check for low cohesion
        let cohesion_metrics = self
            .metrics_calculator
            .calculate_cohesion_metrics(&graph.capsules, &graph.relations);
        if cohesion_metrics.average_layer_cohesion < 0.3 {
            issues.push(GraphValidationIssue {
                issue_type: GraphIssueType::LowCohesion,
                severity: Priority::Medium,
                description: format!(
                    "Low cohesion detected (cohesion: {:.2})",
                    cohesion_metrics.average_layer_cohesion
                ),
                affected_capsules: Vec::new(),
                suggestion: "Improve cohesion by grouping related functionality".to_string(),
            });
        }

        Ok(issues)
    }

    /// Find orphaned capsules (no connections)
    fn find_orphaned_capsules(&self, graph: &CapsuleGraph) -> Vec<Uuid> {
        let mut orphaned = Vec::new();

        for capsule_id in graph.capsules.keys() {
            let has_incoming = graph.relations.iter().any(|r| r.to_id == *capsule_id);
            let has_outgoing = graph.relations.iter().any(|r| r.from_id == *capsule_id);

            if !has_incoming && !has_outgoing {
                orphaned.push(*capsule_id);
            }
        }

        orphaned
    }

    /// Optimize graph structure
    pub fn optimize_graph(&mut self, graph: &mut CapsuleGraph) -> Result<Vec<OptimizationResult>> {
        let mut optimizations = Vec::new();

        // Remove weak relations
        let original_relation_count = graph.relations.len();
        graph.relations.retain(|r| r.strength > 0.1);
        let removed_relations = original_relation_count - graph.relations.len();

        if removed_relations > 0 {
            optimizations.push(OptimizationResult {
                optimization_type: OptimizationType::WeakRelationRemoval,
                description: format!("Removed {} weak relations", removed_relations),
                impact: OptimizationImpact::Performance,
            });
        }

        // Merge similar capsules (same layer, similar functionality)
        let merge_candidates = self.find_merge_candidates(graph);
        if !merge_candidates.is_empty() {
            optimizations.push(OptimizationResult {
                optimization_type: OptimizationType::CapsuleMerging,
                description: format!("Found {} merge candidates", merge_candidates.len()),
                impact: OptimizationImpact::Structure,
            });
        }

        // Recalculate metrics after optimization
        graph.metrics = self
            .metrics_calculator
            .calculate_advanced_metrics(&graph.capsules, &graph.relations)?;

        Ok(optimizations)
    }

    /// Find candidates for merging
    fn find_merge_candidates(&self, graph: &CapsuleGraph) -> Vec<(Uuid, Uuid)> {
        let mut candidates = Vec::new();

        let capsules: Vec<_> = graph.capsules.values().collect();

        for i in 0..capsules.len() {
            for j in (i + 1)..capsules.len() {
                let capsule1 = capsules[i];
                let capsule2 = capsules[j];

                // Check if they should be merged
                if self.should_merge_capsules(capsule1, capsule2) {
                    candidates.push((capsule1.id, capsule2.id));
                }
            }
        }

        candidates
    }

    /// Check if two capsules should be merged
    fn should_merge_capsules(&self, capsule1: &Capsule, capsule2: &Capsule) -> bool {
        // Same layer
        if capsule1.layer != capsule2.layer {
            return false;
        }

        // Similar size (within 50% difference)
        let size_ratio = (capsule1.size as f32) / (capsule2.size as f32);
        if size_ratio < 0.5 || size_ratio > 2.0 {
            return false;
        }

        // Similar complexity
        let complexity_diff = (capsule1.complexity as i32 - capsule2.complexity as i32).abs();
        if complexity_diff > 5 {
            return false;
        }

        // Same file directory
        if capsule1.file_path.parent() != capsule2.file_path.parent() {
            return false;
        }

        true
    }
}

/// Graph analysis result
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    pub cycles: Vec<Vec<Uuid>>,
    pub strongly_connected_components: Vec<Vec<Uuid>>,
    pub coupling_metrics: crate::graph::CouplingMetrics,
    pub cohesion_metrics: crate::graph::CohesionMetrics,
    pub complexity_distribution: crate::graph::ComplexityDistribution,
    pub total_capsules: usize,
    pub total_relations: usize,
    pub layer_count: usize,
}

/// Graph validation issue
#[derive(Debug, Clone)]
pub struct GraphValidationIssue {
    pub issue_type: GraphIssueType,
    pub severity: Priority,
    pub description: String,
    pub affected_capsules: Vec<Uuid>,
    pub suggestion: String,
}

/// Types of graph issues
#[derive(Debug, Clone)]
pub enum GraphIssueType {
    CircularDependency,
    OrphanedCapsules,
    HighCoupling,
    LowCohesion,
    DeepNesting,
    GodObject,
}

/// Optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub optimization_type: OptimizationType,
    pub description: String,
    pub impact: OptimizationImpact,
}

/// Types of optimizations
#[derive(Debug, Clone)]
pub enum OptimizationType {
    WeakRelationRemoval,
    CapsuleMerging,
    LayerReorganization,
    DependencySimplification,
}

/// Impact of optimization
#[derive(Debug, Clone)]
pub enum OptimizationImpact {
    Performance,
    Structure,
    Maintainability,
    Complexity,
}

impl Default for CapsuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
