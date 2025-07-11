pub mod types;
pub mod file_scanner;
pub mod parser_ast;
pub mod metadata_extractor;
pub mod capsule_constructor;
pub mod capsule_enricher;
pub mod capsule_graph_builder;
pub mod validator_optimizer;
pub mod exporter;
pub mod diff_analyzer;
pub mod advanced_metrics;
pub mod commands;
pub mod cli;
// pub mod integration_tests;  // Временно отключено для отладки

#[cfg(test)]
mod test_commands; 