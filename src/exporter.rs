// #ДОДЕЛАТЬ: Реализация Exporter для экспорта в различные форматы

use crate::core::*;
use std::path::Path;
use serde_json;

/// Экспортер результатов анализа в различные форматы
#[derive(Debug)]
pub struct Exporter {
    #[allow(dead_code)]
    mermaid_theme: String,
}

impl Exporter {
    pub fn new() -> Self {
        Self {
            mermaid_theme: "default".to_string(),
        }
    }
    
    pub fn with_theme(theme: String) -> Self {
        Self {
            mermaid_theme: theme,
        }
    }
    
    pub fn export(&self, graph: &CapsuleGraph, format: ExportFormat, output_path: &Path) -> Result<String> {
        let content = match format {
            ExportFormat::Json => self.export_to_json(graph)?,
            ExportFormat::Yaml => self.export_to_yaml(graph)?,
            ExportFormat::Mermaid => self.export_to_mermaid(graph)?,
            ExportFormat::DOT => self.export_to_dot(graph)?,
            ExportFormat::GraphML => self.export_to_graphml(graph)?,
            ExportFormat::ChainOfThought => self.export_to_chain_of_thought(graph)?,
            ExportFormat::LLMPrompt => self.export_to_llm_prompt(graph)?,
        };
        
        std::fs::write(output_path, &content)?;
        Ok(content)
    }
    
    pub fn export_to_json(&self, graph: &CapsuleGraph) -> Result<String> {
        // Создаем упрощенную структуру для JSON
        let json_graph = JsonGraph::from_capsule_graph(graph);
        let json = serde_json::to_string_pretty(&json_graph)
            .map_err(|e| AnalysisError::Internal(format!("JSON serialization error: {}", e)))?;
        Ok(json)
    }
    
    pub fn export_to_yaml(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut yaml = String::new();
        
        yaml.push_str("# Архитектурный анализ проекта\n");
        yaml.push_str(&format!("created_at: '{}'\n", graph.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
        yaml.push_str("\n");
        
        // Метрики
        yaml.push_str("metrics:\n");
        yaml.push_str(&format!("  total_capsules: {}\n", graph.metrics.total_capsules));
        yaml.push_str(&format!("  total_relations: {}\n", graph.metrics.total_relations));
        yaml.push_str(&format!("  complexity_average: {:.2}\n", graph.metrics.complexity_average));
        yaml.push_str(&format!("  coupling_index: {:.2}\n", graph.metrics.coupling_index));
        yaml.push_str(&format!("  cohesion_index: {:.2}\n", graph.metrics.cohesion_index));
        yaml.push_str(&format!("  cyclomatic_complexity: {}\n", graph.metrics.cyclomatic_complexity));
        yaml.push_str(&format!("  depth_levels: {}\n", graph.metrics.depth_levels));
        yaml.push_str("\n");
        
        // Слои
        yaml.push_str("layers:\n");
        for (layer_name, capsule_ids) in &graph.layers {
            yaml.push_str(&format!("  {}:\n", layer_name));
            yaml.push_str(&format!("    count: {}\n", capsule_ids.len()));
            yaml.push_str("    capsules:\n");
            for capsule_id in capsule_ids {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    yaml.push_str(&format!("      - name: '{}'\n", capsule.name));
                    yaml.push_str(&format!("        type: '{:?}'\n", capsule.capsule_type));
                    yaml.push_str(&format!("        complexity: {}\n", capsule.complexity));
                    yaml.push_str(&format!("        path: '{}'\n", capsule.file_path.display()));
                }
            }
        }
        yaml.push_str("\n");
        
        // Связи
        yaml.push_str("relations:\n");
        for relation in &graph.relations {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                yaml.push_str(&format!("  - from: '{}'\n", from_capsule.name));
                yaml.push_str(&format!("    to: '{}'\n", to_capsule.name));
                yaml.push_str(&format!("    type: '{:?}'\n", relation.relation_type));
                yaml.push_str(&format!("    strength: {:.2}\n", relation.strength));
                if let Some(desc) = &relation.description {
                    yaml.push_str(&format!("    description: '{}'\n", desc));
                }
            }
        }
        
        Ok(yaml)
    }
    
