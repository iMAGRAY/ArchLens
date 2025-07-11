// Новый модульный обогатитель капсул

use crate::types::*;
use crate::enrichment::*;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Результат обогащения капсулы
#[derive(Debug, Clone)]
pub struct EnrichmentResult {
    pub semantic_links: Vec<SemanticLink>,
    pub quality_metrics: QualityMetrics,
    pub architectural_patterns: Vec<ArchitecturalPattern>,
    pub code_smells: Vec<CodeSmell>,
    pub dependency_analysis: DependencyAnalysis,
    pub content_analysis: ContentAnalysis,
}

/// Модульный обогатитель капсул
pub struct ModularCapsuleEnricher {
    semantic_engine: SemanticAnalysisEngine,
    pattern_detector: PatternDetector,
    quality_calculator: QualityMetricsCalculator,
    smell_detector: CodeSmellDetector,
    dependency_analyzer: DependencyAnalyzer,
    content_analyzer: ContentAnalyzer,
    analysis_cache: HashMap<String, EnrichmentResult>,
}

impl ModularCapsuleEnricher {
    pub fn new() -> Self {
        Self {
            semantic_engine: SemanticAnalysisEngine::new(),
            pattern_detector: PatternDetector::new(),
            quality_calculator: QualityMetricsCalculator::new(),
            smell_detector: CodeSmellDetector::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            content_analyzer: ContentAnalyzer::new(),
            analysis_cache: HashMap::new(),
        }
    }
    
    /// Обогащение всего графа капсул
    pub fn enrich_graph(&mut self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        let mut enriched_capsules = HashMap::new();
        let mut enriched_relations = graph.relations.clone();
        
        for (id, capsule) in &graph.capsules {
            let mut enriched_capsule = capsule.clone();
            
            // Обогащаем капсулу содержимым файла
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                let enrichment_result = self.enrich_capsule_content(&content, &capsule.file_path)?;
                
                // Обновляем капсулу данными из анализа
                self.update_capsule_with_enrichment(&mut enriched_capsule, &enrichment_result)?;
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
    
    /// Обогащение содержимого одной капсулы
    pub fn enrich_capsule_content(&mut self, content: &str, file_path: &Path) -> Result<EnrichmentResult> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Проверяем кеш
        if let Some(cached) = self.analysis_cache.get(&file_path_str) {
            return Ok(cached.clone());
        }
        
        let file_type = self.determine_file_type(file_path);
        
        // Семантический анализ
        let semantic_result = self.semantic_engine.analyze(content, file_type)?;
        
        // Обнаружение архитектурных паттернов
        let architectural_patterns = self.pattern_detector
            .detect_patterns(content, &semantic_result.semantic_links)?;
        
        // Вычисление метрик качества
        let quality_metrics = self.quality_calculator
            .calculate_metrics(content, file_type, &semantic_result.semantic_links)?;
        
        // Обнаружение запахов кода
        let code_smells = self.smell_detector
            .detect_code_smells(content, file_type)?;
        
        // Анализ зависимостей
        let dependency_analysis = self.dependency_analyzer
            .analyze_dependencies(content, &file_type, file_path)?;
        
        // Анализ содержимого
        let content_analysis = self.content_analyzer
            .analyze_content(content, &file_type, file_path)?;
        
        let result = EnrichmentResult {
            semantic_links: semantic_result.semantic_links,
            quality_metrics,
            architectural_patterns,
            code_smells,
            dependency_analysis,
            content_analysis,
        };
        
        // Кешируем результат
        self.analysis_cache.insert(file_path_str, result.clone());
        
        Ok(result)
    }
    
    /// Обновление капсулы данными из анализа
    fn update_capsule_with_enrichment(&self, capsule: &mut Capsule, enrichment: &EnrichmentResult) -> Result<()> {
        // Обновляем размер из метрик файла
        capsule.size = enrichment.content_analysis.file_metrics.total_lines;
        
        // Обновляем описание из документации
        if !enrichment.content_analysis.documentation.is_empty() {
            capsule.description = enrichment.content_analysis.documentation.join(" ");
        }
        
        // Обновляем теги из архитектурных паттернов
        capsule.tags = enrichment.architectural_patterns.iter()
            .map(|p| format!("{:?}", p.pattern_type))
            .collect();
        
        // Генерируем предупреждения из запахов кода
        let mut warnings = Vec::new();
        for smell in &enrichment.code_smells {
            warnings.push(AnalysisWarning {
                warning_type: WarningType::CodeSmell,
                message: smell.description.clone(),
                severity: smell.severity.clone(),
                suggestion: Some(smell.suggestion.clone()),
                location: smell.location.clone(),
                category: "code_quality".to_string(),
            });
        }
        capsule.warnings = warnings;
        
        // Обновляем качество
        capsule.quality_index = enrichment.quality_metrics.maintainability_index as f64;
        
        Ok(())
    }
    
    /// Обогащение связей на основе найденных зависимостей
    fn enrich_relations(&self, capsules: &HashMap<Uuid, Capsule>, relations: &mut Vec<CapsuleRelation>) -> Result<()> {
        // Добавляем новые связи на основе семантического анализа
        for (id, capsule) in capsules {
            // Здесь можно добавить логику для создания новых связей
            // на основе найденных зависимостей и семантических связей
        }
        
        Ok(())
    }
    
    /// Определение типа файла
    fn determine_file_type(&self, path: &Path) -> FileType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => FileType::Rust,
            Some("js") => FileType::JavaScript,
            Some("ts") => FileType::TypeScript,
            Some("tsx") => FileType::TypeScript,
            Some("py") => FileType::Python,
            Some("cpp") | Some("cc") | Some("cxx") => FileType::Cpp,
            Some("c") => FileType::C,
            Some("h") | Some("hpp") => FileType::C,
            Some("java") => FileType::Java,
            Some("go") => FileType::Go,
            Some("rb") => FileType::Ruby,
            Some("php") => FileType::Php,
            Some("swift") => FileType::Swift,
            Some("kt") => FileType::Kotlin,
            Some("cs") => FileType::CSharp,
            Some("json") => FileType::Json,
            Some("xml") => FileType::Xml,
            Some("html") => FileType::Html,
            Some("css") => FileType::Css,
            Some("sql") => FileType::Sql,
            Some("sh") => FileType::Shell,
            Some("yaml") | Some("yml") => FileType::Yaml,
            Some("toml") => FileType::Toml,
            Some("md") => FileType::Markdown,
            Some("txt") => FileType::Text,
            _ => FileType::Other,
        }
    }
    
    /// Очистка кеша
    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
    }
    
    /// Получение статистики кеша
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.analysis_cache.len(), self.analysis_cache.capacity())
    }
}

impl Default for ModularCapsuleEnricher {
    fn default() -> Self {
        Self::new()
    }
} 