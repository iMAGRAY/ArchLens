// Advanced semantic analysis for code understanding
use crate::enrichment::enricher_core::*;
use crate::types::*;
use regex::Regex;
use std::collections::HashMap;

/// Semantic analyzer for specific language
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    pub language: FileType,
    pub method_call_patterns: Vec<Regex>,
    pub field_access_patterns: Vec<Regex>,
    pub inheritance_patterns: Vec<Regex>,
    pub composition_patterns: Vec<Regex>,
    pub complexity_patterns: Vec<Regex>,
}

/// Antipattern detector
#[derive(Debug)]
pub struct AntipatternDetector {
    pub pattern_name: String,
    pub detection_regex: Regex,
    pub severity: Priority,
    pub description: String,
}

impl SemanticAnalyzer {
    /// Create semantic analyzers for all supported languages
    pub fn create_analyzers() -> HashMap<FileType, SemanticAnalyzer> {
        let mut analyzers = HashMap::new();

        // Rust analyzer
        analyzers.insert(
            FileType::Rust,
            SemanticAnalyzer {
                language: FileType::Rust,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"(\w+)::(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)").unwrap(),
                    Regex::new(r"self\.(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![
                    Regex::new(r"impl\s+(\w+)\s+for\s+(\w+)").unwrap(),
                    Regex::new(r"struct\s+(\w+).*:\s*(\w+)").unwrap(),
                ],
                composition_patterns: vec![
                    Regex::new(r"struct\s+\w+\s*\{[^}]*(\w+):\s*(\w+)").unwrap(),
                    Regex::new(r"let\s+(\w+)\s*=\s*(\w+)\s*\{").unwrap(),
                ],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\belse\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\bmatch\b").unwrap(),
                    Regex::new(r"\bloop\b").unwrap(),
                ],
            },
        );

        // TypeScript/JavaScript analyzer
        let js_analyzer = SemanticAnalyzer {
            language: FileType::JavaScript,
            method_call_patterns: vec![
                Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                Regex::new(r"this\.(\w+)\s*\(").unwrap(),
            ],
            field_access_patterns: vec![
                Regex::new(r"(\w+)\.(\w+)").unwrap(),
                Regex::new(r"this\.(\w+)").unwrap(),
            ],
            inheritance_patterns: vec![
                Regex::new(r"class\s+(\w+)\s+extends\s+(\w+)").unwrap(),
                Regex::new(r"class\s+(\w+)\s+implements\s+(\w+)").unwrap(),
            ],
            composition_patterns: vec![
                Regex::new(r"new\s+(\w+)\s*\(").unwrap(),
                Regex::new(r"this\.(\w+)\s*=\s*new\s+(\w+)").unwrap(),
            ],
            complexity_patterns: vec![
                Regex::new(r"\bif\b").unwrap(),
                Regex::new(r"\belse\b").unwrap(),
                Regex::new(r"\bfor\b").unwrap(),
                Regex::new(r"\bwhile\b").unwrap(),
                Regex::new(r"\bswitch\b").unwrap(),
                Regex::new(r"\btry\b").unwrap(),
                Regex::new(r"\bcatch\b").unwrap(),
            ],
        };
        analyzers.insert(FileType::JavaScript, js_analyzer.clone());
        analyzers.insert(FileType::TypeScript, js_analyzer);

        // Python analyzer
        analyzers.insert(
            FileType::Python,
            SemanticAnalyzer {
                language: FileType::Python,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"self\.(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)").unwrap(),
                    Regex::new(r"self\.(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![Regex::new(r"class\s+(\w+)\s*\(\s*(\w+)\s*\)").unwrap()],
                composition_patterns: vec![
                    Regex::new(r"self\.(\w+)\s*=\s*(\w+)\s*\(").unwrap(),
                    Regex::new(r"(\w+)\s*=\s*(\w+)\s*\(").unwrap(),
                ],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\belse\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\btry\b").unwrap(),
                    Regex::new(r"\bexcept\b").unwrap(),
                    Regex::new(r"\bwith\b").unwrap(),
                ],
            },
        );

        // Java analyzer
        analyzers.insert(
            FileType::Java,
            SemanticAnalyzer {
                language: FileType::Java,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"this\.(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)").unwrap(),
                    Regex::new(r"this\.(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![
                    Regex::new(r"class\s+(\w+)\s+extends\s+(\w+)").unwrap(),
                    Regex::new(r"class\s+(\w+)\s+implements\s+(\w+)").unwrap(),
                ],
                composition_patterns: vec![
                    Regex::new(r"new\s+(\w+)\s*\(").unwrap(),
                    Regex::new(r"private\s+(\w+)\s+(\w+)").unwrap(),
                ],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\belse\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\bswitch\b").unwrap(),
                    Regex::new(r"\btry\b").unwrap(),
                    Regex::new(r"\bcatch\b").unwrap(),
                ],
            },
        );

        // C++ analyzer
        analyzers.insert(
            FileType::Cpp,
            SemanticAnalyzer {
                language: FileType::Cpp,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"(\w+)->(\w+)\s*\(").unwrap(),
                    Regex::new(r"(\w+)::(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"(\w+)\.(\w+)").unwrap(),
                    Regex::new(r"(\w+)->(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![
                    Regex::new(r"class\s+(\w+)\s*:\s*public\s+(\w+)").unwrap(),
                    Regex::new(r"class\s+(\w+)\s*:\s*private\s+(\w+)").unwrap(),
                ],
                composition_patterns: vec![
                    Regex::new(r"(\w+)\s+(\w+);").unwrap(),
                    Regex::new(r"new\s+(\w+)\s*\(").unwrap(),
                ],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\belse\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\bswitch\b").unwrap(),
                    Regex::new(r"\btry\b").unwrap(),
                    Regex::new(r"\bcatch\b").unwrap(),
                ],
            },
        );

        analyzers
    }

    /// Create antipattern detectors
    pub fn create_antipattern_detectors() -> Vec<AntipatternDetector> {
        vec![
            AntipatternDetector {
                pattern_name: "God Object".to_string(),
                detection_regex: Regex::new(r"class\s+\w+\s*\{[\s\S]{2000,}").unwrap(),
                severity: Priority::Critical,
                description: "Class is too large and has too many responsibilities".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Long Method".to_string(),
                detection_regex: Regex::new(
                    r"(?:fn|function|def)\s+\w+[\s\S]{500,}?(?:\n\s*\}|\n\s*$)",
                )
                .unwrap(),
                severity: Priority::High,
                description: "Method is too long and complex".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Magic Numbers".to_string(),
                detection_regex: Regex::new(r"(^|[^A-Za-z0-9_.])[0-9]{2,}([^A-Za-z0-9_.]|$)").unwrap(),
                severity: Priority::Medium,
                description: "Magic numbers should be replaced with named constants".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Dead Code".to_string(),
                detection_regex: Regex::new(r"//\s*(?:TODO|FIXME|HACK|XXX)").unwrap(),
                severity: Priority::Low,
                description: "Commented code or TODO items detected".to_string(),
            },
        ]
    }

    /// Extract semantic links from code
    pub fn extract_semantic_links(&self, content: &str) -> Result<Vec<SemanticLink>> {
        let mut links = Vec::new();

        // Analyze method calls
        for pattern in &self.method_call_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(object), Some(method)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::MethodCall,
                        target_name: format!("{}.{}", object.as_str(), method.as_str()),
                        strength: 0.8,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }

        // Analyze field access
        for pattern in &self.field_access_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(object), Some(field)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::FieldAccess,
                        target_name: format!("{}.{}", object.as_str(), field.as_str()),
                        strength: 0.6,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }

        // Analyze inheritance
        for pattern in &self.inheritance_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(child), Some(parent)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Inheritance,
                        target_name: format!("{} extends {}", child.as_str(), parent.as_str()),
                        strength: 0.9,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }

        // Analyze composition
        for pattern in &self.composition_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(container), Some(component)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Composition,
                        target_name: format!(
                            "{} contains {}",
                            container.as_str(),
                            component.as_str()
                        ),
                        strength: 0.7,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }

        Ok(links)
    }

    /// Calculate cyclomatic complexity
    pub fn calculate_cyclomatic_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Base complexity

        for pattern in &self.complexity_patterns {
            complexity += pattern.find_iter(content).count() as u32;
        }

        complexity
    }

    /// Calculate cognitive complexity (more sophisticated than cyclomatic)
    pub fn calculate_cognitive_complexity(&self, content: &str) -> u32 {
        let mut complexity: u32 = 0;
        let mut nesting_level: i32 = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Increase nesting for blocks
            if trimmed.contains("{") && !trimmed.contains("}") {
                nesting_level += 1;
            }
            if trimmed.contains("}") && !trimmed.contains("{") {
                nesting_level = nesting_level.saturating_sub(1);
            }

            // Add complexity for control structures
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                complexity = complexity.saturating_add((1 + nesting_level) as u32);
            }
            if trimmed.starts_with("else") {
                complexity = complexity.saturating_add(1);
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                complexity = complexity.saturating_add((1 + nesting_level) as u32);
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                complexity = complexity.saturating_add((1 + nesting_level) as u32);
            }
            if trimmed.starts_with("match ") || trimmed.starts_with("switch ") {
                complexity = complexity.saturating_add((1 + nesting_level) as u32);
            }
        }

        complexity
    }
}