    pub fn export_to_mermaid(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut mermaid = String::new();
        
        mermaid.push_str("graph TD\n");
        mermaid.push_str(&format!("    %% Архитектурная диаграмма ({} компонентов)\n", graph.capsules.len()));
        mermaid.push_str("\n");
        
        // Определяем стили для разных типов капсул
        mermaid.push_str("    %% Стили компонентов\n");
        mermaid.push_str("    classDef moduleClass fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        mermaid.push_str("    classDef functionClass fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
        mermaid.push_str("    classDef structClass fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px\n");
        mermaid.push_str("    classDef classClass fill:#fff3e0,stroke:#e65100,stroke-width:2px\n");
        mermaid.push_str("\n");
        
        // Группируем по слоям
        for (layer_name, capsule_ids) in &graph.layers {
            mermaid.push_str(&format!("    subgraph \"Слой: {}\"\n", layer_name));
            
            for capsule_id in capsule_ids {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    let node_id = self.sanitize_node_id(&capsule.name);
                    let display_name = self.truncate_name(&capsule.name, 20);
                    
                    match capsule.capsule_type {
                        CapsuleType::Module => {
                            mermaid.push_str(&format!("        {}[\"📦 {}\"]\n", node_id, display_name));
                            mermaid.push_str(&format!("        {}:::moduleClass\n", node_id));
                        }
                        CapsuleType::Function | CapsuleType::Method => {
                            mermaid.push_str(&format!("        {}[\"⚙️ {}\"]\n", node_id, display_name));
                            mermaid.push_str(&format!("        {}:::functionClass\n", node_id));
                        }
                        CapsuleType::Struct | CapsuleType::Enum => {
                            mermaid.push_str(&format!("        {}[\"🏗️ {}\"]\n", node_id, display_name));
                            mermaid.push_str(&format!("        {}:::structClass\n", node_id));
                        }
                        CapsuleType::Class | CapsuleType::Interface => {
                            mermaid.push_str(&format!("        {}[\"🎯 {}\"]\n", node_id, display_name));
                            mermaid.push_str(&format!("        {}:::classClass\n", node_id));
                        }
                        _ => {
                            mermaid.push_str(&format!("        {}[\"⚪ {}\"]\n", node_id, display_name));
                        }
                    }
                }
            }
            
            mermaid.push_str("    end\n\n");
        }
        
        // Добавляем связи
        mermaid.push_str("    %% Связи между компонентами\n");
        for relation in &graph.relations {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                
                let from_id = self.sanitize_node_id(&from_capsule.name);
                let to_id = self.sanitize_node_id(&to_capsule.name);
                
                let arrow_style = match relation.relation_type {
                    RelationType::Depends => "-->",
                    RelationType::Uses => "-.->",
                    RelationType::Implements => "==>",
                    RelationType::Extends => "===>",
                    RelationType::Aggregates => "--o",
                    RelationType::Composes => "-->",
                    RelationType::Calls => "-.->",
                    RelationType::References => "-.->",
                };
                
                let label = if relation.strength > 0.7 { "strong" } else if relation.strength > 0.4 { "medium" } else { "weak" };
                mermaid.push_str(&format!("    {} {}|{}| {}\n", from_id, arrow_style, label, to_id));
            }
        }
        
