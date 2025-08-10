// Core enrichment logic for capsules
use crate::types::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use uuid::Uuid;

/// Core capsule enricher with main enrichment functionality
pub struct CapsuleEnricher {
    pub import_patterns: HashMap<FileType, Regex>,
    pub export_patterns: HashMap<FileType, Regex>,
    pub analysis_cache: HashMap<String, EnrichmentResult>,
}

/// Result of capsule enrichment
#[derive(Debug, Clone)]
pub struct EnrichmentResult {
    pub semantic_links: Vec<SemanticLink>,
    pub quality_metrics: QualityMetrics,
    pub architectural_patterns: Vec<ArchitecturalPattern>,
    pub code_smells: Vec<CodeSmell>,
}

/// Semantic link between elements
#[derive(Debug, Clone)]
pub struct SemanticLink {
    pub link_type: SemanticLinkType,
    pub target_name: String,
    pub strength: f32,
    pub context: String,
}

/// Types of semantic links
#[derive(Debug, Clone)]
pub enum SemanticLinkType {
    MethodCall,
    FieldAccess,
    Inheritance,
    Composition,
    Aggregation,
    Dependency,
    Association,
}

/// Code quality metrics
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub maintainability_index: f32,
    pub cognitive_complexity: u32,
    pub technical_debt_ratio: f32,
    pub test_coverage_estimate: f32,
    pub documentation_completeness: f32,
}

/// Architectural patterns
#[derive(Debug, Clone)]
pub struct ArchitecturalPattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub description: String,
}

/// Pattern types
#[derive(Debug, Clone)]
pub enum PatternType {
    Singleton,
    Factory,
    Observer,
    Strategy,
    Command,
    Builder,
    Adapter,
    Repository,
    Service,
    Controller,
    Entity,
    ValueObject,
}

/// Code smell detector
#[derive(Debug, Clone)]
pub struct CodeSmell {
    pub smell_type: CodeSmellType,
    pub severity: Priority,
    pub description: String,
    pub suggestion: String,
}

/// Code smell types
#[derive(Debug, Clone)]
pub enum CodeSmellType {
    LongMethod,
    LongParameterList,
    LargeClass,
    DuplicatedCode,
    DeadCode,
    GodObject,
    FeatureEnvy,
    DataClump,
    PrimitiveObsession,
    ShotgunSurgery,
}

impl CapsuleEnricher {
    pub fn new() -> Self {
        let mut import_patterns = HashMap::new();
        let mut export_patterns = HashMap::new();

        // Rust patterns
        import_patterns.insert(FileType::Rust, Regex::new(r"use\s+([^;]+);").unwrap());
        export_patterns.insert(
            FileType::Rust,
            Regex::new(r"pub\s+(fn|struct|enum|mod|trait|const|static)\s+(\w+)").unwrap(),
        );

        // JavaScript/TypeScript patterns
        let js_import = Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#).unwrap();
        import_patterns.insert(FileType::JavaScript, js_import.clone());
        import_patterns.insert(FileType::TypeScript, js_import);

        let js_export =
            Regex::new(r"export\s+(function|class|const|let|var|default)\s+(\w+)").unwrap();
        export_patterns.insert(FileType::JavaScript, js_export.clone());
        export_patterns.insert(FileType::TypeScript, js_export);

        // Python patterns
        import_patterns.insert(
            FileType::Python,
            Regex::new(r"(?:from\s+(\S+)\s+)?import\s+([^#\n]+)").unwrap(),
        );
        export_patterns.insert(
            FileType::Python,
            Regex::new(r"^(def|class)\s+(\w+)").unwrap(),
        );

        // Java patterns
        import_patterns.insert(FileType::Java, Regex::new(r"import\s+([^;]+);").unwrap());
        export_patterns.insert(
            FileType::Java,
            Regex::new(r"public\s+(class|interface|enum|static\s+\w+)\s+(\w+)").unwrap(),
        );

        // C++ patterns
        import_patterns.insert(
            FileType::Cpp,
            Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).unwrap(),
        );
        export_patterns.insert(
            FileType::Cpp,
            Regex::new(r"(class|struct|namespace|extern)\s+(\w+)").unwrap(),
        );