/// Semantic enricher for analyzing code relationships and patterns
#[derive(Debug)]
pub struct SemanticEnricher {
    pub analyzers: HashMap<FileType, SemanticAnalyzer>,
    pub antipattern_detectors: Vec<AntipatternDetector>,
}

impl SemanticEnricher {
    pub fn new() -> Self {
        Self {
            analyzers: SemanticAnalyzer::create_analyzers(),
            antipattern_detectors: SemanticAnalyzer::create_antipattern_detectors(),
        }
    }

    /// Perform full semantic analysis
    pub fn perform_semantic_analysis(
        &self,
        capsule: &Capsule,
        content: &str,
    ) -> Result<EnrichmentResult> {
        let file_type = self.determine_file_type(&capsule.file_path);
        let analyzer = self.analyzers.get(&file_type);

        let semantic_links = if let Some(analyzer) = analyzer {
            analyzer.extract_semantic_links(content)?
        } else {
            Vec::new()
        };

        let quality_metrics = self.calculate_quality_metrics(content, &semantic_links)?;
        let architectural_patterns =
            self.detect_architectural_patterns(content, &semantic_links)?;
        let code_smells = self.detect_code_smells(content)?;

        Ok(EnrichmentResult {
            semantic_links,
            quality_metrics,
            architectural_patterns,
            code_smells,
        })
    }

