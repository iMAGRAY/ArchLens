use crate::types::{AnalysisWarning, Priority};
use crate::parser_ast::ASTElement;
use std::collections::HashMap;

/// Warning analyzer for capsule elements
pub struct WarningAnalyzer;

impl WarningAnalyzer {
    /// Analyzes warnings for element
    pub fn analyze_warnings(element: &ASTElement) -> Vec<AnalysisWarning> {
        let mut warnings = Vec::new();
        let content_lower = element.content.to_lowercase();

        // Check complexity
        if element.complexity > 10 {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!("High complexity: {}", element.complexity),
                category: "complexity".to_string(),
                capsule_id: None,
                suggestion: Some("Break into smaller functions".to_string()),
            });
        }

        // Check size
        let lines_count = element.end_line - element.start_line + 1;
        if lines_count > 100 {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!("Large size: {lines_count} lines"),
                category: "size".to_string(),
                capsule_id: None,
                suggestion: Some("Consider breaking into multiple modules".to_string()),
            });
        }

        // Check documentation for public elements
        if !matches!(element.element_type, crate::parser_ast::ASTElementType::Import | crate::parser_ast::ASTElementType::Export)
            && element.visibility == "public"
            && !content_lower.contains("///") && !content_lower.contains("/**") {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: "Public element without documentation".to_string(),
                    category: "documentation".to_string(),
                    capsule_id: None,
                    suggestion: Some("Add documentation to public interfaces".to_string()),
                });
            }

        // Check for TODO/FIXME
        if content_lower.contains("todo") {
            warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: "Contains TODO".to_string(),
                category: "maintenance".to_string(),
                capsule_id: None,
                suggestion: Some("Complete or plan TODO execution".to_string()),
            });
        }
        if content_lower.contains("fixme") {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: "Contains FIXME".to_string(),
                category: "maintenance".to_string(),
                capsule_id: None,
                suggestion: Some("Fix indicated issues".to_string()),
            });
        }

        // Check for code duplication
        if element.content.len() > 500 && Self::has_repeated_patterns(&element.content) {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: "Possible code duplication".to_string(),
                category: "duplication".to_string(),
                capsule_id: None,
                suggestion: Some("Extract common logic into separate methods".to_string()),
            });
        }

        warnings
    }

    /// Checks for repeated patterns in code
    fn has_repeated_patterns(content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        let mut pattern_count = HashMap::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.len() > 20 {
                *pattern_count.entry(trimmed).or_insert(0) += 1;
            }
        }
        
        pattern_count.values().any(|&count| count > 3)
    }
} 