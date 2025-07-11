// Модуль анализа содержимого файлов

use crate::types::*;
use regex::Regex;
use std::path::Path;
use std::collections::HashMap;

/// Анализатор содержимого файлов
pub struct ContentAnalyzer {
    documentation_patterns: HashMap<FileType, Vec<Regex>>,
    test_patterns: HashMap<FileType, Regex>,
}

/// Результат анализа содержимого
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub documentation: Vec<String>,
    pub has_tests: bool,
    pub test_coverage_indicators: Vec<String>,
    pub code_quality_score: f32,
    pub file_metrics: FileMetrics,
}

/// Метрики файла
#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub function_count: usize,
    pub class_count: usize,
}

impl ContentAnalyzer {
    pub fn new() -> Self {
        Self {
            documentation_patterns: Self::create_documentation_patterns(),
            test_patterns: Self::create_test_patterns(),
        }
    }
    
    pub fn analyze_content(&self, content: &str, file_type: &FileType, file_path: &Path) -> Result<ContentAnalysis> {
        let documentation = self.extract_documentation(content, file_type);
        let has_tests = self.has_tests(content, file_type);
        let test_coverage_indicators = self.extract_test_coverage_indicators(content, file_type);
        let code_quality_score = self.calculate_code_quality_score(content, file_type);
        let file_metrics = self.calculate_file_metrics(content, file_type);
        
        Ok(ContentAnalysis {
            documentation,
            has_tests,
            test_coverage_indicators,
            code_quality_score,
            file_metrics,
        })
    }
    
    fn extract_documentation(&self, content: &str, file_type: &FileType) -> Vec<String> {
        let mut documentation = Vec::new();
        
        if let Some(patterns) = self.documentation_patterns.get(file_type) {
            for pattern in patterns {
                for cap in pattern.captures_iter(content) {
                    if let Some(doc) = cap.get(1) {
                        let doc_text = doc.as_str().trim();
                        if !doc_text.is_empty() && doc_text.len() > 10 {
                            documentation.push(doc_text.to_string());
                        }
                    }
                }
            }
        }
        
        documentation
    }
    
    fn has_tests(&self, content: &str, file_type: &FileType) -> bool {
        if let Some(pattern) = self.test_patterns.get(file_type) {
            pattern.is_match(content)
        } else {
            false
        }
    }
    
    fn extract_test_coverage_indicators(&self, content: &str, file_type: &FileType) -> Vec<String> {
        let mut indicators = Vec::new();
        
        match file_type {
            FileType::Rust => {
                let test_pattern = Regex::new(r"#\[test\]\s*fn\s+(\w+)").unwrap();
                for cap in test_pattern.captures_iter(content) {
                    if let Some(test_name) = cap.get(1) {
                        indicators.push(format!("Тест: {}", test_name.as_str()));
                    }
                }
                
                let bench_pattern = Regex::new(r"#\[bench\]\s*fn\s+(\w+)").unwrap();
                for cap in bench_pattern.captures_iter(content) {
                    if let Some(bench_name) = cap.get(1) {
                        indicators.push(format!("Бенчмарк: {}", bench_name.as_str()));
                    }
                }
            },
            FileType::JavaScript | FileType::TypeScript => {
                let test_pattern = Regex::new(r"(test|it|describe)\s*\(\s*[^\(\)]*").unwrap();
                for cap in test_pattern.captures_iter(content) {
                    if cap.len() > 1 {
                        indicators.push(format!("Тест: {}", cap.get(1).unwrap().as_str()));
                    }
                }
            },
            FileType::Python => {
                let test_pattern = Regex::new(r"def\s+(test_\w+)").unwrap();
                for cap in test_pattern.captures_iter(content) {
                    if let Some(test_name) = cap.get(1) {
                        indicators.push(format!("Тест: {}", test_name.as_str()));
                    }
                }
                
                let unittest_pattern = Regex::new(r"class\s+(\w*Test\w*)\s*\(").unwrap();
                for cap in unittest_pattern.captures_iter(content) {
                    if let Some(test_class) = cap.get(1) {
                        indicators.push(format!("Тест-класс: {}", test_class.as_str()));
                    }
                }
            },
            _ => {}
        }
        
        indicators
    }
    
    fn calculate_code_quality_score(&self, content: &str, file_type: &FileType) -> f32 {
        let mut score = 0.0;
        let mut factors = 0;
        
        // Фактор документации
        let doc_ratio = self.calculate_documentation_ratio(content, file_type);
        score += doc_ratio * 25.0;
        factors += 1;
        
        // Фактор тестирования
        if self.has_tests(content, file_type) {
            score += 25.0;
        }
        factors += 1;
        
        // Фактор дублирования кода
        let duplication_ratio = self.calculate_duplication_ratio(content);
        score += (1.0 - duplication_ratio) * 25.0;
        factors += 1;
        
        // Фактор сложности
        let complexity_score = self.calculate_complexity_factor(content, file_type);
        score += complexity_score * 25.0;
        factors += 1;
        
        if factors > 0 {
            score / factors as f32
        } else {
            0.0
        }
    }
    
