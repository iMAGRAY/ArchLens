use std::path::Path;
use std::fs;
use std::collections::HashMap;
use super::utils::ExportUtils;

/// Analysis exporter - handles project analysis for exports
pub struct AnalysisExporter;

#[derive(Debug)]
pub struct CompactStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub file_types: HashMap<String, usize>,
    pub components: usize,
    pub connections: usize,
}

#[derive(Debug)]
pub struct CriticalIssue {
    pub severity: String,
    pub description: String,
}

#[derive(Debug)]
pub struct KeyModule {
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug)]
pub struct Recommendation {
    pub priority: String,
    pub description: String,
}

impl AnalysisExporter {
    /// Collects basic project statistics
    pub fn collect_basic_stats(project_path: &str) -> std::result::Result<CompactStats, String> {
        use crate::cli::stats;
        
        let project_stats = stats::get_project_stats(project_path)?;
        
        // Component counting (simplified)
        let components = project_stats.file_types.values().sum::<usize>();
        let connections = (components * 2) / 3; // Approximate estimate
        
        Ok(CompactStats {
            total_files: project_stats.total_files,
            total_lines: project_stats.total_lines,
            file_types: project_stats.file_types,
            components,
            connections,
        })
    }

    /// Analyzes critical issues in the project
    pub fn analyze_critical_issues(project_path: &str) -> std::result::Result<Vec<CriticalIssue>, String> {
        let mut issues = Vec::new();
        
        // Check for large files
        let large_files = ExportUtils::find_large_files(project_path)?;
        if !large_files.is_empty() {
            issues.push(CriticalIssue {
                severity: "HIGH".to_string(),
                description: format!("Found {} large files (>500 lines)", large_files.len()),
            });
        }
        
        // Check for duplicates
        let duplicates = ExportUtils::find_potential_duplicates(project_path)?;
        if !duplicates.is_empty() {
            issues.push(CriticalIssue {
                severity: "MEDIUM".to_string(),
                description: format!("Found {} potential duplicates", duplicates.len()),
            });
        }
        
        Ok(issues)
    }

    /// Analyzes project structure
    pub fn analyze_project_structure(project_path: &str) -> std::result::Result<String, String> {
        let mut structure = String::new();
        let path = Path::new(project_path);
        
        Self::analyze_directory(path, &mut structure, 0)?;
        
        Ok(structure)
    }

    /// Analyzes directory recursively
    fn analyze_directory(dir_path: &Path, structure: &mut String, depth: usize) -> std::result::Result<(), String> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
                
            if ExportUtils::should_skip_directory(name) {
                continue;
            }
            
            let indent = "  ".repeat(depth);
            
            if path.is_dir() {
                structure.push_str(&format!("{}📁 {}/\n", indent, name));
                if depth < 3 { // Limit depth
                    Self::analyze_directory(&path, structure, depth + 1)?;
                }
            } else if ExportUtils::is_important_file(name) {
                structure.push_str(&format!("{}📄 {}\n", indent, name));
            }
        }
        
        Ok(())
    }

    /// Analyzes key modules in the project
    pub fn analyze_key_modules(project_path: &str) -> std::result::Result<Vec<KeyModule>, String> {
        let mut modules = Vec::new();
        
        // Detect Rust project
        if Path::new(&format!("{}/Cargo.toml", project_path)).exists() {
            modules.push(KeyModule {
                name: "Rust Project".to_string(),
                category: "Backend".to_string(),
                description: "Main Rust project".to_string(),
            });
        }
        
        // Detect main entry point
        if Path::new(&format!("{}/src/main.rs", project_path)).exists() {
            modules.push(KeyModule {
                name: "Main Entry".to_string(),
                category: "Core".to_string(),
                description: "Application entry point".to_string(),
            });
        }
        
        Ok(modules)
    }

    /// Generates recommendations based on analysis
    pub fn generate_recommendations(project_path: &str) -> std::result::Result<Vec<Recommendation>, String> {
        let mut recommendations = Vec::new();
        
        // Check for large files
        let large_files = ExportUtils::find_large_files(project_path)?;
        if !large_files.is_empty() {
            recommendations.push(Recommendation {
                priority: "HIGH".to_string(),
                description: "Split large files into smaller modules".to_string(),
            });
        }
        
        // Check test coverage
        let test_coverage = ExportUtils::estimate_test_coverage(project_path)?;
        if test_coverage < 70 {
            recommendations.push(Recommendation {
                priority: "MEDIUM".to_string(),
                description: "Increase test coverage".to_string(),
            });
        }
        
        Ok(recommendations)
    }
} 