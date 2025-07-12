// Relation analysis for capsule connections
use crate::types::*;
use std::collections::HashMap;
use regex::Regex;
use uuid::Uuid;

/// Analyzes relations between capsules
pub struct RelationAnalyzer {
    import_patterns: HashMap<FileType, Vec<Regex>>,
    export_patterns: HashMap<FileType, Vec<Regex>>,
    relation_strength_threshold: f32,
}

impl RelationAnalyzer {
    pub fn new() -> Self {
        Self {
            import_patterns: Self::create_import_patterns(),
            export_patterns: Self::create_export_patterns(),
            relation_strength_threshold: 0.1,
        }
    }
    
    /// Create import patterns for different file types
    fn create_import_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust imports
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"use\s+([^;]+);").unwrap(),
            Regex::new(r"extern\s+crate\s+(\w+)").unwrap(),
            Regex::new(r"mod\s+(\w+)").unwrap(),
        ]);
        
        // JavaScript/TypeScript imports
        let js_patterns = vec![
            Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap(),
            Regex::new(r#"import\s+['"]([^'"]+)['"]"#).unwrap(),
            Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python imports
        patterns.insert(FileType::Python, vec![
            Regex::new(r"import\s+([^\s#]+)").unwrap(),
            Regex::new(r"from\s+([^\s]+)\s+import").unwrap(),
        ]);
        
        // Java imports
        patterns.insert(FileType::Java, vec![
            Regex::new(r"import\s+([^;]+);").unwrap(),
            Regex::new(r"package\s+([^;]+);").unwrap(),
        ]);
        
        // C++ imports
        patterns.insert(FileType::Cpp, vec![
            Regex::new(r#"#include\s+[<"]([^>"]+)[>"]"#).unwrap(),
        ]);
        
        // Go imports
        patterns.insert(FileType::Go, vec![
            Regex::new(r#"import\s+(?:\(\s*)?["']([^"']+)["']"#).unwrap(),
        ]);
        
        patterns
    }
    
    /// Create export patterns for different file types
    fn create_export_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust exports
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"pub\s+(?:fn|struct|enum|mod|trait|const|static)\s+(\w+)").unwrap(),
            Regex::new(r"pub\s+use\s+([^;]+);").unwrap(),
        ]);
        
        // JavaScript/TypeScript exports
        let js_patterns = vec![
            Regex::new(r"export\s+(?:function|class|const|let|var|default)\s+(\w+)").unwrap(),
            Regex::new(r"export\s+\{\s*([^}]+)\s*\}").unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python exports (public functions and classes)
        patterns.insert(FileType::Python, vec![
            Regex::new(r"^(?:def|class)\s+(\w+)").unwrap(),
            Regex::new(r"__all__\s*=\s*\[([^\]]+)\]").unwrap(),
        ]);
        
        // Java exports
        patterns.insert(FileType::Java, vec![
            Regex::new(r"public\s+(?:class|interface|enum|static\s+\w+)\s+(\w+)").unwrap(),
        ]);
        
        // C++ exports
        patterns.insert(FileType::Cpp, vec![
            Regex::new(r"(?:class|struct|namespace|extern)\s+(\w+)").unwrap(),
        ]);
        
        // Go exports
        patterns.insert(FileType::Go, vec![
            Regex::new(r"(?:func|type|var|const)\s+([A-Z]\w*)").unwrap(),
        ]);
        
        patterns
    }
    
    /// Build advanced relations between capsules
    pub fn build_advanced_relations(&self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        for capsule in capsules {
            // Relations through dependencies
            for dep_id in &capsule.dependencies {
                if capsules.iter().any(|c| &c.id == dep_id) {
                    relations.push(CapsuleRelation {
                        from_id: capsule.id,
                        to_id: *dep_id,
                        relation_type: RelationType::Depends,
                        strength: 0.8,
                        description: Some("Direct dependency".to_string()),
                    });
                }
            }
            
            // Relations through file structure
            for other_capsule in capsules {
                if capsule.id != other_capsule.id {
                    if let Some(strength) = self.calculate_file_relation_strength(capsule, other_capsule) {
                        if strength > self.relation_strength_threshold {
                            relations.push(CapsuleRelation {
                                from_id: capsule.id,
                                to_id: other_capsule.id,
                                relation_type: RelationType::References,
                                strength,
                                description: Some("File structure relation".to_string()),
                            });
                        }
                    }
                }
            }
            
            // Relations through architectural layers
            for other_capsule in capsules {
                if capsule.id != other_capsule.id {
                    if let Some(strength) = self.calculate_layer_relation_strength(capsule, other_capsule) {
                        if strength > self.relation_strength_threshold {
                            relations.push(CapsuleRelation {
                                from_id: capsule.id,
                                to_id: other_capsule.id,
                                relation_type: RelationType::Uses,
                                strength,
                                description: Some("Architectural layer relation".to_string()),
                            });
                        }
                    }
                }
            }
            
            // Relations through semantic analysis
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                if let Some(semantic_relations) = self.analyze_semantic_relations(capsule, &content, capsules) {
                    relations.extend(semantic_relations);
                }
            }
        }
        
        Ok(relations)
    }
    
    /// Calculate relation strength based on file structure
    fn calculate_file_relation_strength(&self, capsule1: &Capsule, capsule2: &Capsule) -> Option<f32> {
        let path1 = &capsule1.file_path;
        let path2 = &capsule2.file_path;
        
        // Same directory bonus
        if path1.parent() == path2.parent() {
            return Some(0.3);
        }
        
        // Common parent directories
        let common_depth = self.calculate_common_path_depth(path1, path2);
        if common_depth > 0 {
            return Some(0.1 + (common_depth as f32 * 0.05));
        }
        
        None
    }
    
    /// Calculate relation strength based on architectural layers
    fn calculate_layer_relation_strength(&self, capsule1: &Capsule, capsule2: &Capsule) -> Option<f32> {
        match (&capsule1.layer, &capsule2.layer) {
            (Some(layer1), Some(layer2)) => {
                if layer1 == layer2 {
                    Some(0.4) // Same layer
                } else if self.are_adjacent_layers(layer1, layer2) {
                    Some(0.2) // Adjacent layers
                } else {
                    Some(0.1) // Different layers
                }
            }
            _ => None,
        }
    }
    
    /// Analyze semantic relations through code analysis
    fn analyze_semantic_relations(&self, capsule: &Capsule, content: &str, all_capsules: &[Capsule]) -> Option<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        // Extract imports and exports
        let file_type = self.determine_file_type(&capsule.file_path);
        let imports = self.extract_imports(content, &file_type).unwrap_or_default();
        let exports = self.extract_exports(content, &file_type).unwrap_or_default();
        
        // Find matching capsules
        for other_capsule in all_capsules {
            if capsule.id == other_capsule.id {
                continue;
            }
            
            if let Ok(other_content) = std::fs::read_to_string(&other_capsule.file_path) {
                let other_file_type = self.determine_file_type(&other_capsule.file_path);
                let other_exports = self.extract_exports(&other_content, &other_file_type).unwrap_or_default();
                
                let strength = self.calculate_connection_strength(&imports, &other_exports);
                if strength > self.relation_strength_threshold {
                    relations.push(CapsuleRelation {
                        from_id: capsule.id,
                        to_id: other_capsule.id,
                        relation_type: RelationType::Uses,
                        strength,
                        description: Some("Semantic import-export relation".to_string()),
                    });
                }
            }
        }
        
        if relations.is_empty() {
            None
        } else {
            Some(relations)
        }
    }
    
    /// Extract imports from content
    fn extract_imports(&self, content: &str, file_type: &FileType) -> Result<Vec<String>> {
        let mut imports = Vec::new();
        
        if let Some(patterns) = self.import_patterns.get(file_type) {
            for pattern in patterns {
                for captures in pattern.captures_iter(content) {
                    if let Some(import_match) = captures.get(1) {
                        imports.push(import_match.as_str().to_string());
                    }
                }
            }
        }
        
        Ok(imports)
    }
    
    /// Extract exports from content
    fn extract_exports(&self, content: &str, file_type: &FileType) -> Result<Vec<String>> {
        let mut exports = Vec::new();
        
        if let Some(patterns) = self.export_patterns.get(file_type) {
            for pattern in patterns {
                for captures in pattern.captures_iter(content) {
                    if let Some(export_match) = captures.get(1) {
                        exports.push(export_match.as_str().to_string());
                    }
                }
            }
        }
        
        Ok(exports)
    }
    
    /// Calculate connection strength between imports and exports
    fn calculate_connection_strength(&self, imports: &[String], exports: &[String]) -> f32 {
        let mut strength = 0.0;
        let mut matches = 0;
        
        for import in imports {
            for export in exports {
                if import.contains(export) || export.contains(import) {
                    matches += 1;
                    strength += 1.0;
                }
            }
        }
        
        if imports.is_empty() || exports.is_empty() {
            return 0.0;
        }
        
        strength / ((imports.len() + exports.len()) as f32)
    }
    
    /// Calculate common path depth
    fn calculate_common_path_depth(&self, path1: &std::path::Path, path2: &std::path::Path) -> usize {
        let components1: Vec<_> = path1.components().collect();
        let components2: Vec<_> = path2.components().collect();
        
        components1.iter()
            .zip(components2.iter())
            .take_while(|(a, b)| a == b)
            .count()
    }
    
    /// Check if layers are adjacent in typical architecture
    fn are_adjacent_layers(&self, layer1: &str, layer2: &str) -> bool {
        let adjacent_pairs = [
            ("presentation", "application"),
            ("application", "domain"),
            ("domain", "infrastructure"),
            ("controller", "service"),
            ("service", "repository"),
            ("ui", "core"),
            ("core", "data"),
        ];
        
        adjacent_pairs.iter().any(|(a, b)| {
            (layer1.to_lowercase().contains(a) && layer2.to_lowercase().contains(b)) ||
            (layer1.to_lowercase().contains(b) && layer2.to_lowercase().contains(a))
        })
    }
    
    /// Determine file type from path
    fn determine_file_type(&self, path: &std::path::Path) -> FileType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => FileType::Rust,
            Some("ts") | Some("tsx") => FileType::TypeScript,
            Some("js") | Some("jsx") => FileType::JavaScript,
            Some("py") => FileType::Python,
            Some("java") => FileType::Java,
            Some("go") => FileType::Go,
            Some("cpp") | Some("cc") | Some("cxx") => FileType::Cpp,
            Some("c") => FileType::C,
            Some(ext) => FileType::Other(ext.to_string()),
            None => FileType::Other("unknown".to_string()),
        }
    }
    
    /// Update capsule dependencies based on relations
    pub fn update_capsule_dependencies(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> Result<HashMap<Uuid, Capsule>> {
        let mut updated_capsules = capsules.clone();
        
        for relation in relations {
            // Update dependencies
            if let Some(from_capsule) = updated_capsules.get_mut(&relation.from_id) {
                if !from_capsule.dependencies.contains(&relation.to_id) {
                    from_capsule.dependencies.push(relation.to_id);
                }
            }
            
            // Update dependents
            if let Some(to_capsule) = updated_capsules.get_mut(&relation.to_id) {
                if !to_capsule.dependents.contains(&relation.from_id) {
                    to_capsule.dependents.push(relation.from_id);
                }
            }
        }
        
        Ok(updated_capsules)
    }
}

impl Default for RelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 