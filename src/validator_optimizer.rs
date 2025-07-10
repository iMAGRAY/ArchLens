// #ДОДЕЛАТЬ: Реализация ValidatorOptimizer для валидации и оптимизации графа

use crate::core::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Валидатор и оптимизатор графа капсул
#[derive(Debug)]
pub struct ValidatorOptimizer {
    #[allow(dead_code)]
    min_capsule_complexity: u32,
    max_complexity_threshold: u32,
    #[allow(dead_code)]
    max_depth_threshold: u32,
    coupling_threshold: f32,
    cohesion_threshold: f32,
}

impl ValidatorOptimizer {
    pub fn new() -> Self {
        Self {
            min_capsule_complexity: 5,
            max_complexity_threshold: 15,
            max_depth_threshold: 8,
            coupling_threshold: 0.7,
            cohesion_threshold: 0.3,
        }
    }
    
    pub fn validate_and_optimize(&self, graph: &CapsuleGraph) -> Result<CapsuleGraph> {
        let mut optimized_graph = graph.clone();
        let mut warnings = Vec::new();
        
        // Валидация архитектуры
        self.validate_complexity(&optimized_graph, &mut warnings)?;
        self.validate_coupling(&optimized_graph, &mut warnings)?;
        self.validate_cohesion(&optimized_graph, &mut warnings)?;
        self.validate_dependency_cycles(&optimized_graph, &mut warnings)?;
        self.validate_layer_violations(&optimized_graph, &mut warnings)?;
        self.validate_naming_conventions(&optimized_graph, &mut warnings)?;
        
        // Оптимизация структуры
        self.optimize_relations(&mut optimized_graph)?;
        self.suggest_refactoring(&optimized_graph, &mut warnings)?;
        
        // Обновляем капсулы с предупреждениями
        self.distribute_warnings_to_capsules(&mut optimized_graph, warnings)?;
        
        Ok(optimized_graph)
    }
    
