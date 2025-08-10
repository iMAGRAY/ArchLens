/// Capsule constructor - creates architectural capsules from AST elements
///
/// This module has been refactored into smaller, focused modules:
/// - core: Main capsule construction logic
/// - analyzer: Capsule analysis capabilities  
/// - optimizer: Capsule optimization and merging
/// - warnings: Warning detection and analysis
pub use crate::constructor::{
    CapsuleAnalyzer, CapsuleConstructor, CapsuleOptimizer, WarningAnalyzer,
};

// Re-export for backward compatibility
pub use crate::constructor::core::CapsuleConstructor as Constructor;
