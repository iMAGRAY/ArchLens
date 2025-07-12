/// Validator and optimizer - validates and optimizes capsule graphs
/// 
/// This module has been refactored into smaller, focused modules:
/// - validation/core: Main validator coordination
/// - validation/complexity: Complexity validation
/// - validation/coupling: Coupling analysis
/// - validation/cohesion: Cohesion analysis
/// - validation/patterns: Pattern detection
/// - validation/solid: SOLID principles analysis
/// - validation/cycles: Circular dependency detection
/// - validation/layers: Layer hierarchy validation
/// - validation/naming: Naming convention validation
/// - validation/optimizer: Graph optimization

pub use crate::validation::{
    ValidatorOptimizer,
    ComplexityValidator,
    CouplingValidator,
    CohesionValidator,
    PatternDetector,
    SolidAnalyzer,
    CycleDetector,
    LayerValidator,
    NamingValidator,
    GraphOptimizer,
};

// Re-export for backward compatibility
pub use crate::validation::core::ValidatorOptimizer as Validator; 