    fn validate_complexity(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        // Проверка общей сложности системы
        if graph.metrics.complexity_average > self.max_complexity_threshold as f32 {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!(
                    "Высокая средняя сложность системы: {:.2}. Рекомендуется разбиение на более простые компоненты.",
                    graph.metrics.complexity_average
                ),
                capsule_id: None,
                suggestion: Some("Выделите общую функциональность в отдельные модули".to_string()),
            });
        }
        
        // Проверка отдельных капсул
        for capsule in graph.capsules.values() {
            if capsule.complexity > self.max_complexity_threshold {
                warnings.push(AnalysisWarning {
                    level: Priority::Medium,
                    message: format!(
                        "Компонент '{}' имеет высокую сложность: {}",
                        capsule.name, capsule.complexity
                    ),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Рассмотрите разбиение на более мелкие функции".to_string()),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_coupling(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        if graph.metrics.coupling_index > self.coupling_threshold {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!(
                    "Высокий уровень связанности: {:.2}. Система слишком тесно связана.",
                    graph.metrics.coupling_index
                ),
                capsule_id: None,
                suggestion: Some("Используйте инверсию зависимостей и интерфейсы".to_string()),
            });
        }
        
        // Анализ конкретных узлов с высокой связанностью
        let mut coupling_counts: HashMap<Uuid, usize> = HashMap::new();
        for relation in &graph.relations {
            *coupling_counts.entry(relation.from_id).or_insert(0) += 1;
            *coupling_counts.entry(relation.to_id).or_insert(0) += 1;
        }
        
        for (capsule_id, count) in coupling_counts {
            if count > 10 {
                if let Some(capsule) = graph.capsules.get(&capsule_id) {
                    warnings.push(AnalysisWarning {
                        level: Priority::Medium,
                        message: format!(
                            "Компонент '{}' имеет слишком много связей: {}",
                            capsule.name, count
                        ),
                        capsule_id: Some(capsule_id),
                        suggestion: Some("Рассмотрите применение паттерна Facade".to_string()),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_cohesion(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        if graph.metrics.cohesion_index < self.cohesion_threshold {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!(
                    "Низкая сплоченность системы: {:.2}. Компоненты слабо связаны функционально.",
                    graph.metrics.cohesion_index
                ),
                capsule_id: None,
                suggestion: Some("Группируйте связанную функциональность в модули".to_string()),
            });
        }
        
        // Анализ сплоченности по слоям
        for (layer_name, capsule_ids) in &graph.layers {
            if capsule_ids.len() < 2 { continue; }
            
            let layer_relations = graph.relations.iter()
                .filter(|r| capsule_ids.contains(&r.from_id) && capsule_ids.contains(&r.to_id))
                .count();
            
            let max_possible = capsule_ids.len() * (capsule_ids.len() - 1);
            let cohesion = layer_relations as f32 / max_possible as f32;
            
            if cohesion < self.cohesion_threshold {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: format!(
                        "Слой '{}' имеет низкую внутреннюю связность: {:.2}",
                        layer_name, cohesion
                    ),
                    capsule_id: None,
                    suggestion: Some(format!("Пересмотрите архитектуру слоя '{}'", layer_name)),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_dependency_cycles(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        let cycles = self.find_dependency_cycles(graph);
        
        for cycle in cycles {
            let cycle_names: Vec<String> = cycle.iter()
                .filter_map(|id| graph.capsules.get(id))
                .map(|c| c.name.clone())
                .collect();
            
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!(
                    "Обнаружен цикл зависимостей: {}",
                    cycle_names.join(" -> ")
                ),
                capsule_id: cycle.first().copied(),
                suggestion: Some("Разорвите цикл через инверсию зависимостей".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn validate_layer_violations(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        // Определяем иерархию слоев
        let layer_hierarchy = self.get_layer_hierarchy();
        
        for relation in &graph.relations {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                
                if let (Some(from_layer), Some(to_layer)) = (&from_capsule.layer, &to_capsule.layer) {
                    if self.violates_layer_hierarchy(from_layer, to_layer, &layer_hierarchy) {
                        warnings.push(AnalysisWarning {
                            level: Priority::Medium,
                            message: format!(
                                "Нарушение архитектурных слоев: '{}' ({}) -> '{}' ({})",
                                from_capsule.name, from_layer,
                                to_capsule.name, to_layer
                            ),
                            capsule_id: Some(relation.from_id),
                            suggestion: Some("Используйте правильную направленность зависимостей".to_string()),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_naming_conventions(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        for capsule in graph.capsules.values() {
            // Проверка длины имен
            if capsule.name.len() < 3 {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: format!("Слишком короткое имя: '{}'", capsule.name),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Используйте более описательные имена".to_string()),
                });
            }
            
            if capsule.name.len() > 50 {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: format!("Слишком длинное имя: '{}'", capsule.name),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Сократите имя, сохранив смысл".to_string()),
                });
            }
            
            // Проверка соглашений именования
            match capsule.capsule_type {
                CapsuleType::Struct | CapsuleType::Class | CapsuleType::Interface => {
                    if !capsule.name.chars().next().unwrap_or('a').is_uppercase() {
                        warnings.push(AnalysisWarning {
                            level: Priority::Low,
                            message: format!("Тип '{}' должен начинаться с заглавной буквы", capsule.name),
                            capsule_id: Some(capsule.id),
                            suggestion: Some("Следуйте соглашениям именования".to_string()),
                        });
                    }
                }
                CapsuleType::Function | CapsuleType::Method => {
                    if capsule.name.chars().next().unwrap_or('A').is_uppercase() &&
                       !capsule.name.starts_with("Test") {
                        warnings.push(AnalysisWarning {
                            level: Priority::Low,
                            message: format!("Функция '{}' должна начинаться с строчной буквы", capsule.name),
                            capsule_id: Some(capsule.id),
                            suggestion: Some("Следуйте соглашениям именования".to_string()),
                        });
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn optimize_relations(&self, graph: &mut CapsuleGraph) -> Result<()> {
        // Удаляем слабые связи
        graph.relations.retain(|relation| relation.strength > 0.2);
        
        // Укрепляем важные связи
        for relation in &mut graph.relations {
            if relation.relation_type == RelationType::Depends {
                relation.strength = (relation.strength * 1.2).min(1.0);
            }
        }
        
        Ok(())
    }
    
    fn suggest_refactoring(&self, graph: &CapsuleGraph, warnings: &mut Vec<AnalysisWarning>) -> Result<()> {
        // Предложения по рефакторингу больших файлов
        for capsule in graph.capsules.values() {
            if capsule.line_end - capsule.line_start > 500 {
                warnings.push(AnalysisWarning {
                    level: Priority::Medium,
                    message: format!(
                        "Большой файл '{}' ({} строк). Рассмотрите разделение.",
                        capsule.name, capsule.line_end - capsule.line_start
                    ),
                    capsule_id: Some(capsule.id),
                    suggestion: Some("Выделите отдельные ответственности в модули".to_string()),
                });
            }
        }
        
        // Предложения по созданию интерфейсов
        let high_coupling_capsules: Vec<_> = graph.capsules.values()
            .filter(|c| {
                let relation_count = graph.relations.iter()
                    .filter(|r| r.from_id == c.id || r.to_id == c.id)
                    .count();
                relation_count > 8
            })
            .collect();
        
        for capsule in high_coupling_capsules {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!(
                    "Компонент '{}' имеет много связей. Рассмотрите введение интерфейса.",
                    capsule.name
                ),
                capsule_id: Some(capsule.id),
                suggestion: Some("Создайте абстракцию для снижения связанности".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn distribute_warnings_to_capsules(&self, graph: &mut CapsuleGraph, warnings: Vec<AnalysisWarning>) -> Result<()> {
        for warning in warnings {
            if let Some(capsule_id) = warning.capsule_id {
                if let Some(capsule) = graph.capsules.get_mut(&capsule_id) {
                    capsule.warnings.push(warning.message);
                }
            }
        }
        Ok(())
    }
    
    fn find_dependency_cycles(&self, graph: &CapsuleGraph) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for capsule_id in graph.capsules.keys() {
            if !visited.contains(capsule_id) {
                let mut path = Vec::new();
                if self.has_cycle_dfs(*capsule_id, graph, &mut visited, &mut rec_stack, &mut path) {
                    cycles.push(path);
                }
            }
        }
        
        cycles
    }
    
    fn has_cycle_dfs(
        &self,
        capsule_id: Uuid,
        graph: &CapsuleGraph,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> bool {
        visited.insert(capsule_id);
        rec_stack.insert(capsule_id);
        path.push(capsule_id);
        
        for relation in &graph.relations {
            if relation.from_id == capsule_id && relation.relation_type == RelationType::Depends {
                let target = relation.to_id;
                
                if rec_stack.contains(&target) {
                    return true;
                }
                
                if !visited.contains(&target) && 
                   self.has_cycle_dfs(target, graph, visited, rec_stack, path) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(&capsule_id);
        path.pop();
        false
    }
    
    fn get_layer_hierarchy(&self) -> HashMap<String, usize> {
        let mut hierarchy = HashMap::new();
        hierarchy.insert("UI".to_string(), 0);
        hierarchy.insert("API".to_string(), 1);
        hierarchy.insert("Business".to_string(), 2);
        hierarchy.insert("Domain".to_string(), 3);
        hierarchy.insert("Core".to_string(), 4);
        hierarchy.insert("Utils".to_string(), 5);
        hierarchy
    }
    
    fn violates_layer_hierarchy(&self, from_layer: &str, to_layer: &str, hierarchy: &HashMap<String, usize>) -> bool {
        if let (Some(&from_level), Some(&to_level)) = (hierarchy.get(from_layer), hierarchy.get(to_layer)) {
            // Нарушение: зависимость от слоя выше в иерархии (кроме Utils)
            to_level < from_level && to_layer != "Utils"
        } else {
            false
        }
    }
}

impl Default for ValidatorOptimizer {
    fn default() -> Self {
        Self::new()
    }
} 