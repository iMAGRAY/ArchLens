//! # ArchLens - Architectural Analysis Tool
//! 
//! ArchLens is a comprehensive architectural analysis tool for software projects.
//! It provides deep insights into code structure, dependencies, quality metrics,
//! and architectural patterns.
//! 
//! ## Features
//! 
//! - **Code Analysis**: Analyze code structure and complexity
//! - **Dependency Tracking**: Track and visualize dependencies
//! - **Quality Metrics**: Calculate maintainability and quality scores
//! - **Pattern Detection**: Detect architectural patterns and anti-patterns
//! - **Validation**: Validate SOLID principles and architectural rules
//! - **Export**: Export analysis results in various formats
//! 
//! ## Usage
//! 
//! ```rust
//! use archlens::*;
//! 
//! // Analyze a project
//! let analyzer = commands::ArchLensAnalyzer::new();
//! let result = analyzer.analyze_project("path/to/project");
//! ```

/// Core type definitions and data structures
pub mod types;

/// File system scanning and analysis
pub mod file_scanner;

/// Abstract Syntax Tree parsing
pub mod parser_ast;

/// Metadata extraction from files
pub mod metadata_extractor;

/// Modular capsule construction system
pub mod constructor;

/// Legacy capsule constructor (deprecated, use constructor module)
pub mod capsule_constructor;

/// Capsule enrichment with analysis data
pub mod capsule_enricher;

/// Graph building for capsule relationships
pub mod capsule_graph_builder;

/// Validation and optimization system
pub mod validation;

/// Legacy validator (deprecated, use validation module)
pub mod validator_optimizer;

/// Export functionality for analysis results
pub mod exporter;

/// Differential analysis between versions
pub mod diff_analyzer;

/// Advanced metrics calculation
pub mod advanced_metrics;

/// Command handling and execution
pub mod commands;

/// Command-line interface
pub mod cli;

/// Enrichment analysis system
pub mod enrichment;

/// Graph analysis and building
pub mod graph;

// pub mod integration_tests;  // Temporarily disabled for debugging

#[cfg(test)]
mod test_commands; 