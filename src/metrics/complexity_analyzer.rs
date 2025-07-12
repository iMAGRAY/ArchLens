// Complexity analysis for code metrics
use crate::types::*;
use regex::Regex;
use std::collections::HashMap;

/// Complexity analyzer for different types of complexity metrics
pub struct ComplexityAnalyzer {
    cyclomatic_patterns: HashMap<FileType, Vec<CyclomaticPattern>>,
    cognitive_patterns: HashMap<FileType, Vec<CognitivePattern>>,
}

/// Pattern for cyclomatic complexity calculation
#[derive(Debug, Clone)]
pub struct CyclomaticPattern {
    pub pattern: Regex,
    pub complexity_weight: u32,
    pub description: String,
}

/// Pattern for cognitive complexity calculation
#[derive(Debug, Clone)]
pub struct CognitivePattern {
    pub pattern: Regex,
    pub cognitive_weight: u32,
    pub nesting_penalty: u32,
    pub description: String,
}

/// Complexity metrics result
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub essential_complexity: u32,
    pub design_complexity: u32,
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            cyclomatic_patterns: Self::create_cyclomatic_patterns(),
            cognitive_patterns: Self::create_cognitive_patterns(),
        }
    }
    
    /// Calculate all complexity metrics for content
    pub fn calculate_complexity(&self, content: &str, file_type: &FileType) -> Result<ComplexityMetrics> {
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content, file_type)?;
        let cognitive_complexity = self.calculate_cognitive_complexity(content, file_type)?;
        let essential_complexity = self.calculate_essential_complexity(content, file_type)?;
        let design_complexity = self.calculate_design_complexity(content, file_type)?;
        
        Ok(ComplexityMetrics {
            cyclomatic_complexity,
            cognitive_complexity,
            essential_complexity,
            design_complexity,
        })
    }
    
    /// Calculate cyclomatic complexity
    fn calculate_cyclomatic_complexity(&self, content: &str, file_type: &FileType) -> Result<u32> {
        let empty_vec = vec![];
        let patterns = self.cyclomatic_patterns.get(file_type).unwrap_or(&empty_vec);
        let mut complexity = 1; // Base complexity
        
        for pattern in patterns {
            let matches = pattern.pattern.find_iter(content).count();
            complexity += matches as u32 * pattern.complexity_weight;
        }
        
        Ok(complexity)
    }
    
    /// Calculate cognitive complexity
    fn calculate_cognitive_complexity(&self, content: &str, file_type: &FileType) -> Result<u32> {
        let empty_vec = vec![];
        let patterns = self.cognitive_patterns.get(file_type).unwrap_or(&empty_vec);
        let mut complexity = 0;
        let mut nesting_level = 0;
        
        for line in content.lines() {
            // Track nesting level
            if line.contains("{") && !line.contains("}") {
                nesting_level += 1;
            }
            if line.contains("}") && !line.contains("{") {
                nesting_level = nesting_level.saturating_sub(1);
            }
            
            // Apply patterns with nesting penalty
            for pattern in patterns {
                if pattern.pattern.is_match(line) {
                    complexity += pattern.cognitive_weight + (nesting_level * pattern.nesting_penalty);
                }
            }
        }
        
        Ok(complexity)
    }
    
    /// Calculate essential complexity (irreducible complexity)
    fn calculate_essential_complexity(&self, content: &str, _file_type: &FileType) -> Result<u32> {
        // Simplified essential complexity calculation
        let mut complexity = 1;
        
        // Count goto statements and unstructured control flow
        complexity += content.matches("goto").count() as u32;
        complexity += content.matches("break").count() as u32;
        complexity += content.matches("continue").count() as u32;
        complexity += content.matches("return").count() as u32;
        
        Ok(complexity)
    }
    
    /// Calculate design complexity (interface complexity)
    fn calculate_design_complexity(&self, content: &str, _file_type: &FileType) -> Result<u32> {
        // Simplified design complexity calculation
        let mut complexity = 0;
        
        // Count function calls and method invocations
        complexity += content.matches("(").count() as u32;
        complexity += content.matches(".").count() as u32;
        complexity += content.matches("->").count() as u32;
        complexity += content.matches("::").count() as u32;
        
        // Normalize by dividing by average calls per function
        complexity = complexity / 10; // Rough normalization
        
        Ok(complexity.max(1))
    }
    
    /// Create cyclomatic complexity patterns for different languages
    fn create_cyclomatic_patterns() -> HashMap<FileType, Vec<CyclomaticPattern>> {
        let mut patterns = HashMap::new();
        
        // Rust patterns
        patterns.insert(FileType::Rust, vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "If statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                complexity_weight: 1,
                description: "Else statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "While loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "For loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bmatch\b").unwrap(),
                complexity_weight: 1,
                description: "Match expression".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bloop\b").unwrap(),
                complexity_weight: 1,
                description: "Loop statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"=>").unwrap(),
                complexity_weight: 1,
                description: "Match arm".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\|\|").unwrap(),
                complexity_weight: 1,
                description: "Logical OR".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"&&").unwrap(),
                complexity_weight: 1,
                description: "Logical AND".to_string(),
            },
        ]);
        
        // JavaScript/TypeScript patterns
        let js_patterns = vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "If statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                complexity_weight: 1,
                description: "Else statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "While loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "For loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bswitch\b").unwrap(),
                complexity_weight: 1,
                description: "Switch statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bcase\b").unwrap(),
                complexity_weight: 1,
                description: "Case statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\btry\b").unwrap(),
                complexity_weight: 1,
                description: "Try statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bcatch\b").unwrap(),
                complexity_weight: 1,
                description: "Catch statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\|\|").unwrap(),
                complexity_weight: 1,
                description: "Logical OR".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"&&").unwrap(),
                complexity_weight: 1,
                description: "Logical AND".to_string(),
            },
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python patterns
        patterns.insert(FileType::Python, vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "If statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belif\b").unwrap(),
                complexity_weight: 1,
                description: "Elif statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                complexity_weight: 1,
                description: "Else statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "While loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "For loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\btry\b").unwrap(),
                complexity_weight: 1,
                description: "Try statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bexcept\b").unwrap(),
                complexity_weight: 1,
                description: "Except statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwith\b").unwrap(),
                complexity_weight: 1,
                description: "With statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\band\b").unwrap(),
                complexity_weight: 1,
                description: "Logical AND".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bor\b").unwrap(),
                complexity_weight: 1,
                description: "Logical OR".to_string(),
            },
        ]);
        
        patterns
    }
    
    /// Create cognitive complexity patterns for different languages
    fn create_cognitive_patterns() -> HashMap<FileType, Vec<CognitivePattern>> {
        let mut patterns = HashMap::new();
        
        // Rust patterns
        patterns.insert(FileType::Rust, vec![
            CognitivePattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "If statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "Else statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "While loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "For loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bmatch\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "Match expression".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bloop\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "Loop statement".to_string(),
            },
        ]);
        
        // JavaScript/TypeScript patterns
        let js_patterns = vec![
            CognitivePattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "If statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "Else statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "While loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "For loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bswitch\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "Switch statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\btry\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "Try statement".to_string(),
            },
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python patterns
        patterns.insert(FileType::Python, vec![
            CognitivePattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "If statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\belif\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "Elif statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\belse\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "Else statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "While loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "For loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\btry\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "Try statement".to_string(),
            },
        ]);
        
        patterns
    }
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 