        Ok(mermaid)
    }
    
    pub fn export_to_dot(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut dot = String::new();
        
        dot.push_str("digraph architecture {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=box, style=filled];\n");
        dot.push_str("    edge [fontsize=10];\n\n");
        
        // Определяем цвета для типов
        dot.push_str("    // Стили узлов\n");
        for capsule in graph.capsules.values() {
            let color = match capsule.capsule_type {
                CapsuleType::Module => "lightblue",
                CapsuleType::Function | CapsuleType::Method => "lightgreen",
                CapsuleType::Struct | CapsuleType::Enum => "lightyellow",
                CapsuleType::Class | CapsuleType::Interface => "lightcoral",
                _ => "lightgray",
            };
            
            let node_id = self.sanitize_node_id(&capsule.name);
            dot.push_str(&format!("    \"{}\" [fillcolor={}, label=\"{}\"];\n", 
                                 node_id, color, self.escape_label(&capsule.name)));
        }
        
        dot.push_str("\n    // Связи\n");
        for relation in &graph.relations {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                
                let from_id = self.sanitize_node_id(&from_capsule.name);
                let to_id = self.sanitize_node_id(&to_capsule.name);
                
                let style = match relation.relation_type {
                    RelationType::Depends => "solid",
                    RelationType::Uses => "dashed",
                    RelationType::Implements => "bold",
                    _ => "dotted",
                };
                
                dot.push_str(&format!("    \"{}\" -> \"{}\" [style={}, label=\"{:.1}\"];\n", 
                                     from_id, to_id, style, relation.strength));
            }
        }
        
        dot.push_str("}\n");
        Ok(dot)
    }
    
    pub fn export_to_graphml(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut graphml = String::new();
        
        graphml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        graphml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
        graphml.push_str("  <key id=\"name\" for=\"node\" attr.name=\"name\" attr.type=\"string\"/>\n");
        graphml.push_str("  <key id=\"type\" for=\"node\" attr.name=\"type\" attr.type=\"string\"/>\n");
        graphml.push_str("  <key id=\"complexity\" for=\"node\" attr.name=\"complexity\" attr.type=\"int\"/>\n");
        graphml.push_str("  <key id=\"relation_type\" for=\"edge\" attr.name=\"relation_type\" attr.type=\"string\"/>\n");
        graphml.push_str("  <key id=\"strength\" for=\"edge\" attr.name=\"strength\" attr.type=\"double\"/>\n");
        graphml.push_str("  <graph id=\"architecture\" edgedefault=\"directed\">\n");
        
        // Узлы
        for capsule in graph.capsules.values() {
            graphml.push_str(&format!("    <node id=\"{}\">\n", capsule.id));
            graphml.push_str(&format!("      <data key=\"name\">{}</data>\n", self.escape_xml(&capsule.name)));
            graphml.push_str(&format!("      <data key=\"type\">{:?}</data>\n", capsule.capsule_type));
            graphml.push_str(&format!("      <data key=\"complexity\">{}</data>\n", capsule.complexity));
            graphml.push_str("    </node>\n");
        }
        
        // Ребра
        for relation in &graph.relations {
            graphml.push_str(&format!("    <edge source=\"{}\" target=\"{}\">\n", 
                                     relation.from_id, relation.to_id));
            graphml.push_str(&format!("      <data key=\"relation_type\">{:?}</data>\n", relation.relation_type));
            graphml.push_str(&format!("      <data key=\"strength\">{}</data>\n", relation.strength));
            graphml.push_str("    </edge>\n");
        }
        
        graphml.push_str("  </graph>\n");
        graphml.push_str("</graphml>\n");
        Ok(graphml)
    }
    
    pub fn export_to_chain_of_thought(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut cot = String::new();
        
        cot.push_str("# Chain of Thought: Архитектурный анализ\n\n");
        
        cot.push_str("## 🎯 Цель анализа\n");
        cot.push_str("Провести детальный анализ архитектуры системы для выявления:\n");
        cot.push_str("- Структурных паттернов и зависимостей\n");
        cot.push_str("- Потенциальных проблем качества кода\n");
        cot.push_str("- Возможностей для оптимизации\n\n");
        
        cot.push_str("## 📊 Статистика системы\n");
        cot.push_str(&format!("- **Общее количество компонентов**: {}\n", graph.metrics.total_capsules));
        cot.push_str(&format!("- **Связей между компонентами**: {}\n", graph.metrics.total_relations));
        cot.push_str(&format!("- **Средняя сложность**: {:.2}\n", graph.metrics.complexity_average));
        cot.push_str(&format!("- **Индекс связности**: {:.2}\n", graph.metrics.coupling_index));
        cot.push_str(&format!("- **Индекс сплоченности**: {:.2}\n", graph.metrics.cohesion_index));
        cot.push_str(&format!("- **Глубина зависимостей**: {}\n", graph.metrics.depth_levels));
        cot.push_str("\n");
        
        cot.push_str("## 🏗️ Архитектурные слои\n");
        for (layer_name, capsule_ids) in &graph.layers {
            cot.push_str(&format!("### Слой: {}\n", layer_name));
            cot.push_str(&format!("Компонентов в слое: {}\n\n", capsule_ids.len()));
            
            cot.push_str("**Ключевые компоненты:**\n");
            for capsule_id in capsule_ids.iter().take(5) {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    cot.push_str(&format!("- `{}` ({:?}, сложность: {})\n", 
                                         capsule.name, capsule.capsule_type, capsule.complexity));
                }
            }
            cot.push_str("\n");
        }
        
        cot.push_str("## 🔗 Критические связи\n");
        let important_relations: Vec<_> = graph.relations.iter()
            .filter(|r| r.strength > 0.7)
            .collect();
        
        for relation in important_relations.iter().take(10) {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                cot.push_str(&format!("- `{}` → `{}` ({:?}, сила: {:.2})\n", 
                                     from_capsule.name, to_capsule.name, 
                                     relation.relation_type, relation.strength));
            }
        }
        cot.push_str("\n");
        
        cot.push_str("## 💡 Ключевые выводы\n");
        cot.push_str("1. **Структурная сложность**: ");
        if graph.metrics.complexity_average > 15.0 {
            cot.push_str("Высокая. Рекомендуется рефакторинг.\n");
        } else if graph.metrics.complexity_average > 8.0 {
            cot.push_str("Умеренная. Следите за ростом сложности.\n");
        } else {
            cot.push_str("Низкая. Архитектура хорошо структурирована.\n");
        }
        
        cot.push_str("2. **Связанность компонентов**: ");
        if graph.metrics.coupling_index > 0.7 {
            cot.push_str("Высокая. Система тесно связана, что может затруднить изменения.\n");
        } else if graph.metrics.coupling_index > 0.4 {
            cot.push_str("Умеренная. Баланс между связанностью и модульностью.\n");
        } else {
            cot.push_str("Низкая. Хорошая модульность архитектуры.\n");
        }
        
        cot.push_str("3. **Сплоченность модулей**: ");
        if graph.metrics.cohesion_index < 0.3 {
            cot.push_str("Низкая. Компоненты слабо связаны функционально.\n");
        } else if graph.metrics.cohesion_index < 0.6 {
            cot.push_str("Умеренная. Есть место для улучшения.\n");
        } else {
            cot.push_str("Высокая. Компоненты хорошо организованы.\n");
        }
        
        Ok(cot)
    }
    
    pub fn export_to_llm_prompt(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut prompt = String::new();
        
        prompt.push_str("# Архитектурный анализ для LLM\n\n");
        prompt.push_str("Ты архитектор программного обеспечения. Проанализируй следующую архитектуру и предложи улучшения.\n\n");
        
        prompt.push_str("## Структура системы\n");
        prompt.push_str(&format!("Система содержит {} компонентов, организованных в {} слоёв:\n\n", 
                                graph.metrics.total_capsules, graph.layers.len()));
        
        for (layer_name, capsule_ids) in &graph.layers {
            prompt.push_str(&format!("**{}** ({} компонентов):\n", layer_name, capsule_ids.len()));
            for capsule_id in capsule_ids.iter().take(3) {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    prompt.push_str(&format!("- {}: {:?} (сложность {})\n", 
                                           capsule.name, capsule.capsule_type, capsule.complexity));
                }
            }
            prompt.push_str("\n");
        }
        
        prompt.push_str("## Метрики качества\n");
        prompt.push_str(&format!("- Средняя сложность: {:.2}\n", graph.metrics.complexity_average));
        prompt.push_str(&format!("- Связанность: {:.2}\n", graph.metrics.coupling_index));
        prompt.push_str(&format!("- Сплоченность: {:.2}\n", graph.metrics.cohesion_index));
        prompt.push_str(&format!("- Глубина зависимостей: {}\n", graph.metrics.depth_levels));
        prompt.push_str("\n");
        
        prompt.push_str("## Задачи для анализа\n");
        prompt.push_str("1. Оцени архитектурные паттерны\n");
        prompt.push_str("2. Выяви потенциальные проблемы\n");
        prompt.push_str("3. Предложи конкретные улучшения\n");
        prompt.push_str("4. Оцени соответствие принципам SOLID\n");
        prompt.push_str("5. Дай рекомендации по рефакторингу\n\n");
        
        prompt.push_str("## Контекст\n");
        prompt.push_str("Это современное приложение с требованиями к масштабируемости, ");
        prompt.push_str("поддерживаемости и производительности. ");
        prompt.push_str("Команда разработки состоит из 3-5 разработчиков.\n\n");
        
        prompt.push_str("Твой ответ должен содержать:\n");
        prompt.push_str("- Оценку текущего состояния (1-10)\n");
        prompt.push_str("- 3-5 конкретных рекомендаций\n");
        prompt.push_str("- Приоритизацию изменений\n");
        prompt.push_str("- Оценку рисков\n");
        
        Ok(prompt)
    }
    
    // Вспомогательные методы
    fn sanitize_node_id(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
    
    fn truncate_name(&self, name: &str, max_len: usize) -> String {
        if name.len() <= max_len {
            name.to_string()
        } else {
            format!("{}...", &name[..max_len-3])
        }
    }
    
    fn escape_label(&self, text: &str) -> String {
        text.replace("\"", "\\\"")
            .replace("\n", "\\n")
    }
    
    fn escape_xml(&self, text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&apos;")
    }
}

