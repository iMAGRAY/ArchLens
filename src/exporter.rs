use crate::types::*;
use std::path::Path;
use uuid::Uuid;
use serde_json;
use std::collections::HashMap;
use crate::types::Result;

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
    
    /// Основной метод экспорта
    pub fn export(&self, graph: &CapsuleGraph, format: ExportFormat, output_path: &Path) -> Result<String> {
        let content = match format {
            ExportFormat::JSON => self.export_to_json(graph)?,
            ExportFormat::YAML => self.export_to_yaml(graph)?,
            ExportFormat::Mermaid => self.export_to_mermaid(graph)?,
            ExportFormat::DOT => self.export_to_dot(graph)?,
            ExportFormat::GraphML => self.export_to_graphml(graph)?,
            ExportFormat::SVG => self.export_to_svg(graph)?,
            ExportFormat::InteractiveHTML => self.export_to_interactive_html(graph)?,
            ExportFormat::ChainOfThought => self.export_to_chain_of_thought(graph)?,
            ExportFormat::LLMPrompt => self.export_to_llm_prompt(graph)?,
            ExportFormat::AICompact => self.export_to_ai_compact(graph)?,
        };
        
        std::fs::write(output_path, &content)?;
        Ok(content)
    }

    /// Экспорт в JSON формат
    pub fn export_to_json(&self, graph: &CapsuleGraph) -> Result<String> {
        let json_graph = JsonGraph::from_capsule_graph(graph);
        let json = serde_json::to_string_pretty(&json_graph)
            .map_err(|e| AnalysisError::GenericError(format!("JSON serialization error: {e}")))?;
        Ok(json)
    }
    
    pub fn export_to_yaml(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut yaml = String::new();
        
        yaml.push_str("# Архитектурный анализ проекта\n");
        yaml.push_str(&format!("created_at: '{}'\n", graph.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
        yaml.push('\n');
        
        // Метрики
        yaml.push_str("metrics:\n");
        yaml.push_str(&format!("  total_capsules: {}\n", graph.metrics.total_capsules));
        yaml.push_str(&format!("  total_relations: {}\n", graph.metrics.total_relations));
        yaml.push_str(&format!("  complexity_average: {:.2}\n", graph.metrics.complexity_average));
        yaml.push_str(&format!("  coupling_index: {:.2}\n", graph.metrics.coupling_index));
        yaml.push_str(&format!("  cohesion_index: {:.2}\n", graph.metrics.cohesion_index));
        yaml.push_str(&format!("  cyclomatic_complexity: {}\n", graph.metrics.cyclomatic_complexity));
        yaml.push_str(&format!("  depth_levels: {}\n", graph.metrics.depth_levels));
        yaml.push('\n');
        
        // Слои
        yaml.push_str("layers:\n");
        for (layer_name, capsule_ids) in &graph.layers {
            yaml.push_str(&format!("  {layer_name}:\n"));
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
        yaml.push('\n');
        
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
                    yaml.push_str(&format!("    description: '{desc}'\n"));
                }
            }
        }
        
        Ok(yaml)
    }
    
    pub fn export_to_mermaid(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut mermaid = String::new();
        
        mermaid.push_str("graph TD\n");
        mermaid.push_str(&format!("    %% Архитектурная диаграмма ({} компонентов)\n", graph.capsules.len()));
        mermaid.push('\n');
        
        // Определяем стили для разных типов капсул
        mermaid.push_str("    %% Стили компонентов\n");
        mermaid.push_str("    classDef moduleClass fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        mermaid.push_str("    classDef functionClass fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
        mermaid.push_str("    classDef structClass fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px\n");
        mermaid.push_str("    classDef classClass fill:#fff3e0,stroke:#e65100,stroke-width:2px\n");
        mermaid.push('\n');
        
        // Группируем по слоям
        for (layer_name, capsule_ids) in &graph.layers {
            mermaid.push_str(&format!("    subgraph \"Слой: {layer_name}\"\n"));
            
            for capsule_id in capsule_ids {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    let node_id = self.sanitize_node_id(&capsule.name);
                    let display_name = self.truncate_name(&capsule.name, 20);
                    
                    match capsule.capsule_type {
                        CapsuleType::Module => {
                            mermaid.push_str(&format!("        {node_id}[\"📦 {display_name}\"]\n"));
                            mermaid.push_str(&format!("        {node_id}:::moduleClass\n"));
                        }
                        CapsuleType::Function | CapsuleType::Method => {
                            mermaid.push_str(&format!("        {node_id}[\"⚙️ {display_name}\"]\n"));
                            mermaid.push_str(&format!("        {node_id}:::functionClass\n"));
                        }
                        CapsuleType::Struct | CapsuleType::Enum => {
                            mermaid.push_str(&format!("        {node_id}[\"🏗️ {display_name}\"]\n"));
                            mermaid.push_str(&format!("        {node_id}:::structClass\n"));
                        }
                        CapsuleType::Class | CapsuleType::Interface => {
                            mermaid.push_str(&format!("        {node_id}[\"🎯 {display_name}\"]\n"));
                            mermaid.push_str(&format!("        {node_id}:::classClass\n"));
                        }
                        _ => {
                            mermaid.push_str(&format!("        {node_id}[\"⚪ {display_name}\"]\n"));
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
                mermaid.push_str(&format!("    {from_id} {arrow_style}|{label}| {to_id}\n"));
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

    pub fn export_to_svg(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut svg = String::new();
        
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 600\" width=\"800\" height=\"600\">\n");
        svg.push_str(&format!("  <text x=\"400\" y=\"50\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"16\">Архитектурная диаграмма</text>\n"));
        svg.push_str(&format!("  <text x=\"400\" y=\"80\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"12\">Компонентов: {}, Связей: {}</text>\n", 
            graph.capsules.len(), graph.relations.len()));
        
        let mut y = 120;
        for capsule in graph.capsules.values() {
            svg.push_str(&format!("  <rect x=\"100\" y=\"{}\" width=\"600\" height=\"30\" fill=\"lightblue\" stroke=\"black\"/>\n", y));
            svg.push_str(&format!("  <text x=\"110\" y=\"{}\" font-family=\"Arial\" font-size=\"12\">{}</text>\n", y + 20, capsule.name));
            y += 40;
        }
        
        svg.push_str("</svg>\n");
        Ok(svg)
    }
    
    /// Экспорт в интерактивный HTML
    pub fn export_to_interactive_html(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<head>\n");
        html.push_str("  <title>Архитектурная диаграмма</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("    .component { margin: 10px; padding: 10px; border: 1px solid #ccc; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("  <h1>Архитектурная диаграмма</h1>\n");
        html.push_str(&format!("  <p>Компонентов: {}, Связей: {}</p>\n", 
            graph.capsules.len(), graph.relations.len()));
        
        for capsule in graph.capsules.values() {
            html.push_str(&format!("  <div class=\"component\">\n"));
            html.push_str(&format!("    <h3>{}</h3>\n", capsule.name));
            html.push_str(&format!("    <p>Сложность: {}</p>\n", capsule.complexity));
            html.push_str(&format!("    <p>Файл: {}</p>\n", capsule.file_path.display()));
            html.push_str("  </div>\n");
        }
        
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        Ok(html)
    }
    
    /// Экспорт в формат Chain of Thought
    pub fn export_to_chain_of_thought(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut cot = String::new();
        
        cot.push_str("# Chain of Thought - Анализ архитектуры\n\n");
        cot.push_str(&format!("## Общая информация\n"));
        cot.push_str(&format!("- Компонентов: {}\n", graph.capsules.len()));
        cot.push_str(&format!("- Связей: {}\n", graph.relations.len()));
        cot.push_str(&format!("- Средняя сложность: {:.2}\n\n", graph.metrics.complexity_average));
        
        cot.push_str("## Компоненты\n");
        for capsule in graph.capsules.values() {
            cot.push_str(&format!("- {} ({}): сложность {}\n", 
                capsule.name, format!("{:?}", capsule.capsule_type), capsule.complexity));
        }
        
        Ok(cot)
    }

    /// Экспорт в формат LLM Prompt
    pub fn export_to_llm_prompt(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut prompt = String::new();
        
        prompt.push_str("Analyze the following software architecture:\n\n");
        prompt.push_str(&format!("Components: {}\n", graph.capsules.len()));
        prompt.push_str(&format!("Relations: {}\n", graph.relations.len()));
        prompt.push_str(&format!("Average complexity: {:.2}\n\n", graph.metrics.complexity_average));
        
        prompt.push_str("Component details:\n");
        for capsule in graph.capsules.values() {
            prompt.push_str(&format!("- {}: type={:?}, complexity={}\n", 
                capsule.name, capsule.capsule_type, capsule.complexity));
        }
        
        Ok(prompt)
    }

    /// Супер-компактный сводный экспорт под ИИ: топ метрик, без длинных блоков
    pub fn export_to_ai_compact(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut compact = String::new();
        compact.push_str("# AI Compact Analysis\n\n");
        compact.push_str(&format!("## Summary\n- Components: {}\n- Relations: {}\n- Complexity(avg): {:.2}\n\n", graph.metrics.total_capsules, graph.metrics.total_relations, graph.metrics.complexity_average));
        
        // Краткие проблемы (эвристики)
        compact.push_str("## Problems (Heuristic)\n");
        if graph.metrics.coupling_index > 0.7 { compact.push_str("- High coupling\n"); }
        if graph.metrics.cohesion_index < 0.3 { compact.push_str("- Low cohesion\n"); }
        if graph.metrics.cyclomatic_complexity > (graph.metrics.total_relations as u32).saturating_add(10) { compact.push_str("- High graph cyclomatic complexity\n"); }
        // Подсчёт предупреждений
        let total_warnings: usize = graph.capsules.values().map(|c| c.warnings.len()).sum();
        if total_warnings > 0 { compact.push_str(&format!("- Warnings: {}\n", total_warnings)); }
        if compact.ends_with("Heuristic)\n") { compact.push_str("- None\n"); }
        compact.push_str("\n");
        
        // Топ-капсулы по сложности
        let mut top: Vec<_> = graph.capsules.values().collect();
        top.sort_by_key(|c| std::cmp::Reverse(c.complexity));
        let top = top.into_iter().take(10);
        compact.push_str("## Top Complexity Components\n");
        for capsule in top { compact.push_str(&format!("- {} ({:?}) : {}\n", capsule.name, capsule.capsule_type, capsule.complexity)); }
        
        // Краткие слои
        if !graph.layers.is_empty() {
            compact.push_str("\n## Layers\n");
            let mut layers: Vec<_> = graph.layers.iter().map(|(k,v)| (k.clone(), v.len())).collect();
            layers.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
            for (name, count) in layers.into_iter().take(8) { compact.push_str(&format!("- {}: {}\n", name, count)); }
        }
        Ok(compact)
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
                    warnings: capsule.warnings.iter().map(|w| w.message.clone()).collect(),
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