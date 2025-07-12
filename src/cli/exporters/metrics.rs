use std::path::Path;
use super::utils::ExportUtils;

/// Metrics calculator for export analysis
pub struct MetricsCalculator;

#[derive(Debug)]
pub struct QualityMetrics {
    pub maintainability: u8,
    pub complexity: u8,
    pub documentation_coverage: u8,
    pub tech_debt: String,
}

impl MetricsCalculator {
    /// Calculates quality metrics for the project
    pub fn calculate_quality_metrics(project_path: &str) -> std::result::Result<QualityMetrics, String> {
        let maintainability = Self::estimate_maintainability(project_path)?;
        let complexity = Self::estimate_complexity(project_path)?;
        let documentation_coverage = Self::estimate_documentation_coverage(project_path)?;
        let tech_debt = Self::estimate_tech_debt(project_path)?;
        
        Ok(QualityMetrics {
            maintainability,
            complexity,
            documentation_coverage,
            tech_debt,
        })
    }

    /// Estimates maintainability index
    fn estimate_maintainability(project_path: &str) -> std::result::Result<u8, String> {
        let mut score = 100u8;
        
        // Check for large files
        let large_files = ExportUtils::find_large_files(project_path)?;
        if !large_files.is_empty() {
            score = score.saturating_sub(large_files.len() as u8 * 5);
        }
        
        // Check test coverage
        let test_coverage = ExportUtils::estimate_test_coverage(project_path)?;
        if test_coverage < 50 {
            score = score.saturating_sub(20);
        }
        
        Ok(score.max(30))
    }

    /// Estimates complexity
    fn estimate_complexity(project_path: &str) -> std::result::Result<u8, String> {
        let mut complexity = 5u8;
        
        // Check for large files
        let large_files = ExportUtils::find_large_files(project_path)?;
        complexity += large_files.len() as u8;
        
        // Check for nested directories
        if Self::has_deep_nesting(project_path)? {
            complexity += 3;
        }
        
        Ok(complexity.min(20))
    }

    /// Estimates documentation coverage
    fn estimate_documentation_coverage(project_path: &str) -> std::result::Result<u8, String> {
        let mut coverage = 50u8; // Base coverage
        
        // Check for README
        if Path::new(&format!("{}/README.md", project_path)).exists() {
            coverage += 20;
        }
        
        // Check for docs directory
        if Path::new(&format!("{}/docs", project_path)).exists() {
            coverage += 15;
        }
        
        Ok(coverage.min(100))
    }

    /// Estimates technical debt
    fn estimate_tech_debt(project_path: &str) -> std::result::Result<String, String> {
        let large_files = ExportUtils::find_large_files(project_path)?;
        let duplicates = ExportUtils::find_potential_duplicates(project_path)?;
        
        let debt_level = large_files.len() + duplicates.len();
        
        let debt_description = match debt_level {
            0..=2 => "Low",
            3..=5 => "Medium", 
            6..=10 => "High",
            _ => "Critical",
        };
        
        Ok(debt_description.to_string())
    }

    /// Checks for deep directory nesting
    fn has_deep_nesting(project_path: &str) -> std::result::Result<bool, String> {
        use std::fs;
        
        fn check_depth(path: &Path, current_depth: usize) -> std::result::Result<bool, String> {
            if current_depth > 5 {
                return Ok(true);
            }
            
            let entries = fs::read_dir(path)
                .map_err(|e| format!("Failed to read directory: {}", e))?;
                
            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                        
                    if !ExportUtils::should_skip_directory(name) {
                        if check_depth(&path, current_depth + 1)? {
                            return Ok(true);
                        }
                    }
                }
            }
            
            Ok(false)
        }
        
        check_depth(Path::new(project_path), 0)
    }
} 