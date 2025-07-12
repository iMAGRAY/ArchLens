/// Capsule constructor module - creates architectural capsules from AST elements
pub mod core;
pub mod analyzer;
pub mod optimizer;
pub mod warnings;

pub use core::CapsuleConstructor;
pub use analyzer::CapsuleAnalyzer;
pub use optimizer::CapsuleOptimizer;
pub use warnings::WarningAnalyzer; 