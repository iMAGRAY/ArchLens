use crate::types::Result;
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    CohesionValidator, ComplexityValidator, CouplingValidator, CycleValidator, GraphOptimizer,
    LayerValidator, NamingValidator, PatternDetector, SolidAnalyzer,
};

/// Main validator and optimizer for capsule graphs
#[derive(Debug)]
pub struct ValidatorOptimizer {
    pub max_complexity_threshold: u32,
    pub coupling_threshold: f32,
    pub cohesion_threshold: f32,
    pub god_object_threshold: u32,

    // Validators
    complexity_validator: ComplexityValidator,
    coupling_validator: CouplingValidator,
    cohesion_validator: CohesionValidator,
    pattern_detector: PatternDetector,
    cycle_validator: CycleValidator,
    layer_validator: LayerValidator,
    naming_validator: NamingValidator,
    optimizer: GraphOptimizer,
}

impl ValidatorOptimizer {
    pub fn new() -> Self {
        Self {
            max_complexity_threshold: 15,
            coupling_threshold: 0.7,
            cohesion_threshold: 0.3,
            god_object_threshold: 20,

            complexity_validator: ComplexityValidator::new(),
            coupling_validator: CouplingValidator::new(),
            cohesion_validator: CohesionValidator::new(),
            pattern_detector: PatternDetector::new(),
            cycle_validator: CycleValidator::new(),
            layer_validator: LayerValidator::new(),
            naming_validator: NamingValidator::new(),
            optimizer: GraphOptimizer::new(),
        }
    }

    /// Main validation and optimization entry point
    pub fn validate_and_optimize(&self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        let mut optimized_graph = graph.clone();
        let mut warnings = Vec::new();

        // Run all validations
        self.complexity_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.coupling_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.cohesion_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.cycle_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.layer_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.naming_validator
            .validate(&optimized_graph, &mut warnings)?;
        self.pattern_detector
            .validate(&optimized_graph, &mut warnings)?;

        // Optimize the graph
        self.optimizer.optimize(&mut optimized_graph)?;

        // Distribute warnings to capsules
        self.distribute_warnings_to_capsules(&mut optimized_graph, warnings)?;

        Ok(optimized_graph)
    }

    /// Distributes warnings to their corresponding capsules
    fn distribute_warnings_to_capsules(
        &self,
        graph: &mut CapsuleGraph,
        warnings: Vec<AnalysisWarning>,
    ) -> Result<()> {
        for warning in warnings {
            if let Some(capsule_id) = warning.capsule_id {
                if let Some(capsule) = graph.capsules.get_mut(&capsule_id) {
                    capsule.warnings.push(warning);
                }
            }
        }
        Ok(())
    }
}

impl Default for ValidatorOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
