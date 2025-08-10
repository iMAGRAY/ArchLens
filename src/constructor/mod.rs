pub mod analyzer;
/// Capsule constructor module - creates architectural capsules from AST elements
pub mod core;
pub mod optimizer;
pub mod warnings;

pub use analyzer::CapsuleAnalyzer;
pub use core::CapsuleConstructor;
pub use optimizer::CapsuleOptimizer;
pub use warnings::WarningAnalyzer;
