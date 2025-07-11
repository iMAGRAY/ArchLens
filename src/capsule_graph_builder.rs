// Продвинутый построитель графа капсул с реальным анализом зависимостей

use crate::types::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use regex::Regex;
use std::path::Path;
use crate::types::Result;

pub struct CapsuleGraphBuilder {
    relation_strength_threshold: f32,
    // Кеш для анализа зависимостей
    dependency_cache: HashMap<String, Vec<String>>,
    // Паттерны для анализа импортов/экспортов
    import_patterns: HashMap<FileType, Vec<Regex>>,
    export_patterns: HashMap<FileType, Vec<Regex>>,
    // Анализатор циклических зависимостей
    cycle_detector: CycleDetector,
}

/// Детектор циклических зависимостей
#[derive(Debug)]
pub struct CycleDetector {
    visited: HashSet<Uuid>,
    recursion_stack: HashSet<Uuid>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            recursion_stack: HashSet::new(),
        }
    }
    
    /// Обнаруживает циклы в графе зависимостей
    pub fn find_cycles(&mut self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        self.visited.clear();
        self.recursion_stack.clear();
        
        for capsule_id in graph.capsules.keys() {
            if !self.visited.contains(capsule_id) {
                if let Some(cycle) = self.dfs_cycle_detection(*capsule_id, graph, &mut Vec::new()) {
                    cycles.push(cycle);
                }
            }
        }
        
        cycles
    }
    
    fn dfs_cycle_detection(&mut self, capsule_id: Uuid, graph: &CapsuleGraph, path: &mut Vec<Uuid>) -> Option<Vec<Uuid>> {
        self.visited.insert(capsule_id);
        self.recursion_stack.insert(capsule_id);
        path.push(capsule_id);
        
        // Проверяем все зависимости текущей капсулы
        if let Some(capsule) = graph.capsules.get(&capsule_id) {
            for &dependency_id in &capsule.dependencies {
                if !self.visited.contains(&dependency_id) {
                    if let Some(cycle) = self.dfs_cycle_detection(dependency_id, graph, path) {
                        return Some(cycle);
                    }
                } else if self.recursion_stack.contains(&dependency_id) {
                    // Найден цикл
                    if let Some(cycle_start_pos) = path.iter().position(|&id| id == dependency_id) {
                        return Some(path[cycle_start_pos..].to_vec());
                    } else {
                        // Если не можем найти начало цикла, возвращаем весь путь
                        return Some(path.clone());
                    }
                }
            }
        }
        
        path.pop();
        self.recursion_stack.remove(&capsule_id);
        None
    }
}

