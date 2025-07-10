pub mod core;
pub mod file_scanner;
pub mod parser_ast;
pub mod metadata_extractor;
pub mod capsule_constructor;
pub mod capsule_graph_builder;
pub mod capsule_enricher;
pub mod validator_optimizer;
pub mod exporter;
pub mod commands;
pub mod cli;

#[cfg(test)]
mod test_commands;

pub use core::*; 