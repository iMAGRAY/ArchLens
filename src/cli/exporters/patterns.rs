use std::path::Path;
use super::utils::ExportUtils;

/// Pattern detector for architectural analysis
pub struct PatternDetector;

#[derive(Debug)]
pub struct ArchitecturalPattern {
    pub name: String,
    pub description: String,
    pub confidence: u8,
}

impl PatternDetector {
    /// Detects architectural patterns in the project
    pub fn detect_architectural_patterns(project_path: &str) -> std::result::Result<Vec<ArchitecturalPattern>, String> {
        let mut patterns = Vec::new();
        
        // Check for MVC pattern
        if Self::has_mvc_structure(project_path)? {
            patterns.push(ArchitecturalPattern {
                name: "MVC".to_string(),
                description: "Model-View-Controller pattern".to_string(),
                confidence: 85,
            });
        }
        
        // Check for modular structure
        if Self::has_modular_structure(project_path)? {
            patterns.push(ArchitecturalPattern {
                name: "Modular".to_string(),
                description: "Modular architecture".to_string(),
                confidence: 90,
            });
        }
        
        // Check for layered architecture
        if Self::has_layered_structure(project_path)? {
            patterns.push(ArchitecturalPattern {
                name: "Layered".to_string(),
                description: "Layered architecture pattern".to_string(),
                confidence: 75,
            });
        }
        
        Ok(patterns)
    }

    /// Checks for MVC structure
    fn has_mvc_structure(project_path: &str) -> std::result::Result<bool, String> {
        let has_models = Path::new(&format!("{}/models", project_path)).exists() ||
                        Path::new(&format!("{}/src/models", project_path)).exists();
        let has_views = Path::new(&format!("{}/views", project_path)).exists() ||
                       Path::new(&format!("{}/src/views", project_path)).exists();
        let has_controllers = Path::new(&format!("{}/controllers", project_path)).exists() ||
                             Path::new(&format!("{}/src/controllers", project_path)).exists();
        
        Ok(has_models && has_views && has_controllers)
    }

    /// Checks for modular structure
    fn has_modular_structure(project_path: &str) -> std::result::Result<bool, String> {
        use std::fs;
        
        let src_path = Path::new(&format!("{}/src", project_path));
        if !src_path.exists() {
            return Ok(false);
        }
        
        let mut module_count = 0;
        let entries = fs::read_dir(src_path)
            .map_err(|e| format!("Failed to read src directory: {}", e))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if !ExportUtils::should_skip_directory(name) {
                    module_count += 1;
                }
            }
        }
        
        Ok(module_count >= 3) // At least 3 modules for modular structure
    }

    /// Checks for layered architecture
    fn has_layered_structure(project_path: &str) -> std::result::Result<bool, String> {
        let common_layers = [
            "presentation", "business", "data", "domain",
            "api", "service", "repository", "entity",
            "ui", "logic", "persistence"
        ];
        
        let mut found_layers = 0;
        
        for layer in &common_layers {
            if Path::new(&format!("{}/{}", project_path, layer)).exists() ||
               Path::new(&format!("{}/src/{}", project_path, layer)).exists() {
                found_layers += 1;
            }
        }
        
        Ok(found_layers >= 2) // At least 2 layers for layered architecture
    }
} 