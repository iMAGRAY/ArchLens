// Модуль обнаружения запахов кода

use crate::types::*;
use regex::Regex;
use std::collections::HashMap;

/// Обнаружитель запахов кода
#[derive(Debug, Clone)]
pub struct CodeSmell {
    pub smell_type: CodeSmellType,
    pub severity: Priority,
    pub description: String,
    pub suggestion: String,
    pub location: Option<String>,
    pub confidence: f32,
}

/// Типы запахов кода
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeSmellType {
    LongMethod,
    LongParameterList,
    MagicNumbers,
    LongLineLength,
    DeepNesting,
    DuplicatedCode,
    UnusedImports,
    UnusedVariables,
    MissingDocumentation,
    ComplexCondition,
    TooManyParameters,
    LargeClass,
    DeadCode,
    InconsistentNaming,
    TightCoupling,
    DataClump,
    FeatureEnvy,
    ShotgunSurgery,
    CommentedOutCode,
    PrimitiveObsession,
    EmptyExceptionHandling,
    TooManyComments,
    TooFewComments,
    HardcodedValues,
}

/// Детектор запахов кода
pub struct CodeSmellDetector {
    smell_rules: HashMap<CodeSmellType, Vec<SmellRule>>,
}

/// Правило для обнаружения запаха кода
#[derive(Debug, Clone)]
pub struct SmellRule {
    pub name: String,
    pub pattern: Regex,
    pub threshold: Option<f32>,
    pub severity: Priority,
    pub description: String,
    pub suggestion: String,
}

/// Детектор антипаттернов
#[derive(Debug)]
pub struct AntipatternDetector {
    pub pattern_name: String,
    pub detection_regex: Regex,
    pub severity: Priority,
    pub description: String,
}

impl CodeSmellDetector {
    pub fn new() -> Self {
        Self {
            smell_rules: Self::create_smell_rules(),
        }
    }
    
    pub fn detect_code_smells(&self, content: &str, file_type: FileType) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        for (smell_type, rules) in &self.smell_rules {
            for rule in rules {
                let detected_smells = self.apply_rule(content, smell_type, rule)?;
                smells.extend(detected_smells);
            }
        }
        
        // Добавляем специфичные для типа файла проверки
        smells.extend(self.detect_file_specific_smells(content, file_type)?);
        
        // Сортируем по убыванию важности
        smells.sort_by(|a, b| {
            let severity_order = |s: &Priority| match s {
                Priority::Critical => 4,
                Priority::High => 3,
                Priority::Medium => 2,
                Priority::Low => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });
        
        Ok(smells)
    }
    
    fn apply_rule(&self, content: &str, smell_type: &CodeSmellType, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        match smell_type {
            CodeSmellType::LongMethod => {
                smells.extend(self.detect_long_methods(content, rule)?);
            },
            CodeSmellType::LongParameterList => {
                smells.extend(self.detect_long_parameter_lists(content, rule)?);
            },
            CodeSmellType::LargeClass => {
                smells.extend(self.detect_large_classes(content, rule)?);
            },
            CodeSmellType::DuplicatedCode => {
                smells.extend(self.detect_duplicated_code(content, rule)?);
            },
            CodeSmellType::DeadCode => {
                smells.extend(self.detect_dead_code(content, rule)?);
            },
            CodeSmellType::LongLineLength => {
                smells.extend(self.detect_long_lines(content, rule)?);
            },
            CodeSmellType::DeepNesting => {
                smells.extend(self.detect_deep_nesting(content, rule)?);
            },
            CodeSmellType::MagicNumbers => {
                smells.extend(self.detect_magic_numbers(content, rule)?);
            },
            CodeSmellType::EmptyExceptionHandling => {
                smells.extend(self.detect_empty_exception_handling(content, rule)?);
            },
            CodeSmellType::TooManyComments => {
                smells.extend(self.detect_too_many_comments(content, rule)?);
            },
            CodeSmellType::TooFewComments => {
                smells.extend(self.detect_too_few_comments(content, rule)?);
            },
            CodeSmellType::HardcodedValues => {
                smells.extend(self.detect_hardcoded_values(content, rule)?);
            },
            _ => {
                // Базовая проверка по регулярному выражению
                let matches: Vec<_> = rule.pattern.captures_iter(content).collect();
                if !matches.is_empty() {
                    smells.push(CodeSmell {
                        smell_type: smell_type.clone(),
                        severity: rule.severity.clone(),
                        description: rule.description.clone(),
                        suggestion: rule.suggestion.clone(),
                        location: None,
                        confidence: 0.8,
                    });
                }
            }
        }
        
        Ok(smells)
    }
    
