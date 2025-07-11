// Модуль метрик качества кода

use crate::types::*;
use crate::enrichment::semantic_analysis::SemanticLink;
use regex::Regex;
use std::collections::HashMap;

/// Метрики качества кода
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub maintainability_index: f32,
    pub cognitive_complexity: u32,
    pub cyclomatic_complexity: u32,
    pub technical_debt_ratio: f32,
    pub test_coverage_estimate: f32,
    pub documentation_completeness: f32,
    pub duplication_ratio: f32,
    pub lines_of_code: u32,
    pub comment_ratio: f32,
}

/// Вычислитель метрик качества
pub struct QualityMetricsCalculator {
    complexity_patterns: HashMap<FileType, Vec<Regex>>,
    comment_patterns: HashMap<FileType, Regex>,
    test_patterns: HashMap<FileType, Regex>,
}

impl QualityMetricsCalculator {
    pub fn new() -> Self {
        Self {
            complexity_patterns: Self::create_complexity_patterns(),
            comment_patterns: Self::create_comment_patterns(),
            test_patterns: Self::create_test_patterns(),
        }
    }
    
    pub fn calculate_quality_metrics(&self, content: &str, file_type: &FileType, semantic_links: &[SemanticLink]) -> Result<QualityMetrics> {
        let lines_of_code = self.count_lines_of_code(content);
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content, file_type);
        let cognitive_complexity = self.calculate_cognitive_complexity(content, &file_type);
        let comment_ratio = self.calculate_comment_ratio(content, file_type);
        let documentation_completeness = self.calculate_documentation_completeness(content, file_type);
        let test_coverage_estimate = self.estimate_test_coverage(content, file_type);
        let duplication_ratio = self.calculate_duplication_ratio(content);
        let technical_debt_ratio = self.calculate_technical_debt_ratio(content, semantic_links);
        let maintainability_index = self.calculate_maintainability_index(
            lines_of_code,
            cyclomatic_complexity,
            comment_ratio,
            duplication_ratio,
        );
        
