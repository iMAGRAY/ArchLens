// Продвинутый построитель графа капсул с реальным анализом зависимостей
// Рефакторенная версия - использует модульную архитектуру

use crate::graph::{
    CapsuleGraphBuilder as CoreGraphBuilder, GraphAnalysis, GraphValidationIssue,
    OptimizationResult,
};
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Главный построитель графа капсул - композитный класс, использующий специализированные анализаторы
pub struct CapsuleGraphBuilder {
    core_builder: CoreGraphBuilder,
}

// Переэкспорт типов из модулей для обратной совместимости
pub use crate::graph::{CycleDetector, GraphIssueType, OptimizationImpact, OptimizationType};

impl CapsuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            core_builder: CoreGraphBuilder::new(),
        }
    }

    /// Main graph building function - delegates to core builder
    pub fn build_graph(&mut self, capsules: &[Capsule]) -> Result<CapsuleGraph> {
        self.core_builder.build_graph(capsules)
    }

    /// Perform comprehensive graph analysis
    pub fn analyze_graph(&mut self, graph: &CapsuleGraph) -> Result<GraphAnalysis> {
        self.core_builder.analyze_graph(graph)
    }

    /// Validate graph structure and find issues
    pub fn validate_graph(&mut self, graph: &CapsuleGraph) -> Result<Vec<GraphValidationIssue>> {
        self.core_builder.validate_graph(graph)
    }

    /// Optimize graph structure
    pub fn optimize_graph(&mut self, graph: &mut CapsuleGraph) -> Result<Vec<OptimizationResult>> {
        self.core_builder.optimize_graph(graph)
    }

    /// Legacy method for backward compatibility
    pub fn calculate_metrics(
        &self,
        capsules: &[Capsule],
        relations: &[CapsuleRelation],
    ) -> Result<GraphMetrics> {
        let capsules_map: HashMap<Uuid, Capsule> =
            capsules.iter().map(|c| (c.id, c.clone())).collect();
        self.core_builder
            .metrics_calculator
            .calculate_advanced_metrics(&capsules_map, relations)
    }

    /// Calculate dependency depth for a capsule
    pub fn calculate_dependency_depth(
        &self,
        capsule_id: Uuid,
        relations: &[CapsuleRelation],
    ) -> u32 {
        self.core_builder
            .metrics_calculator
            .calculate_dependency_depth(capsule_id, relations, &mut Vec::new())
    }

    /// Update capsule dependencies based on relations
    pub fn update_capsule_dependencies(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> Result<HashMap<Uuid, Capsule>> {
        self.core_builder
            .relation_analyzer
            .update_capsule_dependencies(capsules, relations)
    }

    /// Build advanced relations between capsules
    pub fn build_advanced_relations(
        &mut self,
        capsules: &[Capsule],
    ) -> Result<Vec<CapsuleRelation>> {
        self.core_builder
            .relation_analyzer
            .build_advanced_relations(capsules)
    }

    /// Calculate advanced metrics for the graph
    pub fn calculate_advanced_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> Result<GraphMetrics> {
        self.core_builder
            .metrics_calculator
            .calculate_advanced_metrics(capsules, relations)
    }

    /// Add cycle warnings to graph
    pub fn add_cycle_warnings(
        &mut self,
        graph: &mut CapsuleGraph,
        cycles: &[Vec<Uuid>],
    ) -> Result<()> {
        self.core_builder
            .cycle_detector
            .add_cycle_warnings(graph, cycles)
    }

    /// Find cycles in the graph
    pub fn find_cycles(&mut self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        self.core_builder.cycle_detector.find_cycles(graph)
    }

    /// Get strongly connected components
    pub fn get_strongly_connected_components(&mut self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        self.core_builder
            .cycle_detector
            .get_strongly_connected_components(graph)
    }

    /// Check if graph has cycles
    pub fn has_cycles(&mut self, graph: &CapsuleGraph) -> bool {
        self.core_builder.cycle_detector.has_cycles(graph)
    }

    /// Calculate coupling metrics
    pub fn calculate_coupling_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> crate::graph::CouplingMetrics {
        self.core_builder
            .metrics_calculator
            .calculate_coupling_metrics(capsules, relations)
    }

    /// Calculate cohesion metrics
    pub fn calculate_cohesion_metrics(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &[CapsuleRelation],
    ) -> crate::graph::CohesionMetrics {
        self.core_builder
            .metrics_calculator
            .calculate_cohesion_metrics(capsules, relations)
    }

    /// Calculate complexity distribution
    pub fn calculate_complexity_distribution(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
    ) -> crate::graph::ComplexityDistribution {
        self.core_builder
            .metrics_calculator
            .calculate_complexity_distribution(capsules)
    }
}

impl Default for CapsuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