    fn detect_long_methods(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(20.0) as usize;
        
        // Ищем функции и считаем их длину
        let fn_pattern = Regex::new(r"fn\s+(\w+)\s*\([^)]*\)\s*\{").unwrap();
        
        for cap in fn_pattern.captures_iter(content) {
            let fn_name = cap.get(1).unwrap().as_str();
            let fn_start = cap.get(0).unwrap().start();
            
            // Подсчитываем строки функции (упрощенно)
            let lines_after = content[fn_start..].lines().take(100).count();
            
            if lines_after > threshold {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::LongMethod,
                    severity: rule.severity.clone(),
                    description: format!("Функция '{}' слишком длинная ({} строк)", fn_name, lines_after),
                    suggestion: format!("Разбейте функцию '{}' на несколько более мелких функций", fn_name),
                    location: Some(format!("Функция: {}", fn_name)),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_long_parameter_lists(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(4.0) as usize;
        
        let fn_pattern = Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)").unwrap();
        
        for cap in fn_pattern.captures_iter(content) {
            let fn_name = cap.get(1).unwrap().as_str();
            let params = cap.get(2).unwrap().as_str();
            
            let param_count = params.split(',').filter(|p| !p.trim().is_empty()).count();
            
            if param_count > threshold {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::LongParameterList,
                    severity: rule.severity.clone(),
                    description: format!("Функция '{}' имеет слишком много параметров ({})", fn_name, param_count),
                    suggestion: format!("Сгруппируйте параметры функции '{}' в структуру", fn_name),
                    location: Some(format!("Функция: {}", fn_name)),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_large_classes(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(200.0) as usize;
        
        let struct_pattern = Regex::new(r"struct\s+(\w+)").unwrap();
        let impl_pattern = Regex::new(r"impl\s+(\w+)").unwrap();
        
        for cap in struct_pattern.captures_iter(content) {
            let struct_name = cap.get(1).unwrap().as_str();
            
            // Ищем impl блок для этой структуры
            let impl_regex = Regex::new(&format!(r"impl\s+{}\s*\{{", struct_name)).unwrap();
            if let Some(impl_match) = impl_regex.find(content) {
                let impl_content = &content[impl_match.start()..];
                let impl_lines = impl_content.lines().take(500).count();
                
                if impl_lines > threshold {
                    smells.push(CodeSmell {
                        smell_type: CodeSmellType::LargeClass,
                        severity: rule.severity.clone(),
                        description: format!("Структура '{}' слишком большая ({} строк)", struct_name, impl_lines),
                        suggestion: format!("Разбейте структуру '{}' на несколько более мелких", struct_name),
                        location: Some(format!("Структура: {}", struct_name)),
                        confidence: 0.8,
                    });
                }
            }
        }
        
        Ok(smells)
    }
    
    fn detect_duplicated_code(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut duplicates = HashMap::new();
        
        // Ищем дублированные строки (игнорируем пустые и комментарии)
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") && trimmed.len() > 10 {
                duplicates.entry(trimmed).or_insert(Vec::new()).push(i + 1);
            }
        }
        
        for (line, positions) in duplicates {
            if positions.len() > 1 {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::DuplicatedCode,
                    severity: rule.severity.clone(),
                    description: format!("Дублированный код найден в строках: {:?}", positions),
                    suggestion: "Выделите повторяющийся код в отдельную функцию".to_string(),
                    location: Some(format!("Строки: {:?}", positions)),
                    confidence: 0.7,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_dead_code(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        // Ищем функции, которые не используются
        let fn_pattern = Regex::new(r"fn\s+(\w+)\s*\(").unwrap();
        let mut all_functions = Vec::new();
        
        for cap in fn_pattern.captures_iter(content) {
            let fn_name = cap.get(1).unwrap().as_str();
            if fn_name != "main" && fn_name != "new" && !fn_name.starts_with("test") {
                all_functions.push(fn_name);
            }
        }
        
        for fn_name in all_functions {
            let usage_pattern = Regex::new(&format!(r"\b{}\s*\(", fn_name)).unwrap();
            let usage_count = usage_pattern.find_iter(content).count();
            
            if usage_count <= 1 { // Только объявление
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::DeadCode,
                    severity: rule.severity.clone(),
                    description: format!("Функция '{}' не используется", fn_name),
                    suggestion: format!("Удалите неиспользуемую функцию '{}' или добавьте её использование", fn_name),
                    location: Some(format!("Функция: {}", fn_name)),
                    confidence: 0.6,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_long_lines(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(120.0) as usize;
        
        for (i, line) in content.lines().enumerate() {
            if line.len() > threshold {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::LongLineLength,
                    severity: rule.severity.clone(),
                    description: format!("Слишком длинная строка ({} символов)", line.len()),
                    suggestion: "Разбейте длинную строку на несколько коротких".to_string(),
                    location: Some(format!("Строка: {}", i + 1)),
                    confidence: 1.0,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_deep_nesting(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(4.0) as usize;
        let mut nesting_level: i32 = 0;
        let mut max_nesting = 0;
        
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.contains('{') {
                nesting_level += 1;
                max_nesting = max_nesting.max(nesting_level);
            }
            
            if trimmed.contains('}') {
                nesting_level = nesting_level.saturating_sub(1);
            }
            
            if nesting_level > threshold {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::DeepNesting,
                    severity: rule.severity.clone(),
                    description: format!("Глубокая вложенность ({} уровней)", nesting_level),
                    suggestion: "Выделите вложенную логику в отдельные функции".to_string(),
                    location: Some(format!("Строка: {}", i + 1)),
                    confidence: 0.8,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_magic_numbers(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let magic_pattern = Regex::new(r"\b(\d{2,})\b").unwrap();
        
        for cap in magic_pattern.captures_iter(content) {
            let number = cap.get(1).unwrap().as_str();
            if number != "0" && number != "1" && number != "100" {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::MagicNumbers,
                    severity: rule.severity.clone(),
                    description: format!("Магическое число: {}", number),
                    suggestion: format!("Замените число {} на именованную константу", number),
                    location: None,
                    confidence: 0.7,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_empty_exception_handling(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let empty_catch_pattern = Regex::new(r"catch\s*\([^)]*\)\s*\{\s*\}").unwrap();
        
        for _cap in empty_catch_pattern.captures_iter(content) {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::EmptyExceptionHandling,
                severity: rule.severity.clone(),
                description: "Пустой блок обработки исключений".to_string(),
                suggestion: "Добавьте логирование или обработку исключения".to_string(),
                location: None,
                confidence: 0.9,
            });
        }
        
        Ok(smells)
    }
    
    fn detect_too_many_comments(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(0.5);
        
        let total_lines = content.lines().count() as f32;
        let comment_lines = content.lines()
            .filter(|line| line.trim().starts_with("//"))
            .count() as f32;
        
        if total_lines > 0.0 && comment_lines / total_lines > threshold {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::TooManyComments,
                severity: rule.severity.clone(),
                description: format!("Слишком много комментариев ({:.1}%)", (comment_lines / total_lines) * 100.0),
                suggestion: "Упростите код, чтобы уменьшить необходимость в комментариях".to_string(),
                location: None,
                confidence: 0.6,
            });
        }
        
        Ok(smells)
    }
    
    fn detect_too_few_comments(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let threshold = rule.threshold.unwrap_or(0.1);
        
        let total_lines = content.lines().count() as f32;
        let comment_lines = content.lines()
            .filter(|line| line.trim().starts_with("//"))
            .count() as f32;
        
        if total_lines > 50.0 && comment_lines / total_lines < threshold {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::TooFewComments,
                severity: rule.severity.clone(),
                description: format!("Слишком мало комментариев ({:.1}%)", (comment_lines / total_lines) * 100.0),
                suggestion: "Добавьте комментарии для объяснения сложной логики".to_string(),
                location: None,
                confidence: 0.5,
            });
        }
        
        Ok(smells)
    }
    
    fn detect_hardcoded_values(&self, content: &str, rule: &SmellRule) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        let string_pattern = Regex::new(r#""([^"]{10,})""#).unwrap();
        
        for cap in string_pattern.captures_iter(content) {
            let value = cap.get(1).unwrap().as_str();
            if value.contains("http://") || value.contains("https://") || value.contains("localhost") {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::HardcodedValues,
                    severity: rule.severity.clone(),
                    description: format!("Жестко закодированное значение: {}", value),
                    suggestion: "Вынесите значение в конфигурационный файл".to_string(),
                    location: None,
                    confidence: 0.8,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_file_specific_smells(&self, content: &str, file_type: FileType) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        match file_type {
            FileType::Rust => {
                smells.extend(self.detect_rust_specific_smells(content)?);
            },
            FileType::JavaScript | FileType::TypeScript => {
                smells.extend(self.detect_js_specific_smells(content)?);
            },
            FileType::Python => {
                smells.extend(self.detect_python_specific_smells(content)?);
            },
            _ => {}
        }
        
        Ok(smells)
    }
    
    fn detect_rust_specific_smells(&self, content: &str) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        // Неиспользуемые импорты
        let unused_import_pattern = Regex::new(r"use\s+([^;]+);").unwrap();
        for cap in unused_import_pattern.captures_iter(content) {
            let import = cap.get(1).unwrap().as_str();
            let import_name = import.split("::").last().unwrap_or(import);
            
            if !content.contains(&format!("{}(", import_name)) && !content.contains(&format!("{}::", import_name)) {
                smells.push(CodeSmell {
                    smell_type: CodeSmellType::UnusedImports,
                    severity: Priority::Low,
                    description: format!("Неиспользуемый импорт: {}", import),
                    suggestion: format!("Удалите неиспользуемый импорт: {}", import),
                    location: None,
                    confidence: 0.6,
                });
            }
        }
        
        Ok(smells)
    }
    
    fn detect_js_specific_smells(&self, content: &str) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        // Использование var вместо let/const
        let var_pattern = Regex::new(r"\bvar\s+\w+").unwrap();
        for _cap in var_pattern.captures_iter(content) {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::PrimitiveObsession,
                severity: Priority::Medium,
                description: "Использование var вместо let/const".to_string(),
                suggestion: "Используйте let или const вместо var".to_string(),
                location: None,
                confidence: 0.9,
            });
        }
        
        Ok(smells)
    }
    
    fn detect_python_specific_smells(&self, content: &str) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        // Bare except
        let bare_except_pattern = Regex::new(r"except\s*:").unwrap();
        for _cap in bare_except_pattern.captures_iter(content) {
            smells.push(CodeSmell {
                smell_type: CodeSmellType::EmptyExceptionHandling,
                severity: Priority::High,
                description: "Использование bare except".to_string(),
                suggestion: "Укажите конкретный тип исключения".to_string(),
                location: None,
                confidence: 0.9,
            });
        }
        
        Ok(smells)
    }
    
    fn create_smell_rules() -> HashMap<CodeSmellType, Vec<SmellRule>> {
        let mut rules = HashMap::new();
        
        // Long Method
        rules.insert(CodeSmellType::LongMethod, vec![
            SmellRule {
                name: "Длинная функция".to_string(),
                pattern: Regex::new(r"fn\s+\w+").unwrap(),
                threshold: Some(25.0),
                severity: Priority::Medium,
                description: "Функция содержит слишком много строк кода".to_string(),
                suggestion: "Разбейте функцию на несколько более мелких".to_string(),
            },
        ]);
        
        // Long Parameter List
        rules.insert(CodeSmellType::LongParameterList, vec![
            SmellRule {
                name: "Длинный список параметров".to_string(),
                pattern: Regex::new(r"fn\s+\w+\s*\([^)]*\)").unwrap(),
                threshold: Some(5.0),
                severity: Priority::Medium,
                description: "Функция имеет слишком много параметров".to_string(),
                suggestion: "Сгруппируйте параметры в структуру".to_string(),
            },
        ]);
        
        // Magic Numbers
        rules.insert(CodeSmellType::MagicNumbers, vec![
            SmellRule {
                name: "Магические числа".to_string(),
                pattern: Regex::new(r"\b\d{2,}\b").unwrap(),
                threshold: None,
                severity: Priority::Low,
                description: "Использование магических чисел в коде".to_string(),
                suggestion: "Замените числа на именованные константы".to_string(),
            },
        ]);
        
        // Long Lines
        rules.insert(CodeSmellType::LongLineLength, vec![
            SmellRule {
                name: "Длинные строки".to_string(),
                pattern: Regex::new(r".{121,}").unwrap(),
                threshold: Some(120.0),
                severity: Priority::Low,
                description: "Строка превышает рекомендуемую длину".to_string(),
                suggestion: "Разбейте длинную строку на несколько коротких".to_string(),
            },
        ]);
        
        // Deep Nesting
        rules.insert(CodeSmellType::DeepNesting, vec![
            SmellRule {
                name: "Глубокая вложенность".to_string(),
                pattern: Regex::new(r"\{").unwrap(),
                threshold: Some(4.0),
                severity: Priority::Medium,
                description: "Слишком глубокая вложенность блоков кода".to_string(),
                suggestion: "Выделите вложенную логику в отдельные функции".to_string(),
            },
        ]);
        
        rules
    }
}

impl Default for CodeSmellDetector {
    fn default() -> Self {
        Self::new()
    }
} 