        Self {
            import_patterns,
            export_patterns,
            analysis_cache: HashMap::new(),
        }
    }

    /// Main enrichment function for capsule graphs
    pub fn enrich_graph(&self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        let mut enriched_capsules = HashMap::new();
        let mut enriched_relations = graph.relations.clone();

        for (id, capsule) in &graph.capsules {
            let mut enriched_capsule = capsule.clone();

            // Enrich metadata from file content
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                self.enrich_capsule_metadata(&mut enriched_capsule, &content)?;
                self.analyze_dependencies(&mut enriched_capsule, &content)?;
                self.extract_exports(&mut enriched_capsule, &content)?;
                self.generate_warnings(&mut enriched_capsule, &content)?;
            }

            enriched_capsules.insert(*id, enriched_capsule);
        }

        // Enrich relations based on found dependencies
        self.enrich_relations(&enriched_capsules, &mut enriched_relations)?;

        Ok(CapsuleGraph {
            capsules: enriched_capsules,
            relations: enriched_relations,
            layers: graph.layers.clone(),
            metrics: graph.metrics.clone(),
            created_at: graph.created_at,
            previous_analysis: graph.previous_analysis.clone(),
        })
    }

    /// Enrich capsule metadata from content
    fn enrich_capsule_metadata(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        // Extract comments and documentation
        let doc_comments = self.extract_documentation(content, &capsule.file_path);
        if let Some(doc) = doc_comments.first() {
            capsule.summary = Some(doc.clone());
        }

        // Update line count
        let actual_lines = content.lines().count();
        if actual_lines != capsule.line_end {
            capsule.line_end = actual_lines;
        }

        // Content-based metadata
        capsule
            .metadata
            .insert("actual_lines".to_string(), actual_lines.to_string());
        capsule
            .metadata
            .insert("has_tests".to_string(), self.has_tests(content).to_string());
        capsule.metadata.insert(
            "has_documentation".to_string(),
            (!doc_comments.is_empty()).to_string(),
        );

        // Code quality analysis
        let code_quality_score = self.calculate_code_quality(content);
        capsule
            .metadata
            .insert("quality_score".to_string(), code_quality_score.to_string());

        Ok(())
    }

    /// Analyze dependencies in capsule content
    fn analyze_dependencies(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        let file_type = self.determine_file_type(&capsule.file_path);

        if let Some(pattern) = self.import_patterns.get(&file_type) {
            let mut dependencies = HashSet::new();

            for capture in pattern.captures_iter(content) {
                if let Some(dep) = capture.get(1).or_else(|| capture.get(2)) {
                    let dep_str = dep.as_str().trim();
                    dependencies.insert(dep_str.to_string());
                }
            }

            capsule.metadata.insert(
                "external_dependencies".to_string(),
                dependencies.into_iter().collect::<Vec<_>>().join(", "),
            );
        }

        Ok(())
    }

    /// Extract exports from capsule content
    fn extract_exports(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        let file_type = self.determine_file_type(&capsule.file_path);

        if let Some(pattern) = self.export_patterns.get(&file_type) {
            let mut exports = Vec::new();

            for capture in pattern.captures_iter(content) {
                if let Some(export_name) = capture.get(2) {
                    exports.push(export_name.as_str().to_string());
                }
            }

            capsule
                .metadata
                .insert("public_exports".to_string(), exports.join(", "));
        }

        Ok(())
    }

    /// Generate warnings for capsule
    fn generate_warnings(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        let mut warnings = Vec::new();

        // Generate warnings for long methods
        if content.lines().count() > 50 {
            warnings.push(AnalysisWarning {
                message: format!(
                    "Method/function is too long ({} lines)",
                    content.lines().count()
                ),
                level: Priority::Medium,
                category: "code_quality".to_string(),
                capsule_id: None,
                suggestion: Some(
                    "Consider breaking this method into smaller functions".to_string(),
                ),
            });
        }

        // Generate warnings for code duplication
        if self.has_code_duplication(content) {
            warnings.push(AnalysisWarning {
                message: "Potential code duplication detected".to_string(),
                level: Priority::High,
                category: "code_quality".to_string(),
                capsule_id: None,
                suggestion: Some("Consider extracting common functionality".to_string()),
            });
        }

        // Generate warnings for missing documentation
        if !self.has_documentation(content) {
            warnings.push(AnalysisWarning {
                message: "Missing documentation".to_string(),
                level: Priority::Low,
                category: "documentation".to_string(),
                capsule_id: None,
                suggestion: Some("Add documentation comments".to_string()),
            });
        }

        capsule.warnings.extend(warnings);
        Ok(())
    }

    /// Enrich relations between capsules
    fn enrich_relations(
        &self,
        capsules: &HashMap<Uuid, Capsule>,
        relations: &mut Vec<CapsuleRelation>,
    ) -> Result<()> {
        // Add new relations based on found dependencies
        for capsule in capsules.values() {
            if let Some(deps) = capsule.metadata.get("external_dependencies") {
                for dep_name in deps.split(", ") {
                    if dep_name.is_empty() {
                        continue;
                    }

                    // Find capsule with this name
                    for other_capsule in capsules.values() {
                        if other_capsule.name.contains(dep_name)
                            || other_capsule.file_path.to_string_lossy().contains(dep_name)
                        {
                            // Check if relation already exists
                            let relation_exists = relations
                                .iter()
                                .any(|r| r.from_id == capsule.id && r.to_id == other_capsule.id);

                            if !relation_exists {
                                relations.push(CapsuleRelation {
                                    from_id: capsule.id,
                                    to_id: other_capsule.id,
                                    relation_type: RelationType::Uses,
                                    strength: 0.6,
                                    description: Some(format!("Uses {dep_name}")),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract documentation from content
    fn extract_documentation(&self, content: &str, file_path: &Path) -> Vec<String> {
        let mut docs = Vec::new();

        // Rust-style documentation
        if file_path.extension().is_some_and(|e| e == "rs") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("///") {
                    docs.push(trimmed.trim_start_matches("///").trim().to_string());
                }
            }
        }

        // JS/TS-style documentation
        if file_path
            .extension()
            .is_some_and(|e| e == "js" || e == "ts" || e == "tsx")
        {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("/**") || trimmed.starts_with("*") {
                    docs.push(
                        trimmed
                            .trim_start_matches("/**")
                            .trim_start_matches("*")
                            .trim()
                            .to_string(),
                    );
                }
            }
        }

        // Python-style documentation
        if file_path.extension().is_some_and(|e| e == "py") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                    docs.push(
                        trimmed
                            .trim_start_matches("\"\"\"")
                            .trim_start_matches("'''")
                            .trim()
                            .to_string(),
                    );
                }
            }
        }

        docs
    }

    /// Check if content has tests
    fn has_tests(&self, content: &str) -> bool {
        content.contains("#[test]")
            || content.contains("test(")
            || content.contains("describe(")
            || content.contains("it(")
            || content.contains("def test_")
            || content.contains("@Test")
    }

    /// Check if content has documentation
    fn has_documentation(&self, content: &str) -> bool {
        content.contains("///")
            || content.contains("/**")
            || content.contains("\"\"\"")
            || content.contains("'''")
            || content.contains("@param")
            || content.contains("@return")
    }

    /// Calculate code quality score
    fn calculate_code_quality(&self, content: &str) -> f32 {
        let mut score: f32 = 50.0; // Base score

        // Function count analysis
        let function_starts = content.matches("fn ").count()
            + content.matches("function ").count()
            + content.matches("def ").count()
            + content.matches("public ").count();

        if function_starts > 0 {
            score += 10.0;
        }

        // Documentation bonus
        if self.has_documentation(content) {
            score += 15.0;
        }

        // Test bonus
        if self.has_tests(content) {
            score += 20.0;
        }

        // Penalty for code duplication
        if self.has_code_duplication(content) {
            score -= 20.0;
        }

        score.clamp(0.0, 100.0)
    }

    /// Check for code duplication
    fn has_code_duplication(&self, content: &str) -> bool {
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("#"))
            .collect();

        // Simple check for repeating blocks of 3+ lines
        for i in 0..lines.len().saturating_sub(3) {
            let block = &lines[i..i + 3];
            for j in (i + 3)..lines.len().saturating_sub(3) {
                let other_block = &lines[j..j + 3];
                if block == other_block {
                    return true;
                }
            }
        }

        false
    }

    /// Determine file type by extension
    fn determine_file_type(&self, path: &Path) -> FileType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => FileType::Rust,
            Some("ts") | Some("tsx") => FileType::TypeScript,
            Some("js") | Some("jsx") => FileType::JavaScript,
            Some("py") => FileType::Python,
            Some("java") => FileType::Java,
            Some("go") => FileType::Go,
            Some("cpp") | Some("cc") | Some("cxx") => FileType::Cpp,
            Some("c") => FileType::C,
            Some("h") | Some("hpp") => FileType::Other("header".to_string()),
            Some("json") => FileType::Other("json".to_string()),
            Some("yaml") | Some("yml") => FileType::Other("yaml".to_string()),
            Some("toml") => FileType::Other("toml".to_string()),
            Some("md") => FileType::Other("markdown".to_string()),
            Some("txt") => FileType::Other("text".to_string()),
            Some(ext) => FileType::Other(ext.to_string()),
            None => FileType::Other("unknown".to_string()),
        }
    }
}

impl Default for CapsuleEnricher {
    fn default() -> Self {
        Self::new()
    }
}
