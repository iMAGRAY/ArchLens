// Семантический обогатитель капсул с анализом связей и метаданных

use crate::types::*;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::path::Path;
use uuid::Uuid;
use crate::types::Result;

pub struct CapsuleEnricher {
    import_patterns: HashMap<FileType, Regex>,
    export_patterns: HashMap<FileType, Regex>,
    // Продвинутые анализаторы
    semantic_analyzers: HashMap<FileType, SemanticAnalyzer>,
    // Кеш для оптимизации
    analysis_cache: HashMap<String, EnrichmentResult>,
    // Паттерны для обнаружения архитектурных проблем
    antipattern_detectors: Vec<AntipatternDetector>,
}

/// Результат обогащения капсулы
#[derive(Debug, Clone)]
pub struct EnrichmentResult {
    pub semantic_links: Vec<SemanticLink>,
    pub quality_metrics: QualityMetrics,
    pub architectural_patterns: Vec<ArchitecturalPattern>,
    pub code_smells: Vec<CodeSmell>,
}

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

/// Метрики качества кода
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub maintainability_index: f32,
    pub cognitive_complexity: u32,
    pub technical_debt_ratio: f32,
    pub test_coverage_estimate: f32,
    pub documentation_completeness: f32,
}

/// Архитектурные паттерны
#[derive(Debug, Clone)]
pub struct ArchitecturalPattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub description: String,
}

/// Типы архитектурных паттернов
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

/// Обнаружитель запахов кода
#[derive(Debug, Clone)]
pub struct CodeSmell {
    pub smell_type: CodeSmellType,
    pub severity: Priority,
    pub description: String,
    pub suggestion: String,
}

/// Типы запахов кода
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

/// Детектор антипаттернов
#[derive(Debug)]
pub struct AntipatternDetector {
    pub pattern_name: String,
    pub detection_regex: Regex,
    pub severity: Priority,
    pub description: String,
}

impl CapsuleEnricher {
    pub fn new() -> Self {
        let mut import_patterns = HashMap::new();
        let mut export_patterns = HashMap::new();
        
        // Rust паттерны
        import_patterns.insert(
            FileType::Rust,
            Regex::new(r"use\s+([^;]+);").unwrap()
        );
        export_patterns.insert(
            FileType::Rust,
            Regex::new(r"pub\s+(fn|struct|enum|mod|trait|const|static)\s+(\w+)").unwrap()
        );
        
        // JavaScript/TypeScript паттерны
        let js_import = Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#).unwrap();
        import_patterns.insert(FileType::JavaScript, js_import.clone());
        import_patterns.insert(FileType::TypeScript, js_import);
        
        let js_export = Regex::new(r"export\s+(function|class|const|let|var|default)\s+(\w+)").unwrap();
        export_patterns.insert(FileType::JavaScript, js_export.clone());
        export_patterns.insert(FileType::TypeScript, js_export);
        
        // Python паттерны
        import_patterns.insert(
            FileType::Python,
            Regex::new(r"(?:from\s+(\S+)\s+)?import\s+([^#\n]+)").unwrap()
        );
        export_patterns.insert(
            FileType::Python,
            Regex::new(r"^(def|class)\s+(\w+)").unwrap()
        );
        
        Self {
            import_patterns,
            export_patterns,
            semantic_analyzers: Self::create_semantic_analyzers(),
            analysis_cache: HashMap::new(),
            antipattern_detectors: Self::create_antipattern_detectors(),
        }
    }
    
    pub fn enrich_graph(&self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        let mut enriched_capsules = HashMap::new();
        let mut enriched_relations = graph.relations.clone();
        
        for (id, capsule) in &graph.capsules {
            let mut enriched_capsule = capsule.clone();
            
            // Обогащение метаданными из содержимого файла
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                self.enrich_capsule_metadata(&mut enriched_capsule, &content)?;
                self.analyze_dependencies(&mut enriched_capsule, &content)?;
                self.extract_exports(&mut enriched_capsule, &content)?;
                self.generate_warnings(&mut enriched_capsule, &content)?;
            }
            
            enriched_capsules.insert(*id, enriched_capsule);
        }
        
        // Обогащаем связи на основе найденных зависимостей
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
    