    /// Calculate quality metrics
    fn calculate_quality_metrics(
        &self,
        content: &str,
        semantic_links: &[SemanticLink],
    ) -> Result<QualityMetrics> {
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content);
        let cognitive_complexity = self.calculate_cognitive_complexity(content);
        let documentation_ratio = self.calculate_documentation_ratio(content);
        let test_coverage = self.estimate_test_coverage(content);
        let tech_debt_ratio = self.calculate_technical_debt_ratio(content, semantic_links);

        // Calculate maintainability index (Microsoft formula)
        let lines_of_code = content.lines().count() as f32;
        let maintainability_index = 171.0
            - 5.2 * (lines_of_code.ln())
            - 0.23 * (cyclomatic_complexity as f32)
            - 16.2 * (lines_of_code.ln());

        Ok(QualityMetrics {
            maintainability_index: maintainability_index.clamp(0.0, 100.0),
            cognitive_complexity,
            technical_debt_ratio: tech_debt_ratio,
            test_coverage_estimate: test_coverage,
            documentation_completeness: documentation_ratio,
        })
    }

    /// Detect architectural patterns
    fn detect_architectural_patterns(
        &self,
        content: &str,
        _semantic_links: &[SemanticLink],
    ) -> Result<Vec<ArchitecturalPattern>> {
        let mut patterns = Vec::new();

        // Singleton pattern
        if content.contains("private static") && content.contains("getInstance") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Singleton,
                confidence: 0.8,
                description: "Detected Singleton pattern".to_string(),
            });
        }

        // Factory pattern
        if content.contains("create") && content.contains("new ") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Factory,
                confidence: 0.6,
                description: "Possible Factory pattern usage".to_string(),
            });
        }

        // Repository pattern
        if content.contains("Repository") || content.contains("findBy") || content.contains("save")
        {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Repository,
                confidence: 0.7,
                description: "Detected Repository pattern".to_string(),
            });
        }

        // Service pattern
        if content.contains("Service") || content.contains("@Service") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Service,
                confidence: 0.8,
                description: "Detected Service pattern".to_string(),
            });
        }

        // Controller pattern
        if content.contains("Controller")
            || content.contains("@Controller")
            || content.contains("@RestController")
        {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Controller,
                confidence: 0.8,
                description: "Detected Controller pattern".to_string(),
            });
        }

        Ok(patterns)
    }

    /// Detect code smells
    fn detect_code_smells(&self, content: &str) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();

        // Long method
        if content.len() > 2000 {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::LongMethod,
                severity: Priority::High,
                description: "Method or file is too long".to_string(),
                suggestion: "Break down into smaller, focused methods".to_string(),
            });
        }

        // God object
        if content.matches("class ").count() == 1 && content.len() > 5000 {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::GodObject,
                severity: Priority::Critical,
                description: "Class has too many responsibilities".to_string(),
                suggestion: "Split into multiple classes with single responsibilities".to_string(),
            });
        }

        // Dead code
        if content.contains("TODO") || content.contains("FIXME") || content.contains("HACK") {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::DeadCode,
                severity: Priority::Low,
                description: "TODO/FIXME comments indicate unfinished code".to_string(),
                suggestion: "Complete the implementation or remove TODO comments".to_string(),
            });
        }

        // Duplicated code
        if self.has_code_duplication(content) {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::DuplicatedCode,
                severity: Priority::Medium,
                description: "Potential code duplication detected".to_string(),
                suggestion: "Extract common code into shared functions".to_string(),
            });
        }

        Ok(smells)
    }

    // Helper methods
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

    fn calculate_cyclomatic_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Base complexity

        // Count decision points
        complexity += content.matches("if ").count() as u32;
        complexity += content.matches("else").count() as u32;
        complexity += content.matches("for ").count() as u32;
        complexity += content.matches("while ").count() as u32;
        complexity += content.matches("match ").count() as u32;
        complexity += content.matches("switch ").count() as u32;
        complexity += content.matches("case ").count() as u32;
        complexity += content.matches("catch ").count() as u32;
        complexity += content.matches("||").count() as u32;
        complexity += content.matches("&&").count() as u32;

                complexity
    }
 
     fn calculate_cognitive_complexity(&self, content: &str) -> u32 {
        // Simplified cognitive complexity calculation
        let mut complexity: u32 = 0;
        let mut nesting_level: i32 = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.contains("{") && !trimmed.contains("}") {
                nesting_level += 1;
            }
            if trimmed.contains("}") && !trimmed.contains("{") {
                nesting_level = nesting_level.saturating_sub(1);
            }

            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                complexity = complexity.saturating_add((1 + nesting_level) as u32);
            }
        }
        
        complexity
    }

    fn calculate_documentation_ratio(&self, content: &str) -> f32 {
        let total_lines = content.lines().count() as f32;
        if total_lines == 0.0 {
            return 0.0;
        }

        let doc_lines = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("///")
                    || trimmed.starts_with("/**")
                    || trimmed.starts_with("*")
                    || trimmed.starts_with("\"\"\"")
                    || trimmed.starts_with("'''")
            })
            .count() as f32;

        (doc_lines / total_lines).min(1.0)
    }

    fn estimate_test_coverage(&self, content: &str) -> f32 {
        let has_tests = content.contains("test")
            || content.contains("Test")
            || content.contains("spec")
            || content.contains("Spec")
            || content.contains("#[test]")
            || content.contains("@Test");

        if has_tests {
            0.8 // Rough estimate
        } else {
            0.0
        }
    }

    fn calculate_technical_debt_ratio(
        &self,
        content: &str,
        _semantic_links: &[SemanticLink],
    ) -> f32 {
        let mut debt_score = 0.0;

        // TODO/FIXME comments
        debt_score += (content.matches("TODO").count() * 2) as f32;
        debt_score += (content.matches("FIXME").count() * 3) as f32;
        debt_score += (content.matches("HACK").count() * 4) as f32;

        // Code duplication
        if self.has_code_duplication(content) {
            debt_score += 5.0;
        }

        // High complexity
        if self.calculate_cyclomatic_complexity(content) > 10 {
            debt_score += 3.0;
        }

        // Long lines
        for line in content.lines() {
            if line.len() > 120 {
                debt_score += 0.1;
            }
        }

        // Normalize (0.0 - 1.0)
        (debt_score / 100.0).min(1.0)
    }

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
}

impl Default for SemanticEnricher {
    fn default() -> Self {
        Self::new()
    }
}