    fn calculate_documentation_ratio(&self, content: &str, file_type: &FileType) -> f32 {
        let total_lines = content.lines().count() as f32;
        if total_lines == 0.0 {
            return 0.0;
        }
        
        let doc_lines = match file_type {
            FileType::Rust => {
                content.lines()
                    .filter(|line| line.trim().starts_with("///") || line.trim().starts_with("//!"))
                    .count() as f32
            },
            FileType::JavaScript | FileType::TypeScript => {
                let doc_pattern = Regex::new(r"/\*\*[\s\S]*?\*/").unwrap();
                doc_pattern.find_iter(content)
                    .map(|m| m.as_str().lines().count())
                    .sum::<usize>() as f32
            },
            FileType::Python => {
                let doc_pattern = Regex::new(r#""""[\s\S]*?"""#).unwrap();
                doc_pattern.find_iter(content)
                    .map(|m| m.as_str().lines().count())
                    .sum::<usize>() as f32
            },
            _ => 0.0,
        };
        
        doc_lines / total_lines
    }
    
    fn calculate_duplication_ratio(&self, content: &str) -> f32 {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len() as f32;
        
        if total_lines == 0.0 {
            return 0.0;
        }
        
        let mut line_counts = HashMap::new();
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.len() > 5 {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }
        
        let duplicated_lines = line_counts.values()
            .filter(|&&count| count > 1)
            .map(|&count| count - 1)
            .sum::<usize>() as f32;
        
        duplicated_lines / total_lines
    }
    
    fn calculate_complexity_factor(&self, content: &str, file_type: &FileType) -> f32 {
        let complexity_keywords = match file_type {
            FileType::Rust => vec!["if", "else", "for", "while", "match", "loop"],
            FileType::JavaScript | FileType::TypeScript => vec!["if", "else", "for", "while", "switch", "try", "catch"],
            FileType::Python => vec!["if", "elif", "else", "for", "while", "try", "except", "with"],
            _ => vec![],
        };
        
        let mut complexity_count = 0;
        for keyword in complexity_keywords {
            let pattern = Regex::new(&format!(r"\b{}\b", keyword)).unwrap();
            complexity_count += pattern.find_iter(content).count();
        }
        
        let total_lines = content.lines().count();
        if total_lines == 0 {
            return 1.0;
        }
        
        // Нормализуем сложность (меньше сложности - лучше)
        let complexity_ratio = complexity_count as f32 / total_lines as f32;
        (1.0 - complexity_ratio.min(1.0)).max(0.0)
    }
    
    fn calculate_file_metrics(&self, content: &str, file_type: &FileType) -> FileMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if self.is_comment_line(trimmed, file_type) {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }
        
        let function_count = self.count_functions(content, file_type);
        let class_count = self.count_classes(content, file_type);
        
        FileMetrics {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            function_count,
            class_count,
        }
    }
    
    fn is_comment_line(&self, line: &str, file_type: &FileType) -> bool {
        match file_type {
            FileType::Rust => line.starts_with("//") || line.starts_with("/*"),
            FileType::JavaScript | FileType::TypeScript => line.starts_with("//") || line.starts_with("/*"),
            FileType::Python => line.starts_with("#"),
            _ => false,
        }
    }
    
    fn count_functions(&self, content: &str, file_type: &FileType) -> usize {
        match file_type {
            FileType::Rust => {
                let pattern = Regex::new(r"fn\s+\w+").unwrap();
                pattern.find_iter(content).count()
            },
            FileType::JavaScript | FileType::TypeScript => {
                let pattern = Regex::new(r"function\s+\w+|const\s+\w+\s*=\s*\(|let\s+\w+\s*=\s*\(").unwrap();
                pattern.find_iter(content).count()
            },
            FileType::Python => {
                let pattern = Regex::new(r"def\s+\w+").unwrap();
                pattern.find_iter(content).count()
            },
            _ => 0,
        }
    }
    
    fn count_classes(&self, content: &str, file_type: &FileType) -> usize {
        match file_type {
            FileType::Rust => {
                let pattern = Regex::new(r"struct\s+\w+|enum\s+\w+|trait\s+\w+").unwrap();
                pattern.find_iter(content).count()
            },
            FileType::JavaScript | FileType::TypeScript => {
                let pattern = Regex::new(r"class\s+\w+").unwrap();
                pattern.find_iter(content).count()
            },
            FileType::Python => {
                let pattern = Regex::new(r"class\s+\w+").unwrap();
                pattern.find_iter(content).count()
            },
            _ => 0,
        }
    }
    
    fn create_documentation_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"///\s*(.+)").unwrap(),
            Regex::new(r"//!\s*(.+)").unwrap(),
            Regex::new(r"/\*\*\s*([\s\S]*?)\s*\*/").unwrap(),
        ]);
        
        // JavaScript/TypeScript
        let js_patterns = vec![
            Regex::new(r"/\*\*\s*([\s\S]*?)\s*\*/").unwrap(),
            Regex::new(r"//\s*(.+)").unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python
        patterns.insert(FileType::Python, vec![
            Regex::new(r#""""([\s\S]*?)""""#).unwrap(),
            Regex::new(r"#\s*(.+)").unwrap(),
        ]);
        
        patterns
    }
    
    fn create_test_patterns() -> HashMap<FileType, Regex> {
        let mut patterns = HashMap::new();
        
        patterns.insert(FileType::Rust, Regex::new(r"#\[test\]").unwrap());
        patterns.insert(FileType::JavaScript, Regex::new(r"(test|it|describe)\s*\(").unwrap());
        patterns.insert(FileType::TypeScript, Regex::new(r"(test|it|describe)\s*\(").unwrap());
        patterns.insert(FileType::Python, Regex::new(r"def\s+test_\w+|class\s+\w*Test\w*").unwrap());
        
        patterns
    }
}

impl Default for ContentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 