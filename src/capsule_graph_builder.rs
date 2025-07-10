// #ДОДЕЛАТЬ: Реализация CapsuleGraphBuilder для построения графа капсул

use crate::core::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct CapsuleGraphBuilder {
    relation_strength_threshold: f32,
}

impl CapsuleGraphBuilder {
    pub fn new() -> Self {
        Self {
            relation_strength_threshold: 0.1,
        }
    }
    
    pub fn build_graph(&self, capsules: &[Capsule]) -> Result<CapsuleGraph> {
        let mut capsule_map = HashMap::new();
        let mut layers = HashMap::new();
        
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
        
        // Строим связи между капсулами
        let relations = self.build_relations(capsules)?;
        
        // Вычисляем метрики графа
        let metrics = self.calculate_metrics(capsules, &relations)?;
        
        Ok(CapsuleGraph {
            capsules: capsule_map,
            relations,
            layers,
            metrics,
            created_at: chrono::Utc::now(),
            previous_analysis: None,
        })
    }
    
    fn build_relations(&self, capsules: &[Capsule]) -> Result<Vec<CapsuleRelation>> {
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
    
    fn calculate_cohesion_index(&self, capsules: &[Capsule], relations: &[CapsuleRelation]) -> f32 {
        // Простая эвристика: группы капсул в одном слое должны быть сплоченными
        let mut layer_cohesion_sum = 0.0;
        let mut layer_count = 0;
        
        let mut layers: HashMap<String, Vec<&Capsule>> = HashMap::new();
        for capsule in capsules {
            if let Some(layer) = &capsule.layer {
                layers.entry(layer.clone()).or_default().push(capsule);
            }
        }
        
        for (_, layer_capsules) in layers {
            if layer_capsules.len() > 1 {
                let internal_relations = relations.iter()
                    .filter(|r| {
                        layer_capsules.iter().any(|c| c.id == r.from_id) &&
                        layer_capsules.iter().any(|c| c.id == r.to_id)
                    })
                    .count();
                
                let max_possible_relations = layer_capsules.len() * (layer_capsules.len() - 1);
                let cohesion = if max_possible_relations > 0 {
                    internal_relations as f32 / max_possible_relations as f32
                } else {
                    0.0
                };
                
                layer_cohesion_sum += cohesion;
                layer_count += 1;
            }
        }
        
        if layer_count > 0 {
            layer_cohesion_sum / layer_count as f32
        } else {
            0.0
        }
    }
    
    fn calculate_graph_complexity(&self, _capsules: &[Capsule], relations: &[CapsuleRelation]) -> u32 {
        // Упрощенная цикломатическая сложность: V(G) = E - N + 2P
        // где E = ребра, N = узлы, P = связные компоненты
        let edges = relations.len() as u32;
        let nodes = _capsules.len() as u32;
        let components = 1; // Упрощение: считаем один связный компонент
        
        if nodes > 0 {
            edges.saturating_sub(nodes) + 2 * components
        } else {
            0
        }
    }
    
    fn calculate_depth_levels(&self, capsules: &[Capsule], relations: &[CapsuleRelation]) -> u32 {
        // Находим максимальную глубину зависимостей
        let mut max_depth = 0;
        
        for capsule in capsules {
            let depth = self.calculate_dependency_depth(capsule.id, relations, &mut Vec::new());
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }
    
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
}

impl Default for CapsuleGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
} 