// Quality analysis module for code assessment
use crate::types::*;
use std::collections::HashMap;

/// Quality analyzer for code assessment
#[derive(Debug)]
pub struct QualityAnalyzer {
    pub complexity_thresholds: ComplexityThresholds,
    pub quality_weights: QualityWeights,
}

/// Complexity thresholds for different metrics
#[derive(Debug, Clone)]
pub struct ComplexityThresholds {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub max_file_length: usize,
    pub max_method_length: usize,
    pub max_parameter_count: usize,
}

/// Weights for quality calculation
#[derive(Debug, Clone)]
pub struct QualityWeights {
    pub complexity_weight: f32,
    pub documentation_weight: f32,
    pub test_coverage_weight: f32,
    pub maintainability_weight: f32,
    pub technical_debt_weight: f32,
}

/// Detailed quality assessment result
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub overall_score: f32,
    pub maintainability_index: f32,
    pub complexity_score: f32,
    pub documentation_score: f32,
    pub test_coverage_score: f32,
    pub technical_debt_score: f32,
    pub recommendations: Vec<QualityRecommendation>,
    pub metrics_breakdown: HashMap<String, f32>,
}

/// Quality improvement recommendation
#[derive(Debug, Clone)]
pub struct QualityRecommendation {
    pub category: QualityCategory,
    pub priority: Priority,
    pub description: String,
    pub suggestion: String,
    pub estimated_effort: EffortLevel,
}

/// Quality categories
#[derive(Debug, Clone)]
pub enum QualityCategory {
    Complexity,
    Documentation,
    Testing,
    Maintainability,
    Performance,
    Security,
    Architecture,
}

/// Effort levels for improvements
#[derive(Debug, Clone)]
pub enum EffortLevel {
    Low,      // < 1 hour
    Medium,   // 1-4 hours
    High,     // 4-16 hours
    Critical, // > 16 hours
}

impl QualityAnalyzer {
    pub fn new() -> Self {
        Self {
            complexity_thresholds: ComplexityThresholds::default(),
            quality_weights: QualityWeights::default(),
        }
    }

    /// Perform comprehensive quality analysis
    pub fn analyze_quality(&self, capsule: &Capsule, content: &str) -> Result<QualityAssessment> {
        let mut metrics_breakdown = HashMap::new();

        // Calculate individual metrics
        let complexity_score = self.calculate_complexity_score(content);
        let documentation_score = self.calculate_documentation_score(content);
        let test_coverage_score = self.calculate_test_coverage_score(content);
        let maintainability_score = self.calculate_maintainability_score(content);
        let technical_debt_score = self.calculate_technical_debt_score(content);

        // Store metrics
        metrics_breakdown.insert("complexity".to_string(), complexity_score);
        metrics_breakdown.insert("documentation".to_string(), documentation_score);
        metrics_breakdown.insert("test_coverage".to_string(), test_coverage_score);
        metrics_breakdown.insert("maintainability".to_string(), maintainability_score);
        metrics_breakdown.insert("technical_debt".to_string(), technical_debt_score);

        // Calculate weighted overall score
        let overall_score = self.calculate_overall_score(
            complexity_score,
            documentation_score,
            test_coverage_score,
            maintainability_score,
            technical_debt_score,
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            capsule,
            content,
            complexity_score,
            documentation_score,
            test_coverage_score,
            maintainability_score,
            technical_debt_score,
        );

        // Calculate maintainability index using Microsoft formula
        let lines_of_code = content.lines().count() as f32;
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content) as f32;
        let maintainability_index = 171.0
            - 5.2 * lines_of_code.ln()
            - 0.23 * cyclomatic_complexity
            - 16.2 * lines_of_code.ln();