// Структура для JSON экспорта
#[derive(serde::Serialize)]
struct JsonGraph {
    created_at: String,
    metrics: JsonMetrics,
    layers: std::collections::HashMap<String, Vec<JsonCapsule>>,
    relations: Vec<JsonRelation>,
}

#[derive(serde::Serialize)]
struct JsonMetrics {
    total_capsules: usize,
    total_relations: usize,
    complexity_average: f32,
    coupling_index: f32,
    cohesion_index: f32,
    cyclomatic_complexity: u32,
    depth_levels: u32,
}

#[derive(serde::Serialize)]
struct JsonCapsule {
    id: String,
    name: String,
    capsule_type: String,
    complexity: u32,
    file_path: String,
    warnings: Vec<String>,
}

#[derive(serde::Serialize)]
struct JsonRelation {
    from: String,
    to: String,
    relation_type: String,
    strength: f32,
    description: Option<String>,
}

impl JsonGraph {
    fn from_capsule_graph(graph: &CapsuleGraph) -> Self {
        let mut layers = std::collections::HashMap::new();
        
        for (layer_name, capsule_ids) in &graph.layers {
            let layer_capsules: Vec<JsonCapsule> = capsule_ids.iter()
                .filter_map(|id| graph.capsules.get(id))
                .map(|capsule| JsonCapsule {
                    id: capsule.id.to_string(),
                    name: capsule.name.clone(),
                    capsule_type: format!("{:?}", capsule.capsule_type),
                    complexity: capsule.complexity,
                    file_path: capsule.file_path.display().to_string(),
                    warnings: capsule.warnings.clone(),
                })
                .collect();
            layers.insert(layer_name.clone(), layer_capsules);
        }
        
        let relations: Vec<JsonRelation> = graph.relations.iter()
            .filter_map(|relation| {
                let from_name = graph.capsules.get(&relation.from_id)?.name.clone();
                let to_name = graph.capsules.get(&relation.to_id)?.name.clone();
                Some(JsonRelation {
                    from: from_name,
                    to: to_name,
                    relation_type: format!("{:?}", relation.relation_type),
                    strength: relation.strength,
                    description: relation.description.clone(),
                })
            })
            .collect();
        
        Self {
            created_at: graph.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            metrics: JsonMetrics {
                total_capsules: graph.metrics.total_capsules,
                total_relations: graph.metrics.total_relations,
                complexity_average: graph.metrics.complexity_average,
                coupling_index: graph.metrics.coupling_index,
                cohesion_index: graph.metrics.cohesion_index,
                cyclomatic_complexity: graph.metrics.cyclomatic_complexity,
                depth_levels: graph.metrics.depth_levels,
            },
            layers,
            relations,
        }
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new()
    }
} 