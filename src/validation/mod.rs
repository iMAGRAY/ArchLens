/// Validation module - validates and optimizes capsule graphs
pub mod core;
pub mod complexity;
pub mod coupling;
pub mod cohesion;
pub mod patterns;
pub mod solid;
pub mod cycles;
pub mod layers;
pub mod naming;
pub mod optimizer;

pub use core::ValidatorOptimizer;
pub use complexity::ComplexityValidator;
pub use coupling::CouplingValidator;
pub use cohesion::CohesionValidator;
pub use patterns::{PatternDetector, ArchitecturePatternDetector, PatternCriteria};
pub use solid::{SolidAnalyzer, SolidPrinciple};
pub use cycles::CycleDetector;
pub use layers::LayerValidator;
pub use naming::NamingValidator;
pub use optimizer::GraphOptimizer; 