        Ok(QualityMetrics {
            maintainability_index,
            cognitive_complexity,
            cyclomatic_complexity,
            technical_debt_ratio,
            test_coverage_estimate,
            documentation_completeness,
            duplication_ratio,
            lines_of_code,
            comment_ratio,
        })
    }
    
    fn count_lines_of_code(&self, content: &str) -> u32 {
        content.lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .count() as u32
    }
    
    fn calculate_cyclomatic_complexity(&self, content: &str, file_type: &FileType) -> u32 {
        let mut complexity = 1; // Базовая сложность
        
        if let Some(patterns) = self.complexity_patterns.get(file_type) {
            for pattern in patterns {
                complexity += pattern.find_iter(content).count() as u32;
            }
        }
        
        complexity
    }
    
    fn calculate_cognitive_complexity(&self, content: &str, _file_type: &FileType) -> u32 {
        let mut complexity = 0;
        let mut nesting_level: u32 = 0;
        
        // Простой алгоритм для подсчета когнитивной сложности
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Увеличиваем уровень вложенности
            if trimmed.contains('{') {
                nesting_level += 1;
            }
            
            // Уменьшаем уровень вложенности
            if trimmed.contains('}') {
                nesting_level = nesting_level.saturating_sub(1);
            }
            
            // Добавляем сложность для условий и циклов
            if trimmed.contains("if ") || trimmed.contains("else if ") {
                complexity += 1 + nesting_level;
            } else if trimmed.contains("for ") || trimmed.contains("while ") {
                complexity += 1 + nesting_level;
            } else if trimmed.contains("match ") || trimmed.contains("switch ") {
                complexity += 1 + nesting_level;
            } else if trimmed.contains("catch ") || trimmed.contains("except ") {
                complexity += 1 + nesting_level;
            }
        }
        
        complexity
    }
    
    fn calculate_comment_ratio(&self, content: &str, file_type: &FileType) -> f32 {
        let total_lines = content.lines().count() as f32;
        if total_lines == 0.0 {
            return 0.0;
        }
        
        let comment_lines = if let Some(pattern) = self.comment_patterns.get(file_type) {
            content.lines()
                .filter(|line| pattern.is_match(line.trim()))
                .count() as f32
        } else {
            0.0
        };
        
        comment_lines / total_lines
    }
    
    fn calculate_documentation_completeness(&self, content: &str, file_type: &FileType) -> f32 {
        let function_count = self.count_functions(content, file_type);
        let documented_functions = self.count_documented_functions(content, file_type);
        
        if function_count == 0 {
            return 1.0; // Если нет функций, то документация "полная"
        }
        
        documented_functions as f32 / function_count as f32
    }
    
    fn count_functions(&self, content: &str, file_type: &FileType) -> u32 {
        match file_type {
            FileType::Rust => {
                let fn_pattern = Regex::new(r"fn\s+\w+").unwrap();
                fn_pattern.find_iter(content).count() as u32
            },
            FileType::JavaScript | FileType::TypeScript => {
                let fn_pattern = Regex::new(r"function\s+\w+|const\s+\w+\s*=\s*\(").unwrap();
                fn_pattern.find_iter(content).count() as u32
            },
            FileType::Python => {
                let fn_pattern = Regex::new(r"def\s+\w+").unwrap();
                fn_pattern.find_iter(content).count() as u32
            },
            _ => 0,
        }
    }
    
    fn count_documented_functions(&self, content: &str, file_type: &FileType) -> u32 {
        match file_type {
            FileType::Rust => {
                let doc_pattern = Regex::new(r"///.*\n\s*fn\s+\w+").unwrap();
                doc_pattern.find_iter(content).count() as u32
            },
            FileType::JavaScript | FileType::TypeScript => {
                let doc_pattern = Regex::new(r"/\*\*.*?\*/\s*function\s+\w+").unwrap();
                doc_pattern.find_iter(content).count() as u32
            },
            FileType::Python => {
                let doc_pattern = Regex::new(r#"def\s+\w+[^:]*:\s*""".*?""""#).unwrap();
                doc_pattern.find_iter(content).count() as u32
            },
            _ => 0,
        }
    }
    
    fn estimate_test_coverage(&self, content: &str, file_type: &FileType) -> f32 {
        if let Some(pattern) = self.test_patterns.get(file_type) {
            let test_functions = pattern.find_iter(content).count();
            let total_functions = self.count_functions(content, file_type);
            
            if total_functions == 0 {
                return 0.0;
            }
            
            (test_functions as f32 / total_functions as f32).min(1.0)
        } else {
            0.0
        }
    }
    
    fn calculate_duplication_ratio(&self, content: &str) -> f32 {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len() as f32;
        
        if total_lines == 0.0 {
            return 0.0;
        }
        
        let mut duplicated_lines = 0;
        let mut line_counts = HashMap::new();
        
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }
        
        for (_, count) in line_counts {
            if count > 1 {
                duplicated_lines += count - 1;
            }
        }
        
        duplicated_lines as f32 / total_lines
    }
    
    fn calculate_technical_debt_ratio(&self, content: &str, semantic_links: &[SemanticLink]) -> f32 {
        let mut debt_score = 0.0;
        
        // Технический долг от TODO/FIXME/HACK
        let todo_pattern = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX|BUG)").unwrap();
        debt_score += todo_pattern.find_iter(content).count() as f32 * 0.1;
        
        // Технический долг от длинных строк
        let long_lines = content.lines()
            .filter(|line| line.len() > 100)
            .count() as f32;
        debt_score += long_lines * 0.05;
        
        // Технический долг от большого количества параметров
        let long_param_pattern = Regex::new(r"fn\s+\w+\s*\([^)]{50,}\)").unwrap();
        debt_score += long_param_pattern.find_iter(content).count() as f32 * 0.2;
        
        // Технический долг от слишком сложных связей
        let complex_links = semantic_links.iter()
            .filter(|link| link.strength > 1.0)
            .count() as f32;
        debt_score += complex_links * 0.1;
        
        // Нормализуем относительно размера файла
        let lines = content.lines().count() as f32;
        if lines > 0.0 {
            (debt_score / lines).min(1.0)
        } else {
            0.0
        }
    }
    
    fn calculate_maintainability_index(&self, lines_of_code: u32, cyclomatic_complexity: u32, comment_ratio: f32, duplication_ratio: f32) -> f32 {
        // Формула индекса поддерживаемости (упрощенная версия)
        let volume = (lines_of_code as f32).ln();
        let complexity_penalty = cyclomatic_complexity as f32 * 0.23;
        let comment_bonus = comment_ratio * 16.2;
        let duplication_penalty = duplication_ratio * 50.0;
        
        let index = 171.0 - volume * 5.2 - complexity_penalty + comment_bonus - duplication_penalty;
        
        // Ограничиваем значение от 0 до 100
        index.max(0.0).min(100.0)
    }
    
    fn create_complexity_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"\bif\b").unwrap(),
            Regex::new(r"\belse\s+if\b").unwrap(),
            Regex::new(r"\bfor\b").unwrap(),
            Regex::new(r"\bwhile\b").unwrap(),
            Regex::new(r"\bmatch\b").unwrap(),
            Regex::new(r"\?\s*\{").unwrap(),
            Regex::new(r"\|\s*\w+\s*\|").unwrap(), // Closures
        ]);
        
        // JavaScript/TypeScript
        let js_patterns = vec![
            Regex::new(r"\bif\b").unwrap(),
            Regex::new(r"\belse\s+if\b").unwrap(),
            Regex::new(r"\bfor\b").unwrap(),
            Regex::new(r"\bwhile\b").unwrap(),
            Regex::new(r"\bswitch\b").unwrap(),
            Regex::new(r"\btry\b").unwrap(),
            Regex::new(r"\bcatch\b").unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python
        patterns.insert(FileType::Python, vec![
            Regex::new(r"\bif\b").unwrap(),
            Regex::new(r"\belif\b").unwrap(),
            Regex::new(r"\bfor\b").unwrap(),
            Regex::new(r"\bwhile\b").unwrap(),
            Regex::new(r"\btry\b").unwrap(),
            Regex::new(r"\bexcept\b").unwrap(),
            Regex::new(r"\bwith\b").unwrap(),
        ]);
        
        patterns
    }
    
    fn create_comment_patterns() -> HashMap<FileType, Regex> {
        let mut patterns = HashMap::new();
        
        patterns.insert(FileType::Rust, Regex::new(r"^\s*//").unwrap());
        patterns.insert(FileType::JavaScript, Regex::new(r"^\s*//").unwrap());
        patterns.insert(FileType::TypeScript, Regex::new(r"^\s*//").unwrap());
        patterns.insert(FileType::Python, Regex::new(r"^\s*#").unwrap());
        
        patterns
    }
    
    fn create_test_patterns() -> HashMap<FileType, Regex> {
        let mut patterns = HashMap::new();
        
        patterns.insert(FileType::Rust, Regex::new(r"#\[test\]").unwrap());
        patterns.insert(FileType::JavaScript, Regex::new(r"(test|it|describe)\s*\(").unwrap());
        patterns.insert(FileType::TypeScript, Regex::new(r"(test|it|describe)\s*\(").unwrap());
        patterns.insert(FileType::Python, Regex::new(r"def\s+test_\w+").unwrap());
        
        patterns
    }
}

impl Default for QualityMetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 