        Ok(QualityAssessment {
            overall_score,
            maintainability_index: maintainability_index.clamp(0.0, 100.0),
            complexity_score,
            documentation_score,
            test_coverage_score,
            technical_debt_score,
            recommendations,
            metrics_breakdown,
        })
    }

    /// Calculate complexity score (0-100, higher is better)
    fn calculate_complexity_score(&self, content: &str) -> f32 {
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content);
        let cognitive_complexity = self.calculate_cognitive_complexity(content);
        let file_length = content.lines().count();

        let mut score = 100.0;

        // Penalize high cyclomatic complexity
        if cyclomatic_complexity > self.complexity_thresholds.cyclomatic_complexity {
            let excess = cyclomatic_complexity - self.complexity_thresholds.cyclomatic_complexity;
            score -= (excess as f32) * 5.0;
        }

        // Penalize high cognitive complexity
        if cognitive_complexity > self.complexity_thresholds.cognitive_complexity {
            let excess = cognitive_complexity - self.complexity_thresholds.cognitive_complexity;
            score -= (excess as f32) * 3.0;
        }

        // Penalize long files
        if file_length > self.complexity_thresholds.max_file_length {
            let excess = file_length - self.complexity_thresholds.max_file_length;
            score -= (excess as f32) * 0.01;
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate documentation score (0-100, higher is better)
    fn calculate_documentation_score(&self, content: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Calculate based on various documentation indicators
        let has_header_comments = content.lines().take(10).any(|line| {
            line.trim().starts_with("//")
                || line.trim().starts_with("/*")
                || line.trim().starts_with("*")
                || line.trim().starts_with("///")
        });

        let has_function_docs = content.contains("///")
            || content.contains("/**")
            || content.contains("\"\"\"")
            || content.contains("#");

        let has_inline_comments = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.contains("//") && !trimmed.starts_with("//")
            })
            .count()
            > 0;

        let comment_density = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("//")
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with("*")
                    || trimmed.starts_with("///")
            })
            .count() as f32
            / content.lines().count().max(1) as f32;

        // Score based on documentation presence
        if has_header_comments {
            score += 25.0;
        }
        if has_function_docs {
            score += 40.0;
        }
        if has_inline_comments {
            score += 15.0;
        }

        // Bonus for good comment density (5-20%)
        if (0.05..=0.20).contains(&comment_density) {
            score += 20.0;
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate test coverage score (0-100, higher is better)
    fn calculate_test_coverage_score(&self, content: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Check for test patterns
        let has_test_functions = content.contains("#[test]")
            || content.contains("test_")
            || content.contains("describe(")
            || content.contains("it(")
            || content.contains("def test_")
            || content.contains("function test")
            || content.contains("@Test");

        let has_assertions = content.contains("assert")
            || content.contains("expect")
            || content.contains("should")
            || content.contains("verify");

        let has_mocks =
            content.contains("mock") || content.contains("stub") || content.contains("spy");

        let has_test_imports = content.contains("import.*test")
            || content.contains("use.*test")
            || content.contains("from.*test");

        // Score based on test indicators
        if has_test_functions {
            score += 40.0;
        }
        if has_assertions {
            score += 30.0;
        }
        if has_mocks {
            score += 20.0;
        }
        if has_test_imports {
            score += 10.0;
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate maintainability score (0-100, higher is better)
    fn calculate_maintainability_score(&self, content: &str) -> f32 {
        let mut score: f32 = 50.0; // Base score

        // Function count analysis
        let function_count = content.matches("fn ").count()
            + content.matches("function ").count()
            + content.matches("def ").count();

        let lines_count = content.lines().count();

        // Good function-to-lines ratio
        if function_count > 0 && lines_count > 0 {
            let ratio = lines_count as f32 / function_count as f32;
            if ratio < 50.0 {
                // Small functions
                score += 20.0;
            } else if ratio > 200.0 {
                // Large functions
                score -= 20.0;
            }
        }

        // Consistent naming patterns
        if self.has_consistent_naming(content) {
            score += 15.0;
        }

        // Proper error handling
        if content.contains("Result") || content.contains("try") || content.contains("catch") {
            score += 10.0;
        }

        // Avoid magic numbers
        if !self.has_magic_numbers(content) {
            score += 10.0;
        }

        // Single responsibility (fewer imports might indicate focused module)
        let import_count = content.matches("import ").count()
            + content.matches("use ").count()
            + content.matches("from ").count();

        if import_count < 10 {
            score += 5.0;
        } else if import_count > 30 {
            score -= 10.0;
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate technical debt score (0-100, higher is better, less debt)
    fn calculate_technical_debt_score(&self, content: &str) -> f32 {
        let mut score: f32 = 100.0; // Start with perfect score

        // Penalize TODO/FIXME comments
        score -= (content.matches("TODO").count() as f32) * 2.0;
        score -= (content.matches("FIXME").count() as f32) * 3.0;
        score -= (content.matches("HACK").count() as f32) * 5.0;
        score -= (content.matches("XXX").count() as f32) * 4.0;

        // Penalize code duplication
        if self.has_code_duplication(content) {
            score -= 15.0;
        }

        // Penalize long lines
        let long_lines = content.lines().filter(|line| line.len() > 120).count();
        score -= (long_lines as f32) * 0.5;

        // Penalize high complexity
        let complexity = self.calculate_cyclomatic_complexity(content);
        if complexity > 10 {
            score -= ((complexity - 10) as f32) * 2.0;
        }

        // Penalize large files
        let line_count = content.lines().count();
        if line_count > 500 {
            score -= ((line_count - 500) as f32) * 0.02;
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate weighted overall score
    fn calculate_overall_score(
        &self,
        complexity_score: f32,
        documentation_score: f32,
        test_coverage_score: f32,
        maintainability_score: f32,
        technical_debt_score: f32,
    ) -> f32 {
        let weights = &self.quality_weights;

        (complexity_score * weights.complexity_weight
            + documentation_score * weights.documentation_weight
            + test_coverage_score * weights.test_coverage_weight
            + maintainability_score * weights.maintainability_weight
            + technical_debt_score * weights.technical_debt_weight)
            / (weights.complexity_weight
                + weights.documentation_weight
                + weights.test_coverage_weight
                + weights.maintainability_weight
                + weights.technical_debt_weight)
    }

    /// Generate quality improvement recommendations
    #[allow(clippy::too_many_arguments)]
    fn generate_recommendations(
        &self,
        _capsule: &Capsule,
        content: &str,
        complexity_score: f32,
        documentation_score: f32,
        test_coverage_score: f32,
        maintainability_score: f32,
        technical_debt_score: f32,
    ) -> Vec<QualityRecommendation> {
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if complexity_score < 60.0 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Complexity,
                priority: Priority::High,
                description: "High complexity detected".to_string(),
                suggestion: "Break down complex functions into smaller, focused functions"
                    .to_string(),
                estimated_effort: EffortLevel::High,
            });
        }

        // Documentation recommendations
        if documentation_score < 40.0 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Documentation,
                priority: Priority::Medium,
                description: "Low documentation coverage".to_string(),
                suggestion: "Add documentation comments for public functions and modules"
                    .to_string(),
                estimated_effort: EffortLevel::Medium,
            });
        }

        // Test coverage recommendations
        if test_coverage_score < 30.0 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Testing,
                priority: Priority::High,
                description: "Low or missing test coverage".to_string(),
                suggestion: "Add unit tests for core functionality".to_string(),
                estimated_effort: EffortLevel::High,
            });
        }

        // Maintainability recommendations
        if maintainability_score < 50.0 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Maintainability,
                priority: Priority::Medium,
                description: "Maintainability issues detected".to_string(),
                suggestion: "Refactor for better separation of concerns".to_string(),
                estimated_effort: EffortLevel::Medium,
            });
        }

        // Technical debt recommendations
        if technical_debt_score < 70.0 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Architecture,
                priority: Priority::Low,
                description: "Technical debt accumulation".to_string(),
                suggestion: "Address TODO comments and code duplication".to_string(),
                estimated_effort: EffortLevel::Low,
            });
        }

        // File size recommendations
        if content.lines().count() > 500 {
            recommendations.push(QualityRecommendation {
                category: QualityCategory::Architecture,
                priority: Priority::Medium,
                description: "Large file size".to_string(),
                suggestion: "Consider splitting into multiple smaller modules".to_string(),
                estimated_effort: EffortLevel::High,
            });
        }

        recommendations
    }

    // Helper methods
    fn calculate_cyclomatic_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Base complexity

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
        let mut complexity = 0;
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
                complexity += 1 + nesting_level;
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                complexity += 1 + nesting_level;
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                complexity += 1 + nesting_level;
            }
        }

        complexity as u32
    }

    fn has_consistent_naming(&self, content: &str) -> bool {
        // Simple heuristic: check if most identifiers follow consistent patterns
        let snake_case_count = content.matches(char::is_lowercase).count();
        let camel_case_count = content.matches(char::is_uppercase).count();

        // If one style dominates, consider it consistent
        let total = snake_case_count + camel_case_count;
        if total > 0 {
            let dominant_ratio = (snake_case_count.max(camel_case_count) as f32) / (total as f32);
            dominant_ratio > 0.7
        } else {
            true
        }
    }

    fn has_magic_numbers(&self, content: &str) -> bool {
        // Look for numeric literals that might be magic numbers
        use regex::Regex;
        // Упростим: числа из 2+ цифр, выделяем вторую группу как само число
        let magic_number_regex = Regex::new(r"(^|[^A-Za-z0-9_.])([0-9]{2,})([^A-Za-z0-9_.]|$)").unwrap();

        // Ignore common non-magic numbers
        let magic_numbers: Vec<String> = magic_number_regex
            .captures_iter(content)
            .filter_map(|cap| cap.get(2))
            .map(|m| m.as_str().to_string())
            .filter(|num| {
                let common: [&str; 6] = ["0", "1", "2", "10", "100", "1000"];
                !common.contains(&num.as_str())
            })
            .collect();

        magic_numbers.len() > 3
    }

    fn has_code_duplication(&self, content: &str) -> bool {
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("#"))
            .collect();

        // Check for repeating blocks of 3+ lines
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

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 10,
            cognitive_complexity: 15,
            max_file_length: 500,
            max_method_length: 50,
            max_parameter_count: 5,
        }
    }
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            complexity_weight: 0.25,
            documentation_weight: 0.15,
            test_coverage_weight: 0.25,
            maintainability_weight: 0.20,
            technical_debt_weight: 0.15,
        }
    }
}

impl Default for QualityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
