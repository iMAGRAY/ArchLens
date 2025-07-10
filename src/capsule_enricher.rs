// #ДОДЕЛАТЬ: Реализация CapsuleEnricher для обогащения капсул связями

use crate::core::*;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::path::Path;

pub struct CapsuleEnricher {
    import_patterns: HashMap<FileType, Regex>,
    export_patterns: HashMap<FileType, Regex>,
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
            capsule.warnings.push(format!(
                "Файл слишком большой ({} строк). Рассмотрите разбиение на модули.",
                content.lines().count()
            ));
        }
        
        // Проверка сложности
        if capsule.complexity > 20 {
            capsule.warnings.push(format!(
                "Высокая сложность ({}). Рассмотрите рефакторинг.",
                capsule.complexity
            ));
        }
        
        // Проверка на TODO/FIXME
        let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
        if todo_count > 3 {
            capsule.warnings.push(format!(
                "Много незавершенных задач ({todo_count} TODO/FIXME)."
            ));
        }
        
        // Проверка на отсутствие документации
        if capsule.summary.is_none() && capsule.capsule_type != CapsuleType::Variable {
            capsule.warnings.push("Отсутствует документация".to_string());
        }
        
        // Проверка на дублирование кода
        if self.has_code_duplication(content) {
            capsule.warnings.push("Обнаружено возможное дублирование кода".to_string());
        }
        
        Ok(())
    }
    
    fn enrich_relations(&self, capsules: &HashMap<uuid::Uuid, Capsule>, relations: &mut Vec<CapsuleRelation>) -> Result<()> {
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
        let mut score = 100.0;
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
    fn calculate_quality_index(&self, capsule: &Capsule) -> f64 {
        let mut score = 50.0; // Базовый балл
        
        // Штраф за сложность
        if capsule.complexity > 10 {
            score -= 10.0;
        }
        
        // Бонус за документацию
        if capsule.summary.is_some() {
            score += 10.0;
        }
        
        // Штраф за предупреждения
        score -= capsule.warnings.len() as f64 * 5.0;
        
        // Бонус за связи
        score += capsule.dependencies.len() as f64 * 2.0;
        
        score.clamp(0.0, 100.0)
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
}

impl Default for CapsuleEnricher {
    fn default() -> Self {
        Self::new()
    }
} 