pub mod cohesion;
pub mod complexity;
/// Validation module - validates and optimizes capsule graphs
pub mod core;
pub mod coupling;
pub mod cycles;
pub mod layers;
pub mod naming;
pub mod optimizer;
pub mod patterns;
pub mod solid;

pub use cohesion::CohesionValidator;
pub use complexity::ComplexityValidator;
pub use core::ValidatorOptimizer;
pub use coupling::CouplingValidator;
pub use cycles::CycleValidator;
pub use layers::LayerValidator;
pub use naming::NamingValidator;
pub use optimizer::GraphOptimizer;
pub use patterns::{ArchitecturePatternDetector, PatternCriteria, PatternDetector};
pub use solid::{SolidAnalyzer, SolidPrinciple};