impl CapsuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            relation_strength_threshold: 0.1,
            dependency_cache: HashMap::new(),
            import_patterns: Self::create_import_patterns(),
            export_patterns: Self::create_export_patterns(),
            cycle_detector: CycleDetector::new(),
        }
    }
    
    /// Создает паттерны для анализа импортов
    fn create_import_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust импорты
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"use\s+([^;]+);").unwrap(),
            Regex::new(r"extern\s+crate\s+(\w+)").unwrap(),
            Regex::new(r"mod\s+(\w+)").unwrap(),
        ]);
        
        // JavaScript/TypeScript импорты
        let js_patterns = vec![
            Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap(),
            Regex::new(r#"import\s+['"]([^'"]+)['"]"#).unwrap(),
            Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]"#).unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python импорты
        patterns.insert(FileType::Python, vec![
            Regex::new(r"import\s+([^\s#]+)").unwrap(),
            Regex::new(r"from\s+([^\s]+)\s+import").unwrap(),
        ]);
        
        // Java импорты
        patterns.insert(FileType::Java, vec![
            Regex::new(r"import\s+([^;]+);").unwrap(),
            Regex::new(r"package\s+([^;]+);").unwrap(),
        ]);
        
        // C++ импорты
        patterns.insert(FileType::Cpp, vec![
            Regex::new(r#"#include\s+[<"]([^>"]+)[>"]"#).unwrap(),
        ]);
        
        // Go импорты
        patterns.insert(FileType::Go, vec![
            Regex::new(r#"import\s+"([^"]+)""#).unwrap(),
            Regex::new(r"import\s+\(\s*([^)]+)\s*\)").unwrap(),
        ]);
        
        patterns
    }
    
    /// Создает паттерны для анализа экспортов
    fn create_export_patterns() -> HashMap<FileType, Vec<Regex>> {
        let mut patterns = HashMap::new();
        
        // Rust экспорты
        patterns.insert(FileType::Rust, vec![
            Regex::new(r"pub\s+(?:fn|struct|enum|trait|mod|const|static)\s+(\w+)").unwrap(),
            Regex::new(r"pub\s+use\s+([^;]+);").unwrap(),
        ]);
        
        // JavaScript/TypeScript экспорты
        let js_patterns = vec![
            Regex::new(r"export\s+(?:default\s+)?(?:function|class|const|let|var|interface|type|enum)\s+(\w+)").unwrap(),
            Regex::new(r"export\s+\{\s*([^}]+)\s*\}").unwrap(),
            Regex::new(r"module\.exports\s*=\s*(\w+)").unwrap(),
        ];
        patterns.insert(FileType::JavaScript, js_patterns.clone());
        patterns.insert(FileType::TypeScript, js_patterns);
        
        // Python экспорты
        patterns.insert(FileType::Python, vec![
            Regex::new(r"__all__\s*=\s*\[([^\]]+)\]").unwrap(),
            Regex::new(r"^(?:def|class)\s+(\w+)").unwrap(),
        ]);
        
        // Java экспорты
        patterns.insert(FileType::Java, vec![
            Regex::new(r"public\s+(?:class|interface|enum)\s+(\w+)").unwrap(),
            Regex::new(r"public\s+(?:static\s+)?(?:final\s+)?[\w<>]+\s+(\w+)").unwrap(),
        ]);
        
        // C++ экспорты
        patterns.insert(FileType::Cpp, vec![
            Regex::new(r#"extern\s+"C"\s+.*?(\w+)"#).unwrap(),
            Regex::new(r"(?:class|struct|enum)\s+(\w+)").unwrap(),
        ]);
        
        // Go экспорты (начинаются с большой буквы)
        patterns.insert(FileType::Go, vec![
            Regex::new(r"(?:func|type|var|const)\s+([A-Z]\w*)").unwrap(),
        ]);
        
        patterns
    }
    
    pub fn build_graph(&mut self, capsules: &[Capsule]) -> Result<CapsuleGraph> {
        let mut capsule_map = HashMap::new();
        let mut layers: HashMap<String, Vec<Uuid>> = HashMap::new();
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut current_level: i32 = 0;
        
        // Добавляем капсулы в граф
        for capsule in capsules {
            capsule_map.insert(capsule.id, capsule.clone());
            
            // Группируем по слоям
            if let Some(layer) = &capsule.layer {
                layers.entry(layer.clone())
                    .or_insert_with(Vec::new)
                    .push(capsule.id);
            }
        }
        
        // Строим связи между капсулами с использованием продвинутого анализа
        let relations = self.build_advanced_relations(capsules)?;
        
        // Обновляем зависимости в капсулах
        let mut updated_capsules = self.update_capsule_dependencies(&capsule_map, &relations)?;
        
        // Вычисляем метрики графа
        let metrics = self.calculate_advanced_metrics(&updated_capsules, &relations)?;
        
        // Создаем граф
        let mut graph = CapsuleGraph {
            capsules: updated_capsules,
            relations,
            layers,
            metrics,
            created_at: chrono::Utc::now(),
            previous_analysis: None,
        };
        
        // Обнаруживаем циклы
        let cycles = self.cycle_detector.find_cycles(&graph);
        if !cycles.is_empty() {
            self.add_cycle_warnings(&mut graph, &cycles)?;
        }
        
        Ok(graph)
    }
    
    /// Строит продвинутые связи между капсулами
    fn build_advanced_relations(&mut self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        for capsule in capsules {
            // Связи через зависимости
            for dep_id in &capsule.dependencies {
                if capsules.iter().any(|c| &c.id == dep_id) {
                    relations.push(CapsuleRelation {
                        from_id: capsule.id,
                        to_id: *dep_id,
                        relation_type: RelationType::Depends,
                        strength: 0.8,
                        description: Some("Прямая зависимость".to_string()),
                    });
                }
            }
            
            // Связи через файловую структуру
            for other_capsule in capsules {
                if capsule.id != other_capsule.id {
                    if let Some(strength) = self.calculate_file_relation_strength(capsule, other_capsule) {
                        if strength > self.relation_strength_threshold {
                            relations.push(CapsuleRelation {
                                from_id: capsule.id,
                                to_id: other_capsule.id,
                                relation_type: RelationType::References,
                                strength,
                                description: Some("Связь через файловую структуру".to_string()),
                            });
                        }
                    }
                }
            }
            
            // Связи через архитектурные слои
            for other_capsule in capsules {
                if capsule.id != other_capsule.id {
                    if let Some(strength) = self.calculate_layer_relation_strength(capsule, other_capsule) {
                        if strength > self.relation_strength_threshold {
                            relations.push(CapsuleRelation {
                                from_id: capsule.id,
                                to_id: other_capsule.id,
                                relation_type: RelationType::Uses,
                                strength,
                                description: Some("Связь через архитектурный слой".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(relations)
    }
    
    fn calculate_file_relation_strength(&self, capsule1: &Capsule, capsule2: &Capsule) -> Option<f32> {
        let path1 = &capsule1.file_path;
        let path2 = &capsule2.file_path;
        
        // Если в том же каталоге - высокая связь
        if path1.parent() == path2.parent() {
            return Some(0.6);
        }
        
        // Если в соседних каталогах - средняя связь
        if let (Some(parent1), Some(parent2)) = (path1.parent(), path2.parent()) {
            if parent1.parent() == parent2.parent() {
                return Some(0.3);
            }
        }
        
        None
    }
    
    fn calculate_layer_relation_strength(&self, capsule1: &Capsule, capsule2: &Capsule) -> Option<f32> {
        match (&capsule1.layer, &capsule2.layer) {
            (Some(layer1), Some(layer2)) => {
                // Типичные архитектурные связи
                match (layer1.as_str(), layer2.as_str()) {
                    ("API", "Business") => Some(0.7),
                    ("Business", "Domain") => Some(0.8),
                    ("Domain", "Core") => Some(0.6),
                    ("UI", "API") => Some(0.5),
                    ("Utils", _) => Some(0.4),
                    (_, "Utils") => Some(0.3),
                    _ if layer1 == layer2 => Some(0.5),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    fn calculate_metrics(&self, capsules: &[Capsule], relations: &[CapsuleRelation]) -> Result<GraphMetrics> {
        let total_capsules = capsules.len();
        let total_relations = relations.len();
        
        // Средняя сложность
        let complexity_sum: u32 = capsules.iter().map(|c| c.complexity).sum();
        let complexity_average = if total_capsules > 0 {
            complexity_sum as f32 / total_capsules as f32
        } else {
            0.0
        };
        
        // Индекс связности (coupling)
        let coupling_index = if total_capsules > 1 {
            total_relations as f32 / (total_capsules * (total_capsules - 1)) as f32
        } else {
            0.0
        };

        // Преобразуем слайс в HashMap для совместимости с методами
        let capsules_map: HashMap<Uuid, Capsule> = capsules.iter().map(|c| (c.id, c.clone())).collect();
        
        // Индекс сплоченности (cohesion) - на основе групп связанных капсул
        let cohesion_index = self.calculate_cohesion_index(&capsules_map, relations);
        
        // Цикломатическая сложность графа
        let cyclomatic_complexity = self.calculate_graph_complexity(&capsules_map, relations);
        
        // Глубина уровней
        let depth_levels = self.calculate_depth_levels(&capsules_map, relations);
        
        Ok(GraphMetrics {
            total_capsules,
            total_relations,
            complexity_average,
            coupling_index,
            cohesion_index,
            cyclomatic_complexity,
            depth_levels,
        })
    }
    
        // Удален дубликат - используем версию с HashMap
    
    fn calculate_dependency_depth(&self, capsule_id: Uuid, relations: &[CapsuleRelation], visited: &mut Vec<Uuid>) -> u32 {
        if visited.contains(&capsule_id) {
            return 0; // Избегаем циклов
        }
        
        visited.push(capsule_id);
        
        let mut max_child_depth = 0;
        for relation in relations {
            if relation.from_id == capsule_id && relation.relation_type == RelationType::Depends {
                let child_depth = self.calculate_dependency_depth(relation.to_id, relations, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }
        
        visited.pop();
        1 + max_child_depth
    }
    
    /// Обнаруживает тип файла по расширению
    fn detect_file_type(&self, file_path: &Path) -> Result<FileType> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension {
            "rs" => Ok(FileType::Rust),
            "js" => Ok(FileType::JavaScript),
            "ts" | "tsx" => Ok(FileType::TypeScript),
            "py" => Ok(FileType::Python),
            "java" => Ok(FileType::Java),
            "cpp" | "cc" | "cxx" => Ok(FileType::Cpp),
            "c" => Ok(FileType::C),
            "go" => Ok(FileType::Go),
            _ => Ok(FileType::Other(extension.to_string())),
        }
    }
    
    /// Извлекает импорты из содержимого файла
    fn extract_imports(&self, content: &str, file_type: &FileType) -> Result<Vec<String>> {
        let mut imports = Vec::new();
        
        if let Some(patterns) = self.import_patterns.get(file_type) {
            for pattern in patterns {
                for captures in pattern.captures_iter(content) {
                    if let Some(import_match) = captures.get(1) {
                        imports.push(import_match.as_str().to_string());
                    }
                }
            }
        }
        
        Ok(imports)
    }
    
    /// Извлекает экспорты из содержимого файла
    fn extract_exports(&self, content: &str, file_type: &FileType) -> Result<Vec<String>> {
        let mut exports = Vec::new();
        
        if let Some(patterns) = self.export_patterns.get(file_type) {
            for pattern in patterns {
                for captures in pattern.captures_iter(content) {
                    if let Some(export_match) = captures.get(1) {
                        exports.push(export_match.as_str().to_string());
                    }
                }
            }
        }
        
        Ok(exports)
    }
    
    /// Вычисляет силу связи между импортами и экспортами
    fn calculate_connection_strength(&self, imports: &[String], exports: &[String]) -> f32 {
        let mut strength = 0.0;
        let mut matches = 0;
        
        for import in imports {
            for export in exports {
                if import.contains(export) || export.contains(import) {
                    matches += 1;
                    strength += 1.0;
                }
            }
        }
        
        if imports.is_empty() || exports.is_empty() {
            return 0.0;
        }
        
        strength / ((imports.len() + exports.len()) as f32)
    }
    
    /// Определяет тип связи между модулями
    fn determine_relation_type(&self, imports: &[String], exports: &[String], 
                              all_exports: &HashMap<Uuid, Vec<String>>, 
                              _all_imports: &HashMap<Uuid, Vec<String>>) -> Result<RelationType> {
        // Простая эвристика для определения типа связи
        if imports.iter().any(|i| exports.iter().any(|e| i.contains(e))) {
            return Ok(RelationType::Uses);
        }
        
        // Проверяем на наследование/реализацию
        if imports.iter().any(|i| i.contains("extends") || i.contains("implements") || i.contains("trait")) {
            return Ok(RelationType::Implements);
        }
        
        // Проверяем на композицию
        if imports.iter().any(|i| i.contains("new") || i.contains("create") || i.contains("build")) {
            return Ok(RelationType::Composes);
        }
        
        Ok(RelationType::Depends)
    }
    
    /// Анализирует связи наследования
    fn analyze_inheritance_relations(&self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        for capsule in capsules {
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                // Паттерны наследования для разных языков
                let inheritance_patterns = vec![
                    Regex::new(r"extends\s+(\w+)").unwrap(),
                    Regex::new(r"implements\s+(\w+)").unwrap(),
                    Regex::new(r":\s*(\w+)").unwrap(), // Rust trait implementation
                ];
                
                for pattern in &inheritance_patterns {
                    for captures in pattern.captures_iter(&content) {
                        if let Some(parent_name) = captures.get(1) {
                            // Ищем родительский класс среди капсул
                            if let Some(parent_capsule) = capsules.iter().find(|c| c.name == parent_name.as_str()) {
                                relations.push(CapsuleRelation {
                                    from_id: capsule.id,
                                    to_id: parent_capsule.id,
                                    relation_type: RelationType::Extends,
                                    strength: 0.9,
                                    description: Some(format!("Наследует от {}", parent_name.as_str())),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(relations)
    }
    
    /// Анализирует связи композиции
    fn analyze_composition_relations(&self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        for capsule in capsules {
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                // Паттерны композиции
                let composition_patterns = vec![
                    Regex::new(r"new\s+(\w+)\s*\(").unwrap(),
                    Regex::new(r"(\w+)\s*\{").unwrap(), // Rust struct initialization
                    Regex::new(r"(\w+)\.new\s*\(").unwrap(),
                ];
                
                for pattern in &composition_patterns {
                    for captures in pattern.captures_iter(&content) {
                        if let Some(component_name) = captures.get(1) {
                            // Ищем компонент среди капсул
                            if let Some(component_capsule) = capsules.iter().find(|c| c.name == component_name.as_str()) {
                                relations.push(CapsuleRelation {
                                    from_id: capsule.id,
                                    to_id: component_capsule.id,
                                    relation_type: RelationType::Composes,
                                    strength: 0.7,
                                    description: Some(format!("Композиция с {}", component_name.as_str())),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(relations)
    }
    
    /// Анализирует связи использования
    fn analyze_usage_relations(&self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
        let mut relations = Vec::new();
        
        for capsule in capsules {
            if let Ok(content) = std::fs::read_to_string(&capsule.file_path) {
                // Ищем упоминания других капсул в коде
                for other_capsule in capsules {
                    if capsule.id != other_capsule.id {
                        let usage_count = content.matches(&other_capsule.name).count();
                        if usage_count > 0 {
                            let strength = (usage_count as f32 / 10.0).min(1.0);
                            if strength > self.relation_strength_threshold {
                                relations.push(CapsuleRelation {
                                    from_id: capsule.id,
                                    to_id: other_capsule.id,
                                    relation_type: RelationType::Uses,
                                    strength,
                                    description: Some(format!("Использует {} раз", usage_count)),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(relations)
    }
    
    /// Обновляет зависимости в капсулах на основе найденных связей
    fn update_capsule_dependencies(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> Result<HashMap<Uuid, Capsule>> {
        let mut updated_capsules = capsules.clone();
        
        for relation in relations {
            // Обновляем зависимости
            if let Some(from_capsule) = updated_capsules.get_mut(&relation.from_id) {
                if !from_capsule.dependencies.contains(&relation.to_id) {
                    from_capsule.dependencies.push(relation.to_id);
                }
            }
            
            // Обновляем зависимые
            if let Some(to_capsule) = updated_capsules.get_mut(&relation.to_id) {
                if !to_capsule.dependents.contains(&relation.from_id) {
                    to_capsule.dependents.push(relation.from_id);
                }
            }
        }
        
        Ok(updated_capsules)
    }
    
    /// Вычисляет продвинутые метрики графа
    fn calculate_advanced_metrics(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> Result<GraphMetrics> {
        let total_capsules = capsules.len();
        let total_relations = relations.len();
        
        // Средняя сложность
        let complexity_sum: u32 = capsules.values().map(|c| c.complexity).sum();
        let complexity_average = if total_capsules > 0 {
            complexity_sum as f32 / total_capsules as f32
        } else {
            0.0
        };
        
        // Индекс связности (coupling) - учитывает силу связей
        let coupling_sum: f32 = relations.iter().map(|r| r.strength).sum();
        let coupling_index = if total_capsules > 1 {
            coupling_sum / (total_capsules * (total_capsules - 1)) as f32
        } else {
            0.0
        };
        
        // Индекс сплоченности (cohesion) - на основе групп связанных капсул
        let cohesion_index = self.calculate_cohesion_index(capsules, relations);
        
        // Цикломатическая сложность графа
        let cyclomatic_complexity = self.calculate_graph_complexity(capsules, relations);
        
        // Глубина уровней
        let depth_levels = self.calculate_depth_levels(capsules, relations);
        
        Ok(GraphMetrics {
            total_capsules,
            total_relations,
            complexity_average,
            coupling_index,
            cohesion_index,
            cyclomatic_complexity,
            depth_levels,
        })
    }
    
    /// Вычисляет глубину уровней в графе
    fn calculate_depth_levels(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> u32 {
        let mut max_depth = 0;
        
        for capsule_id in capsules.keys() {
            let depth = self.calculate_dependency_depth(*capsule_id, relations, &mut Vec::new());
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }
    
    /// Вычисляет индекс сплоченности (cohesion)
    fn calculate_cohesion_index(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> f32 {
        if capsules.is_empty() {
            return 0.0;
        }
        
        // Группируем капсулы по слоям
        let mut layer_groups: HashMap<String, Vec<Uuid>> = HashMap::new();
        
        for capsule in capsules.values() {
            if let Some(layer) = &capsule.layer {
                layer_groups.entry(layer.clone()).or_insert_with(Vec::new).push(capsule.id);
            }
        }
        
        // Вычисляем внутрислойные связи
        let mut total_internal_connections = 0;
        let mut total_possible_connections = 0;
        
        for group in layer_groups.values() {
            let group_size = group.len();
            if group_size > 1 {
                total_possible_connections += group_size * (group_size - 1);
                
                // Считаем реальные связи внутри группы
                for relation in relations {
                    if group.contains(&relation.from_id) && group.contains(&relation.to_id) {
                        total_internal_connections += 1;
                    }
                }
            }
        }
        
        if total_possible_connections == 0 {
            return 0.0;
        }
        
        total_internal_connections as f32 / total_possible_connections as f32
    }
    
    /// Вычисляет цикломатическую сложность графа
    fn calculate_graph_complexity(&self, capsules: &HashMap<Uuid, Capsule>, relations: &[CapsuleRelation]) -> u32 {
        let nodes = capsules.len() as u32;
        let edges = relations.len() as u32;
        
        // Приблизительная оценка количества компонент связности
        let mut components = 0;
        let mut visited: HashSet<Uuid> = HashSet::new();
        
        for capsule_id in capsules.keys() {
            if !visited.contains(capsule_id) {
                self.dfs_component_visit(*capsule_id, relations, &mut visited);
                components += 1;
            }
        }
        
        // Формула: E - N + 2P (где E - ребра, N - узлы, P - компоненты)
        if nodes > 0 {
            edges.saturating_sub(nodes) + 2 * components
        } else {
            0
        }
    }
    
    /// Обход компонент связности для расчета цикломатической сложности
    fn dfs_component_visit(&self, capsule_id: Uuid, relations: &[CapsuleRelation], visited: &mut HashSet<Uuid>) {
        visited.insert(capsule_id);
        
        // Ищем все связанные капсулы
        for relation in relations {
            let connected_id = if relation.from_id == capsule_id {
                relation.to_id
            } else if relation.to_id == capsule_id {
                relation.from_id
            } else {
                continue;
            };
            
            if !visited.contains(&connected_id) {
                self.dfs_component_visit(connected_id, relations, visited);
            }
        }
    }
    
    /// Добавляет предупреждения о циклах в граф
    fn add_cycle_warnings(&self, graph: &mut CapsuleGraph, cycles: &[Vec<Uuid>]) -> Result<()> {
        for cycle in cycles {
            for &capsule_id in cycle {
                if let Some(capsule) = graph.capsules.get_mut(&capsule_id) {
                    capsule.warnings.push(AnalysisWarning {
                        level: Priority::High,
                        message: format!(
                            "Участвует в циклической зависимости из {} элементов",
                            cycle.len()
                        ),
                        category: "dependencies".to_string(),
                        capsule_id: Some(capsule_id),
                        suggestion: Some("Разорвите цикл через инверсию зависимостей или выделение общего интерфейса".to_string()),
                    });
                }
            }
        }
        Ok(())
    }
}

impl Default for CapsuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
} 