use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Export utilities for common operations
pub struct ExportUtils;

impl ExportUtils {
    /// Finds large files in the project
    pub fn find_large_files(project_path: &str) -> std::result::Result<Vec<String>, String> {
        let mut large_files = Vec::new();
        let path = Path::new(project_path);
        
        Self::scan_for_large_files(path, &mut large_files)?;
        
        Ok(large_files)
    }

    /// Scans directory for large files recursively
    fn scan_for_large_files(dir: &Path, large_files: &mut Vec<String>) -> std::result::Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if !Self::should_skip_directory(name) {
                    Self::scan_for_large_files(&path, large_files)?;
                }
            } else if path.is_file() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if Self::is_code_file(path.extension().and_then(|e| e.to_str()).unwrap_or("")) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let line_count = content.lines().count();
                        if line_count > 500 {
                            large_files.push(format!("{} ({} lines)", name, line_count));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Finds potential code duplicates
    pub fn find_potential_duplicates(project_path: &str) -> std::result::Result<Vec<String>, String> {
        let mut duplicates = Vec::new();
        let mut _file_names: HashMap<String, usize> = HashMap::new();
        
        // Simplified duplicate detection
        // This is a placeholder - real implementation would be more complex
        let large_files = Self::find_large_files(project_path)?;
        if large_files.len() > 5 {
            duplicates.push("Multiple large files detected - potential for refactoring".to_string());
        }
        
        Ok(duplicates)
    }

    /// Estimates test coverage
    pub fn estimate_test_coverage(project_path: &str) -> std::result::Result<u8, String> {
        let mut total_files = 0;
        let mut test_files = 0;
        
        let path = Path::new(project_path);
        Self::scan_for_test_files(path, &mut total_files, &mut test_files)?;
        
        if total_files == 0 {
            return Ok(0);
        }
        
        let coverage = (test_files as f64 / total_files as f64 * 100.0) as u8;
        Ok(coverage.min(100))
    }

    /// Scans for test files recursively
    fn scan_for_test_files(dir: &Path, total_files: &mut usize, test_files: &mut usize) -> std::result::Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if !Self::should_skip_directory(name) {
                    Self::scan_for_test_files(&path, total_files, test_files)?;
                }
            } else if path.is_file() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if Self::is_code_file(path.extension().and_then(|e| e.to_str()).unwrap_or("")) {
                    *total_files += 1;
                    
                    if name.contains("test") || name.contains("spec") || 
                       name.starts_with("test_") || name.ends_with("_test.rs") {
                        *test_files += 1;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Checks if directory should be skipped
    pub fn should_skip_directory(dir_name: &str) -> bool {
        matches!(dir_name, "node_modules" | "target" | ".git" | ".idea" | ".vscode" | "dist" | "build")
    }

    /// Checks if file is important for analysis
    pub fn is_important_file(filename: &str) -> bool {
        filename.ends_with(".rs") || filename.ends_with(".toml") || 
        filename.ends_with(".md") || filename.ends_with(".json") ||
        filename == "Cargo.toml" || filename == "README.md"
    }

    /// Checks if file is a code file
    pub fn is_code_file(ext: &str) -> bool {
        matches!(ext, "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "go" | "rb" | "php")
    }
} 