    fn enrich_capsule_metadata(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        // Извлечение комментариев и документации
        let doc_comments = self.extract_documentation(&content, &capsule.file_path);
        if let Some(doc) = doc_comments.first() {
            capsule.summary = Some(doc.clone());
        }
        
        // Обновление счетчика строк
        let actual_lines = content.lines().count();
        if actual_lines != capsule.line_end {
            capsule.line_end = actual_lines;
        }
        
        // Метаданные по содержимому
        capsule.metadata.insert("actual_lines".to_string(), actual_lines.to_string());
        capsule.metadata.insert("has_tests".to_string(), self.has_tests(content).to_string());
        capsule.metadata.insert("has_documentation".to_string(), (!doc_comments.is_empty()).to_string());
        
        // Анализ качества кода
        let code_quality_score = self.calculate_code_quality(content);
        capsule.metadata.insert("quality_score".to_string(), code_quality_score.to_string());
        
        Ok(())
    }
    
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
                dependencies.into_iter().collect::<Vec<_>>().join(", ")
            );
        }
        
        Ok(())
    }
    
    fn extract_exports(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        let file_type = self.determine_file_type(&capsule.file_path);
        
        if let Some(pattern) = self.export_patterns.get(&file_type) {
            let mut exports = Vec::new();
            
            for capture in pattern.captures_iter(content) {
                if let Some(export_name) = capture.get(2) {
                    exports.push(export_name.as_str().to_string());
                }
            }
            
            capsule.metadata.insert(
                "public_exports".to_string(),
                exports.join(", ")
            );
        }
        
        Ok(())
    }
    
    fn generate_warnings(&self, capsule: &mut Capsule, content: &str) -> Result<()> {
        capsule.warnings.clear();
        
        // Проверка размера файла
        if content.lines().count() > 500 {
            capsule.warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!(
                    "Файл слишком большой ({} строк). Рассмотрите разбиение на модули.",
                    content.lines().count()
                ),
                category: "size".to_string(),
                capsule_id: Some(capsule.id),
                suggestion: Some("Разделите большой файл на несколько модулей".to_string()),
            });
        }
        
        // Проверка сложности
        if capsule.complexity > 20 {
            capsule.warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!(
                    "Высокая сложность ({}). Рассмотрите рефакторинг.",
                    capsule.complexity
                ),
                category: "complexity".to_string(),
                capsule_id: Some(capsule.id),
                suggestion: Some("Упростите логику или разбейте на более мелкие функции".to_string()),
            });
        }
        
        // Проверка на TODO/FIXME
        let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
        if todo_count > 3 {
            capsule.warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: format!(
                    "Много незавершенных задач ({todo_count} TODO/FIXME)."
                ),
                category: "maintenance".to_string(),
                capsule_id: Some(capsule.id),
                suggestion: Some("Завершите или документируйте незавершенные задачи".to_string()),
            });
        }
        
        // Проверка на отсутствие документации
        if capsule.summary.is_none() && capsule.capsule_type != CapsuleType::Variable {
            capsule.warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: "Отсутствует документация".to_string(),
                category: "documentation".to_string(),
                capsule_id: Some(capsule.id),
                suggestion: Some("Добавьте документацию к публичным интерфейсам".to_string()),
            });
        }
        
        // Проверка на дублирование кода
        if self.has_code_duplication(content) {
            capsule.warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: "Обнаружено возможное дублирование кода".to_string(),
                category: "duplication".to_string(),
                capsule_id: Some(capsule.id),
                suggestion: Some("Выделите общую функциональность в отдельные методы".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn enrich_relations(&self, capsules: &HashMap<Uuid, Capsule>, relations: &mut Vec<CapsuleRelation>) -> Result<()> {
        // Добавляем новые связи на основе найденных зависимостей
        for capsule in capsules.values() {
            if let Some(deps) = capsule.metadata.get("external_dependencies") {
                for dep_name in deps.split(", ") {
                    if dep_name.is_empty() { continue; }
                    
                    // Ищем капсулу с таким именем
                    for other_capsule in capsules.values() {
                        if other_capsule.name.contains(dep_name) || 
                           other_capsule.file_path.to_string_lossy().contains(dep_name) {
                            
                            // Проверяем, нет ли уже такой связи
                            let relation_exists = relations.iter().any(|r| 
                                r.from_id == capsule.id && r.to_id == other_capsule.id
                            );
                            
                            if !relation_exists {
                                relations.push(CapsuleRelation {
                                    from_id: capsule.id,
                                    to_id: other_capsule.id,
                                    relation_type: RelationType::Uses,
                                    strength: 0.6,
                                    description: Some(format!("Использует {dep_name}")),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Извлекает документацию из содержимого файла
    fn extract_documentation(&self, content: &str, file_path: &Path) -> Vec<String> {
        let mut docs = Vec::new();
        
        // Rust-style документация
        if file_path.extension().is_some_and(|e| e == "rs") {
            // Ищем /// комментарии
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("///") {
                    docs.push(trimmed.trim_start_matches("///").trim().to_string());
                }
            }
        }
        
        // JS/TS-style документация
        if file_path.extension().is_some_and(|e| e == "js" || e == "ts" || e == "tsx") {
            // Ищем JSDoc комментарии
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("/**") || trimmed.starts_with("*") {
                    docs.push(trimmed.trim_start_matches("/**").trim_start_matches("*").trim().to_string());
                }
            }
        }
        
        // Python-style документация
        if file_path.extension().is_some_and(|e| e == "py") {
            // Ищем docstrings
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                    docs.push(trimmed.trim_start_matches("\"\"\"").trim_start_matches("'''").trim().to_string());
                }
            }
        }
        
        docs
    }
    
    fn has_tests(&self, content: &str) -> bool {
        content.contains("#[test]") ||
        content.contains("test(") ||
        content.contains("describe(") ||
        content.contains("it(") ||
        content.contains("def test_")
    }
    
    fn calculate_code_quality(&self, content: &str) -> f32 {
        let mut score: f32 = 50.0; // Базовый балл
        let lines = content.lines().collect::<Vec<_>>();
        
        // Штраф за длинные строки
        let long_lines = lines.iter().filter(|line| line.len() > 120).count();
        score -= long_lines as f32 * 2.0;
        
        // Штраф за большие функции
        let function_starts = content.matches("fn ").count() + content.matches("function ").count() + content.matches("def ").count();
        if function_starts > 0 {
            let avg_lines_per_function = lines.len() as f32 / function_starts as f32;
            if avg_lines_per_function > 50.0 {
                score -= (avg_lines_per_function - 50.0) * 0.5;
            }
        }
        
        // Бонус за комментарии
        let comment_lines = lines.iter().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*")
        }).count();
        let comment_ratio = comment_lines as f32 / lines.len() as f32;
        score += comment_ratio * 20.0;
        
        score.max(0.0).min(100.0)
    }
    
    fn has_code_duplication(&self, content: &str) -> bool {
        let lines: Vec<&str> = content.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("#"))
            .collect();
        
        // Простая проверка на повторяющиеся блоки из 3+ строк
        for i in 0..lines.len().saturating_sub(3) {
            let block = &lines[i..i+3];
            for j in (i+3)..lines.len().saturating_sub(3) {
                let other_block = &lines[j..j+3];
                if block == other_block {
                    return true;
                }
            }
        }
        
        false
    }

    /// Вычисляет индекс качества капсулы
    #[allow(dead_code)]
    fn calculate_quality_index(&self, capsule: &Capsule) -> f64 {
        let mut score: f32 = 50.0; // Базовый балл
        
        // Факторы качества
        if capsule.complexity <= 10 {
            score += 20.0;
        } else if capsule.complexity <= 20 {
            score += 10.0;
        } else {
            score -= 10.0;
        }
        
        // Размер файла
        if capsule.size <= 100 {
            score += 10.0;
        } else if capsule.size > 1000 {
            score -= 15.0;
        }
        
        score.clamp(0.0, 100.0) as f64
    }

    /// Определяет тип файла по расширению
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

    /// Создает семантические анализаторы для всех поддерживаемых языков
    fn create_semantic_analyzers() -> HashMap<FileType, SemanticAnalyzer> {
        let mut analyzers = HashMap::new();
        
        // Rust анализатор
        analyzers.insert(FileType::Rust, SemanticAnalyzer {
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
        });
        
        // TypeScript/JavaScript анализатор
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
        
        // Python анализатор
        analyzers.insert(FileType::Python, SemanticAnalyzer {
            language: FileType::Python,
            method_call_patterns: vec![
                Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
                Regex::new(r"self\.(\w+)\s*\(").unwrap(),
            ],
            field_access_patterns: vec![
                Regex::new(r"(\w+)\.(\w+)").unwrap(),
                Regex::new(r"self\.(\w+)").unwrap(),
            ],
            inheritance_patterns: vec![
                Regex::new(r"class\s+(\w+)\s*\(\s*(\w+)\s*\)").unwrap(),
            ],
            composition_patterns: vec![
                Regex::new(r"(\w+)\s*=\s*(\w+)\s*\(").unwrap(),
                Regex::new(r"self\.(\w+)\s*=\s*(\w+)\s*\(").unwrap(),
            ],
            complexity_patterns: vec![
                Regex::new(r"\bif\b").unwrap(),
                Regex::new(r"\belif\b").unwrap(),
                Regex::new(r"\belse\b").unwrap(),
                Regex::new(r"\bfor\b").unwrap(),
                Regex::new(r"\bwhile\b").unwrap(),
                Regex::new(r"\btry\b").unwrap(),
                Regex::new(r"\bexcept\b").unwrap(),
            ],
        });
        
        // Java анализатор
        analyzers.insert(FileType::Java, SemanticAnalyzer {
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
        });
        
        analyzers
    }
    
    /// Создает детекторы антипаттернов и запахов кода
    fn create_antipattern_detectors() -> Vec<AntipatternDetector> {
        vec![
            AntipatternDetector {
                pattern_name: "God Object".to_string(),
                detection_regex: Regex::new(r"class\s+\w+\s*\{[\s\S]{1000,}").unwrap(),
                severity: Priority::High,
                description: "Класс слишком большой и выполняет слишком много функций".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Long Method".to_string(),
                detection_regex: Regex::new(r"(?:fn|function|def)\s+\w+[^}]{500,}").unwrap(),
                severity: Priority::Medium,
                description: "Метод слишком длинный и сложный для понимания".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Long Parameter List".to_string(),
                detection_regex: Regex::new(r"(?:fn|function|def)\s+\w+\s*\([^)]{100,}\)").unwrap(),
                severity: Priority::Medium,
                description: "Слишком много параметров в методе".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Duplicated Code".to_string(),
                detection_regex: Regex::new(r"(\b\w+\s+\w+\s+\w+\s+\w+\b).{20,}(\b\w+\s+\w+\s+\w+\s+\w+\b)").unwrap(),
                severity: Priority::Medium,
                description: "Обнаружено дублирование кода".to_string(),
            },
            AntipatternDetector {
                pattern_name: "Dead Code".to_string(),
                detection_regex: Regex::new(r"(?://|#|/\*)\s*TODO|FIXME|XXX|HACK").unwrap(),
                severity: Priority::Low,
                description: "Обнаружен мертвый код или TODO комментарии".to_string(),
            },
        ]
    }
    
    /// Выполняет полный семантический анализ капсулы
    pub fn perform_semantic_analysis(&mut self, capsule: &Capsule, content: &str) -> Result<EnrichmentResult> {
        let cache_key = format!("{}:{}", capsule.file_path.display(), content.len());
        
        if let Some(cached) = self.analysis_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let file_type = self.determine_file_type(&capsule.file_path);
        let analyzer = self.semantic_analyzers.get(&file_type);
        
        let semantic_links = if let Some(analyzer) = analyzer {
            self.extract_semantic_links(content, analyzer)?
        } else {
            Vec::new()
        };
        
        let quality_metrics = self.calculate_quality_metrics(content, &semantic_links)?;
        let architectural_patterns = self.detect_architectural_patterns(content, &semantic_links)?;
        let code_smells = self.detect_code_smells(content)?;
        
        let result = EnrichmentResult {
            semantic_links,
            quality_metrics,
            architectural_patterns,
            code_smells,
        };
        
        self.analysis_cache.insert(cache_key, result.clone());
        Ok(result)
    }
    
    /// Извлекает семантические связи из кода
    fn extract_semantic_links(&self, content: &str, analyzer: &SemanticAnalyzer) -> Result<Vec<SemanticLink>> {
        let mut links = Vec::new();
        
        // Анализ вызовов методов
        for pattern in &analyzer.method_call_patterns {
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
        
        // Анализ доступа к полям
        for pattern in &analyzer.field_access_patterns {
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
        
        // Анализ наследования
        for pattern in &analyzer.inheritance_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(child), Some(parent)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Inheritance,
                        target_name: format!("{} -> {}", child.as_str(), parent.as_str()),
                        strength: 0.9,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }
        
        // Анализ композиции
        for pattern in &analyzer.composition_patterns {
            for captures in pattern.captures_iter(content) {
                if let (Some(container), Some(component)) = (captures.get(1), captures.get(2)) {
                    links.push(SemanticLink {
                        link_type: SemanticLinkType::Composition,
                        target_name: format!("{} contains {}", container.as_str(), component.as_str()),
                        strength: 0.7,
                        context: captures.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }
        
        Ok(links)
    }
    
    /// Вычисляет метрики качества кода
    fn calculate_quality_metrics(&self, content: &str, semantic_links: &[SemanticLink]) -> Result<QualityMetrics> {
        let lines = content.lines().count();
        let complexity = self.calculate_cyclomatic_complexity(content);
        let documentation_ratio = self.calculate_documentation_ratio(content);
        let test_coverage = self.estimate_test_coverage(content);
        
        // Индекс сопровождаемости (упрощенная формула)
        let maintainability_index = (171.0 
            - 5.2 * (lines as f32).ln() 
            - 0.23 * complexity as f32 
            - 16.2 * (lines as f32).ln() 
            + 50.0 * documentation_ratio).max(0.0);
        
        // Когнитивная сложность
        let cognitive_complexity = self.calculate_cognitive_complexity(content);
        
        // Коэффициент технического долга
        let technical_debt_ratio = self.calculate_technical_debt_ratio(content, semantic_links);
        
        Ok(QualityMetrics {
            maintainability_index,
            cognitive_complexity,
            technical_debt_ratio,
            test_coverage_estimate: test_coverage,
            documentation_completeness: documentation_ratio,
        })
    }
    
    /// Обнаруживает архитектурные паттерны
    fn detect_architectural_patterns(&self, content: &str, semantic_links: &[SemanticLink]) -> Result<Vec<ArchitecturalPattern>> {
        let mut patterns = Vec::new();
        
        // Паттерн Singleton
        if content.contains("private static") && content.contains("getInstance") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Singleton,
                confidence: 0.8,
                description: "Обнаружен паттерн Singleton".to_string(),
            });
        }
        
        // Паттерн Factory
        if content.contains("create") && content.contains("new ") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Factory,
                confidence: 0.6,
                description: "Возможно использование паттерна Factory".to_string(),
            });
        }
        
        // Паттерн Repository
        if content.contains("Repository") || content.contains("findBy") || content.contains("save") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Repository,
                confidence: 0.7,
                description: "Обнаружен паттерн Repository".to_string(),
            });
        }
        
        // Паттерн Service
        if content.contains("Service") || content.contains("@Service") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Service,
                confidence: 0.8,
                description: "Обнаружен паттерн Service".to_string(),
            });
        }
        
        // Паттерн Controller
        if content.contains("Controller") || content.contains("@Controller") || content.contains("@RestController") {
            patterns.push(ArchitecturalPattern {
                pattern_type: PatternType::Controller,
                confidence: 0.9,
                description: "Обнаружен паттерн Controller".to_string(),
            });
        }
        
        Ok(patterns)
    }
    
    /// Обнаруживает запахи кода
    fn detect_code_smells(&self, content: &str) -> Result<Vec<CodeSmell>> {
        let mut smells = Vec::new();
        
        for detector in &self.antipattern_detectors {
            if detector.detection_regex.is_match(content) {
                let smell_type = match detector.pattern_name.as_str() {
                    "God Object" => CodeSmellType::GodObject,
                    "Long Method" => CodeSmellType::LongMethod,
                    "Long Parameter List" => CodeSmellType::LongParameterList,
                    "Duplicated Code" => CodeSmellType::DuplicatedCode,
                    "Dead Code" => CodeSmellType::DeadCode,
                    _ => CodeSmellType::DeadCode,
                };
                
                smells.push(CodeSmell {
                    smell_type,
                    severity: detector.severity,
                    description: detector.description.clone(),
                    suggestion: self.generate_suggestion(&detector.pattern_name),
                });
            }
        }
        
        Ok(smells)
    }
    
    /// Генерирует предложение по исправлению
    fn generate_suggestion(&self, pattern_name: &str) -> String {
        match pattern_name {
            "God Object" => "Разбейте класс на меньшие, более специализированные классы".to_string(),
            "Long Method" => "Разбейте метод на более мелкие методы".to_string(),
            "Long Parameter List" => "Используйте объект параметров или паттерн Builder".to_string(),
            "Duplicated Code" => "Выделите общую логику в отдельный метод".to_string(),
            "Dead Code" => "Удалите неиспользуемый код или завершите TODO".to_string(),
            _ => "Рассмотрите рефакторинг этого кода".to_string(),
        }
    }
    
    /// Вычисляет цикломатическую сложность
    fn calculate_cyclomatic_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Базовая сложность
        
        // Подсчет условных конструкций
        complexity += content.matches("if ").count() as u32;
        complexity += content.matches("else if ").count() as u32;
        complexity += content.matches("while ").count() as u32;
        complexity += content.matches("for ").count() as u32;
        complexity += content.matches("match ").count() as u32;
        complexity += content.matches("case ").count() as u32;
        complexity += content.matches("catch ").count() as u32;
        complexity += content.matches("&&").count() as u32;
        complexity += content.matches("||").count() as u32;
        
        complexity
    }
    
    /// Вычисляет когнитивную сложность
    fn calculate_cognitive_complexity(&self, content: &str) -> u32 {
        let mut complexity = 0;
        let mut nesting_level: u32 = 0;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Увеличиваем уровень вложенности
            if trimmed.contains("{") {
                nesting_level += 1;
            }
            
            // Уменьшаем уровень вложенности
            if trimmed.contains("}") {
                nesting_level = nesting_level.saturating_sub(1);
            }
            
            // Добавляем сложность с учетом вложенности
            if trimmed.contains("if ") || trimmed.contains("while ") || trimmed.contains("for ") {
                complexity += 1 + nesting_level;
            }
        }
        
        complexity
    }
    
    /// Вычисляет коэффициент документированности
    fn calculate_documentation_ratio(&self, content: &str) -> f32 {
        let total_lines = content.lines().count();
        if total_lines == 0 {
            return 0.0;
        }
        
        let comment_lines = content.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("//") || trimmed.starts_with("/*") || 
                trimmed.starts_with("*") || trimmed.starts_with("#") ||
                trimmed.starts_with("///") || trimmed.starts_with("/**")
            })
            .count();
        
        comment_lines as f32 / total_lines as f32
    }
    
    /// Оценивает покрытие тестами
    fn estimate_test_coverage(&self, content: &str) -> f32 {
        let has_tests = content.contains("test") || content.contains("Test") || 
                       content.contains("spec") || content.contains("Spec") ||
                       content.contains("#[test]") || content.contains("@Test");
        
        if has_tests {
            0.8 // Приблизительная оценка
        } else {
            0.0
        }
    }
    
    /// Вычисляет коэффициент технического долга
    fn calculate_technical_debt_ratio(&self, content: &str, semantic_links: &[SemanticLink]) -> f32 {
        let mut debt_score = 0.0;
        
        // TODO/FIXME комментарии
        debt_score += (content.matches("TODO").count() * 2) as f32;
        debt_score += (content.matches("FIXME").count() * 3) as f32;
        debt_score += (content.matches("HACK").count() * 4) as f32;
        
        // Дублирование кода
        if self.has_code_duplication(content) {
            debt_score += 5.0;
        }
        
        // Высокая сложность
        if self.calculate_cyclomatic_complexity(content) > 10 {
            debt_score += 3.0;
        }
        
        // Длинные методы
        for line in content.lines() {
            if line.len() > 120 {
                debt_score += 0.1;
            }
        }
        
        // Нормализация (0.0 - 1.0)
        (debt_score / 100.0).min(1.0)
    }
}

impl Default for CapsuleEnricher {
    fn default() -> Self {
        Self::new()
    }
} 