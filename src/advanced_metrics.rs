// Продвинутые метрики для анализа качества кода и архитектуры

use crate::types::*;
use std::collections::HashMap;
use regex::Regex;
use serde::{Serialize, Deserialize};
use crate::types::Result;

/// Калькулятор продвинутых метрик
#[derive(Debug)]
pub struct AdvancedMetricsCalculator {
    cyclomatic_patterns: HashMap<FileType, Vec<CyclomaticPattern>>,
    cognitive_patterns: HashMap<FileType, Vec<CognitivePattern>>,
    solid_analyzers: HashMap<FileType, SOLIDAnalyzer>,
}

/// Паттерн для расчета цикломатической сложности
#[derive(Debug, Clone)]
pub struct CyclomaticPattern {
    pub pattern: Regex,
    pub complexity_weight: u32,
    pub description: String,
}

/// Паттерн для расчета когнитивной сложности
#[derive(Debug, Clone)]
pub struct CognitivePattern {
    pub pattern: Regex,
    pub cognitive_weight: u32,
    pub nesting_penalty: u32,
    pub description: String,
}

/// Анализатор SOLID принципов
#[derive(Debug, Clone)]
pub struct SOLIDAnalyzer {
    pub single_responsibility: SRPAnalyzer,
    pub open_closed: OCPAnalyzer,
    pub liskov_substitution: LSPAnalyzer,
    pub interface_segregation: ISPAnalyzer,
    pub dependency_inversion: DIPAnalyzer,
}

/// Анализатор принципа Single Responsibility
#[derive(Debug, Clone)]
pub struct SRPAnalyzer {
    pub responsibility_indicators: Vec<Regex>,
    pub max_responsibilities: u32,
}

/// Анализатор принципа Open/Closed
#[derive(Debug, Clone)]
pub struct OCPAnalyzer {
    pub modification_indicators: Vec<Regex>,
    pub extension_indicators: Vec<Regex>,
}

/// Анализатор принципа Liskov Substitution
#[derive(Debug, Clone)]
pub struct LSPAnalyzer {
    pub substitution_violations: Vec<Regex>,
    pub behavioral_change_indicators: Vec<Regex>,
}

/// Анализатор принципа Interface Segregation
#[derive(Debug, Clone)]
pub struct ISPAnalyzer {
    pub interface_size_indicators: Vec<Regex>,
    pub max_interface_methods: u32,
}

/// Анализатор принципа Dependency Inversion
#[derive(Debug, Clone)]
pub struct DIPAnalyzer {
    pub concrete_dependency_indicators: Vec<Regex>,
    pub abstraction_indicators: Vec<Regex>,
}

/// Результат расчета продвинутых метрик
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub maintainability_index: f32,
    pub solid_score: SOLIDScore,
    pub code_quality_index: f32,
    pub technical_debt_ratio: f32,
    pub halstead_metrics: HalsteadMetrics,
}

/// Оценка соответствия SOLID принципам
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOLIDScore {
    pub srp_score: f32,
    pub ocp_score: f32,
    pub lsp_score: f32,
    pub isp_score: f32,
    pub dip_score: f32,
    pub overall_score: f32,
}

/// Метрики Холстеда
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    pub vocabulary: u32,
    pub length: u32,
    pub volume: f32,
    pub difficulty: f32,
    pub effort: f32,
    pub time: f32,
    pub bugs: f32,
}

impl AdvancedMetricsCalculator {
    pub fn new() -> Self {
        Self {
            cyclomatic_patterns: Self::create_cyclomatic_patterns(),
            cognitive_patterns: Self::create_cognitive_patterns(),
            solid_analyzers: Self::create_solid_analyzers(),
        }
    }
    
    /// Расчет продвинутых метрик для капсулы
    pub fn calculate_metrics(&self, capsule: &Capsule, content: &str) -> Result<AdvancedMetrics> {
        let file_type = self.determine_file_type(&capsule.file_path);
        
        // Расчет цикломатической сложности
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content, &file_type)?;
        
        // Расчет когнитивной сложности
        let cognitive_complexity = self.calculate_cognitive_complexity(content, &file_type)?;
        
        // Расчет SOLID оценки
        let solid_score = self.calculate_solid_score(content, &file_type)?;
        
