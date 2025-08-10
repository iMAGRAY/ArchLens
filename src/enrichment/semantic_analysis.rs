// Модуль семантического анализа кода

use crate::types::*;
use regex::Regex;
use std::collections::HashMap;
// use std::path::Path;

/// Семантическая связь между элементами
#[derive(Debug, Clone)]
pub struct SemanticLink {
    pub link_type: SemanticLinkType,
    pub target_name: String,
    pub strength: f32,
    pub context: String,
}

/// Типы семантических связей
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

/// Семантический анализатор для конкретного языка
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    pub language: FileType,
    pub method_call_patterns: Vec<Regex>,
    pub field_access_patterns: Vec<Regex>,
    pub inheritance_patterns: Vec<Regex>,
    pub composition_patterns: Vec<Regex>,
    pub complexity_patterns: Vec<Regex>,
}

/// Результат семантического анализа
#[derive(Debug, Clone)]
pub struct SemanticAnalysisResult {
    pub semantic_links: Vec<SemanticLink>,
    pub complexity_score: f32,
    pub abstraction_level: f32,
}

pub struct SemanticAnalysisEngine {
    analyzers: HashMap<FileType, SemanticAnalyzer>,
}

impl SemanticAnalysisEngine {
    pub fn new() -> Self {
        Self {
            analyzers: Self::create_analyzers(),
        }
    }

    pub fn analyze(&self, content: &str, file_type: FileType) -> Result<SemanticAnalysisResult> {
        let analyzer = self
            .analyzers
            .get(&file_type)
            .ok_or_else(|| format!("Анализатор для типа файла {:?} не найден", file_type))?;

        let semantic_links = self.extract_semantic_links(content, analyzer)?;
        let complexity_score = self.calculate_complexity_score(content, analyzer);
        let abstraction_level = self.calculate_abstraction_level(content, &semantic_links);

        Ok(SemanticAnalysisResult {
            semantic_links,
            complexity_score,
            abstraction_level,
        })
    }

    fn extract_semantic_links(
        &self,
        content: &str,
        analyzer: &SemanticAnalyzer,
    ) -> Result<Vec<SemanticLink>> {
        let mut links = Vec::new();

        // Извлекаем вызовы методов
        for pattern in &analyzer.method_call_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(method_name) = cap.get(1) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::MethodCall,
                        target_name: method_name.as_str().to_string(),
                        strength: 1.0,
                        context: cap
                            .get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        // Извлекаем доступ к полям
        for pattern in &analyzer.field_access_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(field_name) = cap.get(1) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::FieldAccess,
                        target_name: field_name.as_str().to_string(),
                        strength: 0.8,
                        context: cap
                            .get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        // Извлекаем наследование
        for pattern in &analyzer.inheritance_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(parent_name) = cap.get(1) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Inheritance,
                        target_name: parent_name.as_str().to_string(),
                        strength: 1.5,
                        context: cap
                            .get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        // Извлекаем композицию
        for pattern in &analyzer.composition_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(composed_name) = cap.get(1) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Composition,
                        target_name: composed_name.as_str().to_string(),
                        strength: 1.2,
                        context: cap
                            .get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        Ok(links)
    }

    fn calculate_complexity_score(&self, content: &str, analyzer: &SemanticAnalyzer) -> f32 {
        let mut score = 0.0;

        for pattern in &analyzer.complexity_patterns {
            score += pattern.captures_iter(content).count() as f32;
        }

        // Нормализуем относительно размера файла
        let lines = content.lines().count() as f32;
        if lines > 0.0 {
            score / lines
        } else {
            0.0
        }
    }

    fn calculate_abstraction_level(&self, _content: &str, semantic_links: &[SemanticLink]) -> f32 {
        let total_links = semantic_links.len() as f32;
        if total_links == 0.0 {
            return 0.0;
        }

        let high_level_links = semantic_links
            .iter()
            .filter(|link| {
                matches!(
                    link.link_type,
                    SemanticLinkType::Inheritance
                        | SemanticLinkType::Composition
                        | SemanticLinkType::Association
                )
            })
            .count() as f32;

        high_level_links / total_links
    }

    fn create_analyzers() -> HashMap<FileType, SemanticAnalyzer> {
        let mut analyzers = HashMap::new();

        // Rust анализатор
        analyzers.insert(
            FileType::Rust,
            SemanticAnalyzer {
                language: FileType::Rust,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\s*\(").unwrap(),
                    Regex::new(r"\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"::(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"\.(\w+)").unwrap(),
                    Regex::new(r"self\.(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![
                    Regex::new(r"impl\s+(\w+)\s+for").unwrap(),
                    Regex::new(r"trait\s+(\w+)").unwrap(),
                ],
                composition_patterns: vec![
                    Regex::new(r"struct\s+\w+\s*\{[^}]*(\w+):\s*\w+").unwrap()
                ],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\bmatch\b").unwrap(),
                    Regex::new(r"\?\s*\{").unwrap(),
                ],
            },
        );

        // JavaScript/TypeScript анализатор
        let js_analyzer = SemanticAnalyzer {
            language: FileType::JavaScript,
            method_call_patterns: vec![
                Regex::new(r"(\w+)\s*\(").unwrap(),
                Regex::new(r"\.(\w+)\s*\(").unwrap(),
            ],
            field_access_patterns: vec![
                Regex::new(r"\.(\w+)").unwrap(),
                Regex::new(r"this\.(\w+)").unwrap(),
            ],
            inheritance_patterns: vec![
                Regex::new(r"extends\s+(\w+)").unwrap(),
                Regex::new(r"implements\s+(\w+)").unwrap(),
            ],
            composition_patterns: vec![Regex::new(r"new\s+(\w+)\s*\(").unwrap()],
            complexity_patterns: vec![
                Regex::new(r"\bif\b").unwrap(),
                Regex::new(r"\bfor\b").unwrap(),
                Regex::new(r"\bwhile\b").unwrap(),
                Regex::new(r"\bswitch\b").unwrap(),
                Regex::new(r"\btry\b").unwrap(),
            ],
        };

        analyzers.insert(FileType::JavaScript, js_analyzer.clone());
        analyzers.insert(FileType::TypeScript, js_analyzer);

        // Python анализатор
        analyzers.insert(
            FileType::Python,
            SemanticAnalyzer {
                language: FileType::Python,
                method_call_patterns: vec![
                    Regex::new(r"(\w+)\s*\(").unwrap(),
                    Regex::new(r"\.(\w+)\s*\(").unwrap(),
                    Regex::new(r"self\.(\w+)\s*\(").unwrap(),
                ],
                field_access_patterns: vec![
                    Regex::new(r"\.(\w+)").unwrap(),
                    Regex::new(r"self\.(\w+)").unwrap(),
                ],
                inheritance_patterns: vec![Regex::new(r"class\s+\w+\s*\(\s*(\w+)\s*\)").unwrap()],
                composition_patterns: vec![Regex::new(r"(\w+)\s*=\s*\w+\s*\(").unwrap()],
                complexity_patterns: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                    Regex::new(r"\btry\b").unwrap(),
                    Regex::new(r"\bexcept\b").unwrap(),
                ],
            },
        );

        analyzers
    }
}

impl Default for SemanticAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