        // Расчет метрик Холстеда
        let halstead_metrics = self.calculate_halstead_metrics(content, &file_type)?;
        
        // Расчет индекса сопровождаемости
        let maintainability_index = self.calculate_maintainability_index(
            cyclomatic_complexity,
            halstead_metrics.volume,
            content.lines().count() as u32,
        )?;
        
        // Расчет индекса качества кода
        let code_quality_index = self.calculate_code_quality_index(
            cyclomatic_complexity,
            cognitive_complexity,
            &solid_score,
            maintainability_index,
        )?;
        
        // Расчет коэффициента технического долга
        let technical_debt_ratio = self.calculate_technical_debt_ratio(
            &solid_score,
            cyclomatic_complexity,
            cognitive_complexity,
        )?;
        
        Ok(AdvancedMetrics {
            cyclomatic_complexity,
            cognitive_complexity,
            maintainability_index,
            solid_score,
            code_quality_index,
            technical_debt_ratio,
            halstead_metrics,
        })
    }
    
    /// Расчет цикломатической сложности
    fn calculate_cyclomatic_complexity(&self, content: &str, file_type: &FileType) -> Result<u32> {
        let empty_vec = vec![];
        let patterns = self.cyclomatic_patterns.get(file_type).unwrap_or(&empty_vec);
        let mut complexity = 1; // Базовая сложность
        
        for pattern in patterns {
            let matches = pattern.pattern.find_iter(content).count();
            complexity += matches as u32 * pattern.complexity_weight;
        }
        
        Ok(complexity)
    }
    
    /// Расчет когнитивной сложности
    fn calculate_cognitive_complexity(&self, content: &str, file_type: &FileType) -> Result<u32> {
        let empty_vec = vec![];
        let patterns = self.cognitive_patterns.get(file_type).unwrap_or(&empty_vec);
        let mut complexity = 0;
        
        let lines: Vec<&str> = content.lines().collect();
        let mut nesting_level: u32 = 0;
        
        for line in lines {
            // Определение уровня вложенности
            let indent_level = line.len() - line.trim_start().len();
            nesting_level = (indent_level / 4) as u32; // Предполагаем отступ в 4 пробела
            
            for pattern in patterns {
                if pattern.pattern.is_match(line) {
                    complexity += pattern.cognitive_weight + (nesting_level * pattern.nesting_penalty);
                }
            }
        }
        
        Ok(complexity)
    }
    
    /// Расчет SOLID оценки
    fn calculate_solid_score(&self, content: &str, file_type: &FileType) -> Result<SOLIDScore> {
        let default_analyzer = SOLIDAnalyzer::default();
        let analyzer = self.solid_analyzers.get(file_type).unwrap_or(&default_analyzer);
        
        let srp_score = self.calculate_srp_score(content, &analyzer.single_responsibility)?;
        let ocp_score = self.calculate_ocp_score(content, &analyzer.open_closed)?;
        let lsp_score = self.calculate_lsp_score(content, &analyzer.liskov_substitution)?;
        let isp_score = self.calculate_isp_score(content, &analyzer.interface_segregation)?;
        let dip_score = self.calculate_dip_score(content, &analyzer.dependency_inversion)?;
        
        let overall_score = (srp_score + ocp_score + lsp_score + isp_score + dip_score) / 5.0;
        
        Ok(SOLIDScore {
            srp_score,
            ocp_score,
            lsp_score,
            isp_score,
            dip_score,
            overall_score,
        })
    }
    
    /// Расчет метрик Холстеда
    fn calculate_halstead_metrics(&self, content: &str, file_type: &FileType) -> Result<HalsteadMetrics> {
        let (operators, operands) = self.extract_operators_and_operands(content, file_type)?;
        
        let n1 = operators.len() as u32; // Количество уникальных операторов
        let n2 = operands.len() as u32; // Количество уникальных операндов
        let big_n1 = operators.values().sum::<u32>(); // Общее количество операторов
        let big_n2 = operands.values().sum::<u32>(); // Общее количество операндов
        
        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;
        let volume = length as f32 * (vocabulary as f32).log2();
        let difficulty = (n1 as f32 / 2.0) * (big_n2 as f32 / n2 as f32);
        let effort = difficulty * volume;
        let time = effort / 18.0; // Секунды
        let bugs = volume / 3000.0; // Предполагаемое количество ошибок
        
        Ok(HalsteadMetrics {
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        })
    }
    
    /// Расчет индекса сопровождаемости
    fn calculate_maintainability_index(&self, cyclomatic: u32, volume: f32, lines: u32) -> Result<f32> {
        // Формула индекса сопровождаемости
        let mi = 171.0 
            - 5.2 * (volume).ln() 
            - 0.23 * (cyclomatic as f32) 
            - 16.2 * (lines as f32).ln();
        
        Ok(mi.max(0.0).min(100.0))
    }
    
    /// Расчет индекса качества кода
    fn calculate_code_quality_index(&self, cyclomatic: u32, cognitive: u32, solid: &SOLIDScore, maintainability: f32) -> Result<f32> {
        let complexity_score = 100.0 - (cyclomatic as f32 * 2.0 + cognitive as f32 * 1.5).min(100.0);
        let solid_score = solid.overall_score * 100.0;
        let maintainability_score = maintainability;
        
        let quality_index = (complexity_score + solid_score + maintainability_score) / 3.0;
        
        Ok(quality_index.max(0.0).min(100.0))
    }
    
    /// Расчет коэффициента технического долга
    fn calculate_technical_debt_ratio(&self, solid: &SOLIDScore, cyclomatic: u32, cognitive: u32) -> Result<f32> {
        let complexity_penalty = (cyclomatic as f32 - 10.0).max(0.0) * 0.1;
        let cognitive_penalty = (cognitive as f32 - 15.0).max(0.0) * 0.15;
        let solid_penalty = (100.0 - solid.overall_score * 100.0) * 0.01;
        
        let debt_ratio = (complexity_penalty + cognitive_penalty + solid_penalty) / 3.0;
        
        Ok(debt_ratio.max(0.0).min(1.0))
    }
    
    /// Создание паттернов для цикломатической сложности
    fn create_cyclomatic_patterns() -> HashMap<FileType, Vec<CyclomaticPattern>> {
        let mut patterns = HashMap::new();
        
        // Rust паттерны
        patterns.insert(FileType::Rust, vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "if statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belse\s+if\b").unwrap(),
                complexity_weight: 1,
                description: "else if statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "while loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "for loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bmatch\b").unwrap(),
                complexity_weight: 1,
                description: "match expression".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"=>\s*[^,\}]").unwrap(),
                complexity_weight: 1,
                description: "match arm".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\&\&|\|\|").unwrap(),
                complexity_weight: 1,
                description: "logical operator".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\?").unwrap(),
                complexity_weight: 1,
                description: "try operator".to_string(),
            },
        ]);
        
        // JavaScript/TypeScript паттерны
        let js_patterns = vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "if statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belse\s+if\b").unwrap(),
                complexity_weight: 1,
                description: "else if statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "while loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "for loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bswitch\b").unwrap(),
                complexity_weight: 1,
                description: "switch statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bcase\b").unwrap(),
                complexity_weight: 1,
                description: "case statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bcatch\b").unwrap(),
                complexity_weight: 1,
                description: "catch statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\&\&|\|\|").unwrap(),
                complexity_weight: 1,
                description: "logical operator".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\?").unwrap(),
                complexity_weight: 1,
                description: "ternary operator".to_string(),
            },
        ];
        
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python паттерны
        patterns.insert(FileType::Python, vec![
            CyclomaticPattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                complexity_weight: 1,
                description: "if statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\belif\b").unwrap(),
                complexity_weight: 1,
                description: "elif statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bwhile\b").unwrap(),
                complexity_weight: 1,
                description: "while loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bfor\b").unwrap(),
                complexity_weight: 1,
                description: "for loop".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\bexcept\b").unwrap(),
                complexity_weight: 1,
                description: "except statement".to_string(),
            },
            CyclomaticPattern {
                pattern: Regex::new(r"\band\b|\bor\b").unwrap(),
                complexity_weight: 1,
                description: "logical operator".to_string(),
            },
        ]);
        
        patterns
    }
    
    /// Создание паттернов для когнитивной сложности
    fn create_cognitive_patterns() -> HashMap<FileType, Vec<CognitivePattern>> {
        let mut patterns = HashMap::new();
        
        // Rust паттерны
        patterns.insert(FileType::Rust, vec![
            CognitivePattern {
                pattern: Regex::new(r"\bif\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "if statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\belse\s+if\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "else if statement".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bloop\b|\bwhile\b|\bfor\b").unwrap(),
                cognitive_weight: 2,
                nesting_penalty: 1,
                description: "loop".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bmatch\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 1,
                description: "match expression".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\&\&|\|\|").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "logical operator".to_string(),
            },
            CognitivePattern {
                pattern: Regex::new(r"\bbreak\b|\bcontinue\b").unwrap(),
                cognitive_weight: 1,
                nesting_penalty: 0,
                description: "jump statement".to_string(),
            },
        ]);
        
        patterns
    }
    
    /// Создание SOLID анализаторов
    fn create_solid_analyzers() -> HashMap<FileType, SOLIDAnalyzer> {
        let mut analyzers = HashMap::new();
        
        // Rust анализатор
        analyzers.insert(FileType::Rust, SOLIDAnalyzer {
            single_responsibility: SRPAnalyzer {
                responsibility_indicators: vec![
                    Regex::new(r"\bimpl\b.*{.*\bfn\b.*\bfn\b.*\bfn\b").unwrap(),
                    Regex::new(r"\bstruct\b.*{.*field.*field.*field.*field.*field").unwrap(),
                ],
                max_responsibilities: 3,
            },
            open_closed: OCPAnalyzer {
                modification_indicators: vec![
                    Regex::new(r"\bmatch\b.*\btype\b.*=>.*\bmatch\b.*\btype\b.*=>").unwrap(),
                    Regex::new(r"\bif\b.*\btype\b.*\belse\b.*\bif\b.*\btype\b").unwrap(),
                ],
                extension_indicators: vec![
                    Regex::new(r"\btrait\b").unwrap(),
                    Regex::new(r"\bimpl\b.*\bfor\b").unwrap(),
                ],
            },
            liskov_substitution: LSPAnalyzer {
                substitution_violations: vec![
                    Regex::new(r"\bpanic!\b").unwrap(),
                    Regex::new(r"\bunreachable!\b").unwrap(),
                ],
                behavioral_change_indicators: vec![
                    Regex::new(r"\boverride\b.*\bthrow\b").unwrap(),
                ],
            },
            interface_segregation: ISPAnalyzer {
                interface_size_indicators: vec![
                    Regex::new(r"\btrait\b.*{.*\bfn\b.*\bfn\b.*\bfn\b.*\bfn\b.*\bfn\b").unwrap(),
                ],
                max_interface_methods: 5,
            },
            dependency_inversion: DIPAnalyzer {
                concrete_dependency_indicators: vec![
                    Regex::new(r"\bnew\b.*\bConcrete").unwrap(),
                    Regex::new(r"\bstruct\b.*{.*field:.*Concrete").unwrap(),
                ],
                abstraction_indicators: vec![
                    Regex::new(r"\btrait\b").unwrap(),
                    Regex::new(r"\bBox<dyn\b").unwrap(),
                ],
            },
        });
        
        analyzers
    }
    
    // Остальные вспомогательные методы
    fn determine_file_type(&self, path: &std::path::Path) -> FileType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => FileType::Rust,
            Some("js") => FileType::JavaScript,
            Some("ts") => FileType::TypeScript,
            Some("py") => FileType::Python,
            Some("java") => FileType::Java,
            Some("go") => FileType::Go,
            Some("cpp") | Some("cc") | Some("cxx") => FileType::Cpp,
            Some("c") => FileType::C,
            Some(ext) => FileType::Other(ext.to_string()),
            None => FileType::Other("unknown".to_string()),
        }
    }
    
    // Вспомогательные методы для расчета SOLID оценок
    fn calculate_srp_score(&self, content: &str, analyzer: &SRPAnalyzer) -> Result<f32> {
        let mut violations = 0;
        
        for pattern in &analyzer.responsibility_indicators {
            violations += pattern.find_iter(content).count();
        }
        
        let score = if violations == 0 {
            1.0
        } else {
            1.0 - (violations as f32 / analyzer.max_responsibilities as f32).min(1.0)
        };
        
        Ok(score)
    }
    
    fn calculate_ocp_score(&self, content: &str, analyzer: &OCPAnalyzer) -> Result<f32> {
        let mut modifications = 0;
        let mut extensions = 0;
        
        for pattern in &analyzer.modification_indicators {
            modifications += pattern.find_iter(content).count();
        }
        
        for pattern in &analyzer.extension_indicators {
            extensions += pattern.find_iter(content).count();
        }
        
        let score = if modifications == 0 {
            1.0
        } else if extensions > modifications {
            0.8
        } else {
            0.3
        };
        
        Ok(score)
    }
    
    fn calculate_lsp_score(&self, content: &str, analyzer: &LSPAnalyzer) -> Result<f32> {
        let mut violations = 0;
        
        for pattern in &analyzer.substitution_violations {
            violations += pattern.find_iter(content).count();
        }
        
        for pattern in &analyzer.behavioral_change_indicators {
            violations += pattern.find_iter(content).count();
        }
        
        let score = if violations == 0 {
            1.0
        } else {
            1.0 - (violations as f32 * 0.2).min(1.0)
        };
        
        Ok(score)
    }
    
    fn calculate_isp_score(&self, content: &str, analyzer: &ISPAnalyzer) -> Result<f32> {
        let mut violations = 0;
        
        for pattern in &analyzer.interface_size_indicators {
            violations += pattern.find_iter(content).count();
        }
        
        let score = if violations == 0 {
            1.0
        } else {
            1.0 - (violations as f32 * 0.3).min(1.0)
        };
        
        Ok(score)
    }
    
    fn calculate_dip_score(&self, content: &str, analyzer: &DIPAnalyzer) -> Result<f32> {
        let mut concrete_deps = 0;
        let mut abstractions = 0;
        
        for pattern in &analyzer.concrete_dependency_indicators {
            concrete_deps += pattern.find_iter(content).count();
        }
        
        for pattern in &analyzer.abstraction_indicators {
            abstractions += pattern.find_iter(content).count();
        }
        
        let score = if concrete_deps == 0 {
            1.0
        } else if abstractions > concrete_deps {
            0.8
        } else {
            0.4
        };
        
        Ok(score)
    }
    
    fn extract_operators_and_operands(&self, content: &str, _file_type: &FileType) -> Result<(HashMap<String, u32>, HashMap<String, u32>)> {
        let mut operators = HashMap::new();
        let mut operands = HashMap::new();
        
        // Простое извлечение операторов и операндов
        let operator_patterns = vec![
            r"\+", r"-", r"\*", r"/", r"=", r"==", r"!=", r"<", r">", r"<=", r">=",
            r"&&", r"\|\|", r"!", r"&", r"\|", r"\^", r"<<", r">>", r"%",
            r"\(", r"\)", r"\[", r"\]", r"\{", r"\}", r";", r",", r"\."
        ];
        
        for pattern in operator_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                let count = regex.find_iter(content).count() as u32;
                if count > 0 {
                    operators.insert(pattern.to_string(), count);
                }
            }
        }
        
        // Простое извлечение операндов (идентификаторы и литералы)
        let operand_pattern = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b|\b\d+\b").unwrap();
        
        for mat in operand_pattern.find_iter(content) {
            let operand = mat.as_str().to_string();
            *operands.entry(operand).or_insert(0) += 1;
        }
        
        Ok((operators, operands))
    }
}

impl Default for AdvancedMetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SOLIDAnalyzer {
    fn default() -> Self {
        Self {
            single_responsibility: SRPAnalyzer {
                responsibility_indicators: vec![],
                max_responsibilities: 3,
            },
            open_closed: OCPAnalyzer {
                modification_indicators: vec![],
                extension_indicators: vec![],
            },
            liskov_substitution: LSPAnalyzer {
                substitution_violations: vec![],
                behavioral_change_indicators: vec![],
            },
            interface_segregation: ISPAnalyzer {
                interface_size_indicators: vec![],
                max_interface_methods: 5,
            },
            dependency_inversion: DIPAnalyzer {
                concrete_dependency_indicators: vec![],
                abstraction_indicators: vec![],
            },
        }
    }
} 