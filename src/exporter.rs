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
            ExportFormat::SVG => self.export_to_svg(graph)?,
            ExportFormat::ChainOfThought => self.export_to_chain_of_thought(graph)?,
            ExportFormat::LLMPrompt => self.export_to_llm_prompt(graph)?,
            ExportFormat::AICompact => self.export_to_ai_compact(graph)?,
        };
        
        std::fs::write(output_path, &content)?;
        Ok(content)
    }
    
    pub fn export_to_json(&self, graph: &CapsuleGraph) -> Result<String> {
        // Создаем упрощенную структуру для JSON
        let json_graph = JsonGraph::from_capsule_graph(graph);
        let json = serde_json::to_string_pretty(&json_graph)
            .map_err(|e| AnalysisError::Internal(format!("JSON serialization error: {e}")))?;
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
        
        // Вычисляем размеры диаграммы
        let total_capsules = graph.capsules.len();
        let layer_count = graph.layers.len().max(1);
        let width = (layer_count * 300 + 200).min(1600);
        let height = (total_capsules * 50 + 400).min(1200);
        
        // SVG заголовок
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str(&format!("<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">\n", width, height, width, height));
        
        // Стили CSS
        svg.push_str("<defs>\n<style>\n");
        svg.push_str(".title { font-family: Arial, sans-serif; font-size: 24px; font-weight: bold; fill: #2c3e50; }\n");
        svg.push_str(".layer-title { font-family: Arial, sans-serif; font-size: 16px; font-weight: bold; fill: #34495e; }\n");
        svg.push_str(".capsule-name { font-family: Arial, sans-serif; font-size: 12px; fill: #2c3e50; }\n");
        svg.push_str(".complexity-text { font-family: Arial, sans-serif; font-size: 10px; fill: #7f8c8d; }\n");
        svg.push_str(".metric-text { font-family: Arial, sans-serif; font-size: 12px; fill: #2c3e50; }\n");
        svg.push_str(".module-capsule { fill: #e3f2fd; stroke: #1976d2; stroke-width: 2; }\n");
        svg.push_str(".function-capsule { fill: #f3e5f5; stroke: #7b1fa2; stroke-width: 2; }\n");
        svg.push_str(".struct-capsule { fill: #e8f5e8; stroke: #388e3c; stroke-width: 2; }\n");
        svg.push_str(".class-capsule { fill: #fff3e0; stroke: #f57c00; stroke-width: 2; }\n");
        svg.push_str(".interface-capsule { fill: #fce4ec; stroke: #c2185b; stroke-width: 2; }\n");
        svg.push_str(".other-capsule { fill: #f5f5f5; stroke: #616161; stroke-width: 2; }\n");
        svg.push_str(".layer-background { fill: #fafafa; stroke: #e0e0e0; stroke-width: 1; }\n");
        svg.push_str(".capsule:hover { filter: brightness(1.1); cursor: pointer; }\n");
        svg.push_str("</style>\n");
        
        // Маркер стрелки
        svg.push_str("<marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
        svg.push_str("<polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#424242\" />\n");
        svg.push_str("</marker>\n</defs>\n\n");
        
        // Заголовок диаграммы
        svg.push_str(&format!("<text x=\"{}\" y=\"25\" class=\"title\" text-anchor=\"middle\">Архитектурная диаграмма ArchLens</text>\n", width / 2));
        svg.push_str(&format!("<text x=\"{}\" y=\"45\" class=\"metric-text\" text-anchor=\"middle\">Капсул: {} | Связей: {} | Слоёв: {} | Сложность: {:.1}</text>\n", 
            width / 2, graph.metrics.total_capsules, graph.metrics.total_relations, graph.layers.len(), graph.metrics.complexity_average));
        
        // Панель метрик
        svg.push_str(&format!("<rect x=\"20\" y=\"60\" width=\"{}\" height=\"60\" class=\"layer-background\" rx=\"5\"/>\n", width - 40));
        svg.push_str(&format!("<text x=\"30\" y=\"80\" class=\"metric-text\">Метрики качества:</text>\n"));
        svg.push_str(&format!("<text x=\"30\" y=\"95\" class=\"complexity-text\">• Связанность: {:.2} • Сплоченность: {:.2} • Глубина: {}</text>\n", 
            graph.metrics.coupling_index, graph.metrics.cohesion_index, graph.metrics.depth_levels));
        svg.push_str(&format!("<text x=\"30\" y=\"110\" class=\"complexity-text\">• Цикломатическая сложность: {}</text>\n", graph.metrics.cyclomatic_complexity));
        
        // Рисуем слои и капсулы
        let layer_width = (width - 100) / layer_count.max(1);
        let mut capsule_positions = std::collections::HashMap::new();
        
        for (layer_index, (layer_name, capsule_ids)) in graph.layers.iter().enumerate() {
            let x = 50 + layer_index * layer_width;
            let layer_height = capsule_ids.len() * 70 + 100;
            
            // Фон слоя
            svg.push_str(&format!("<rect x=\"{}\" y=\"140\" width=\"{}\" height=\"{}\" class=\"layer-background\" rx=\"5\"/>\n", 
                x, layer_width - 20, layer_height));
            
            // Заголовок слоя
            svg.push_str(&format!("<text x=\"{}\" y=\"165\" class=\"layer-title\" text-anchor=\"middle\">{} ({})</text>\n", 
                x + layer_width / 2 - 10, layer_name, capsule_ids.len()));
            
            // Капсулы в слое
            for (cap_index, capsule_id) in capsule_ids.iter().enumerate() {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    let cap_x = x + 10;
                    let cap_y = 180 + cap_index * 70;
                    let cap_width = layer_width - 40;
                    let cap_height = 60;
                    
                    // Сохраняем позицию для рисования связей
                    capsule_positions.insert(*capsule_id, (cap_x + cap_width / 2, cap_y + cap_height / 2));
                    
                    // Определяем класс стиля по типу
                    let capsule_class = match capsule.capsule_type {
                        CapsuleType::Module => "module-capsule",
                        CapsuleType::Function | CapsuleType::Method => "function-capsule",
                        CapsuleType::Struct | CapsuleType::Enum => "struct-capsule",
                        CapsuleType::Class => "class-capsule",
                        CapsuleType::Interface => "interface-capsule",
                        _ => "other-capsule",
                    };
                    
                    // Иконка по типу
                    let icon = match capsule.capsule_type {
                        CapsuleType::Module => "📦",
                        CapsuleType::Function | CapsuleType::Method => "⚙️",
                        CapsuleType::Struct | CapsuleType::Enum => "🏗️",
                        CapsuleType::Class => "🎯",
                        CapsuleType::Interface => "🔗",
                        _ => "⚪",
                    };
                    
                    // Прямоугольник капсулы
                    svg.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" class=\"{} capsule\" rx=\"3\">\n", 
                        cap_x, cap_y, cap_width, cap_height, capsule_class));
                    
                    // Tooltip с деталями
                    svg.push_str(&format!("<title>{} {} ({}): Сложность {}, Строки {}-{}</title>\n", 
                        icon, capsule.name, format!("{:?}", capsule.capsule_type), capsule.complexity, capsule.line_start, capsule.line_end));
                    svg.push_str("</rect>\n");
                    
                    // Иконка и название
                    svg.push_str(&format!("<text x=\"{}\" y=\"{}\" class=\"capsule-name\">{} {}</text>\n", 
                        cap_x + 5, cap_y + 20, icon, self.truncate_name(&capsule.name, 25)));
                    
                    // Сложность и приоритет
                    svg.push_str(&format!("<text x=\"{}\" y=\"{}\" class=\"complexity-text\">Сложность: {} | Приоритет: {:?}</text>\n", 
                        cap_x + 5, cap_y + 35, capsule.complexity, capsule.priority));
                    
                    // Путь к файлу (сокращенный)
                    let short_path = capsule.file_path.file_name().unwrap_or_default().to_string_lossy();
                    svg.push_str(&format!("<text x=\"{}\" y=\"{}\" class=\"complexity-text\">Файл: {}</text>\n", 
                        cap_x + 5, cap_y + 50, short_path));
                    
                    // Предупреждения
                    if !capsule.warnings.is_empty() {
                        svg.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"#f44336\" stroke=\"#ffffff\" stroke-width=\"1\"/>\n", 
                            cap_x + cap_width - 15, cap_y + 15));
                        svg.push_str(&format!("<text x=\"{}\" y=\"{}\" class=\"complexity-text\" fill=\"white\" text-anchor=\"middle\">!</text>\n", 
                            cap_x + cap_width - 15, cap_y + 18));
                    }
                }
            }
        }
        
        // Рисуем связи между капсулами
        svg.push_str(&format!("\n<!-- Связи ({}) -->\n", graph.relations.len()));
        for relation in &graph.relations {
            if let (Some(from_pos), Some(to_pos)) = (
                capsule_positions.get(&relation.from_id),
                capsule_positions.get(&relation.to_id)
            ) {
                let color = match relation.relation_type {
                    RelationType::Depends => "#2196f3",
                    RelationType::Uses => "#4caf50",
                    RelationType::Implements => "#ff9800",
                    RelationType::Extends => "#9c27b0",
                    _ => "#607d8b",
                };
                
                let line_style = match relation.relation_type {
                    RelationType::Uses => "stroke-dasharray=\"5,5\"",
                    _ => "",
                };
                
                svg.push_str(&format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"2\" {} marker-end=\"url(#arrowhead)\">\n", 
                    from_pos.0, from_pos.1, to_pos.0, to_pos.1, color, line_style));
                
                svg.push_str(&format!("<title>{:?}: {:.2}</title>\n", relation.relation_type, relation.strength));
                svg.push_str("</line>\n");
            }
        }
        
        // Легенда
        let legend_y = height - 120;
        svg.push_str(&format!("<rect x=\"20\" y=\"{}\" width=\"300\" height=\"100\" class=\"layer-background\" rx=\"5\"/>\n", legend_y));
        svg.push_str(&format!("<text x=\"30\" y=\"{}\" class=\"layer-title\">Легенда типов:</text>\n", legend_y + 20));
        
        let legend_items = [
            ("📦 Модуль", "module-capsule"),
            ("⚙️ Функция", "function-capsule"),
            ("🏗️ Структура", "struct-capsule"),
            ("🎯 Класс", "class-capsule"),
        ];
        
        for (i, (label, class)) in legend_items.iter().enumerate() {
            let legend_x = 30 + (i % 2) * 140;
            let legend_item_y = legend_y + 35 + (i / 2) * 20;
            
            svg.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"15\" height=\"15\" class=\"{}\"/>\n", 
                legend_x, legend_item_y - 12, class));
            svg.push_str(&format!("<text x=\"{}\" y=\"{}\" class=\"complexity-text\">{}</text>\n", 
                legend_x + 20, legend_item_y, label));
        }
        
        svg.push_str("</svg>");
        Ok(svg)
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
        cot.push('\n');
        
        cot.push_str("## 🏗️ Архитектурные слои\n");
        for (layer_name, capsule_ids) in &graph.layers {
            cot.push_str(&format!("### Слой: {layer_name}\n"));
            cot.push_str(&format!("Компонентов в слое: {}\n\n", capsule_ids.len()));
            
            cot.push_str("**Ключевые компоненты:**\n");
            for capsule_id in capsule_ids.iter().take(5) {
                if let Some(capsule) = graph.capsules.get(capsule_id) {
                    cot.push_str(&format!("- `{}` ({:?}, сложность: {})\n", 
                                         capsule.name, capsule.capsule_type, capsule.complexity));
                }
            }
            cot.push('\n');
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
        cot.push('\n');
        
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
            prompt.push('\n');
        }
        
        prompt.push_str("## Метрики качества\n");
        prompt.push_str(&format!("- Средняя сложность: {:.2}\n", graph.metrics.complexity_average));
        prompt.push_str(&format!("- Связанность: {:.2}\n", graph.metrics.coupling_index));
        prompt.push_str(&format!("- Сплоченность: {:.2}\n", graph.metrics.cohesion_index));
        prompt.push_str(&format!("- Глубина зависимостей: {}\n", graph.metrics.depth_levels));
        prompt.push('\n');
        
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

    pub fn export_to_ai_compact(&self, graph: &CapsuleGraph) -> Result<String> {
        let mut output = String::new();
        
        // Компактный заголовок с ключевыми метриками
        output.push_str("# 🏗️ АРХИТЕКТУРНЫЙ ПРОФИЛЬ СИСТЕМЫ\n\n");
        
        // Сжатые метрики в одну строку
        output.push_str(&format!("📊 МЕТРИКИ: {}к/{}/{}лв | Сложн:{:.1} | Связ:{:.2} | Спл:{:.2}\n\n",
            graph.metrics.total_capsules,
            graph.metrics.total_relations, 
            graph.metrics.depth_levels,
            graph.metrics.complexity_average,
            graph.metrics.coupling_index,
            graph.metrics.cohesion_index
        ));
        
        // Паттерны и аномалии - новый критический блок
        output.push_str("## 🧩 ПАТТЕРНЫ/АНОМАЛИИ\n\n");
        
        // Поиск циклических зависимостей
        let cycles = self.detect_cycles(graph);
        if !cycles.is_empty() {
            output.push_str("- [CYCLE] ");
            for cycle in cycles.iter().take(3) {
                output.push_str(&format!("{} <-> ", cycle));
            }
            output.push_str("...\n");
        }
        
        // God Objects (высокая сложность)
        let god_objects: Vec<_> = graph.capsules.values()
            .filter(|c| c.complexity > 50 || (c.complexity > 25 && c.dependencies.len() > 10))
            .collect();
        if !god_objects.is_empty() {
            output.push_str("- [GOD-OBJECT] ");
            for obj in god_objects.iter().take(5) {
                output.push_str(&format!("{}({}), ", obj.name, obj.complexity));
            }
            output.push_str("...\n");
        }
        
        // Orphan модули (нет зависимостей)
        let orphans: Vec<_> = graph.capsules.values()
            .filter(|c| c.dependencies.is_empty() && c.dependents.is_empty() && 
                      c.capsule_type != CapsuleType::Export && c.capsule_type != CapsuleType::Import)
            .collect();
        if !orphans.is_empty() {
            output.push_str(&format!("- [ORPHAN] {} модулей: ", orphans.len()));
            for orphan in orphans.iter().take(3) {
                output.push_str(&format!("{}, ", orphan.name));
            }
            output.push_str("...\n");
        }
        
        // Модули без тестов (определяем по отсутствию test_ префиксов в связях)
        let no_tests: Vec<_> = graph.capsules.values()
            .filter(|c| c.capsule_type == CapsuleType::Module && 
                      !graph.relations.iter().any(|r| {
                          if let Some(dep) = graph.capsules.get(&r.to_id) {
                              dep.name.contains("test") && r.from_id == c.id
                          } else { false }
                      }))
            .collect();
        if !no_tests.is_empty() {
            output.push_str("- [NO-TESTS] ");
            for nt in no_tests.iter().take(4) {
                output.push_str(&format!("{}, ", nt.name));
            }
            output.push_str("...\n");
        }
        
        output.push('\n');
        
        // Дифф-анализ (если есть предыдущий анализ)
        if let Some(prev) = &graph.previous_analysis {
            output.push_str("## 📉 ИЗМЕНЕНИЯ (от предыдущего анализа)\n\n");
            
            let capsules_diff = graph.metrics.total_capsules as i32 - prev.total_capsules as i32;
            let relations_diff = graph.metrics.total_relations as i32 - prev.total_relations as i32;
            
            // Подсчитываем текущие аномалии
            let current_cycles = self.detect_cycles(graph).len();
            let current_orphans = graph.capsules.values()
                .filter(|c| c.dependencies.is_empty() && c.dependents.is_empty())
                .count();
            let current_max_complexity = graph.capsules.values()
                .map(|c| c.complexity)
                .max()
                .unwrap_or(0);
            let current_max_module = graph.capsules.values()
                .max_by_key(|c| c.complexity)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            
            let cycles_diff = current_cycles as i32 - prev.cycle_count as i32;
            let orphans_diff = current_orphans as i32 - prev.orphan_count as i32;
            let complexity_diff = current_max_complexity as i32 - prev.max_complexity as i32;
            
            output.push_str(&format!("- Модули: {:+}, Связи: {:+}, Циклы: {:+}, Orphan: {:+}\n",
                capsules_diff, relations_diff, cycles_diff, orphans_diff));
                
            if complexity_diff != 0 {
                output.push_str(&format!("- Сложность max: {:+} ({})\n", complexity_diff, current_max_module));
            }
            
            let coupling_diff = graph.metrics.coupling_index - prev.metrics.coupling_index;
            if coupling_diff.abs() > 0.05 {
                output.push_str(&format!("- Связанность: {:+.2}\n", coupling_diff));
            }
            
            let days_since = (graph.created_at - prev.analyzed_at).num_days();
            output.push_str(&format!("- Период: {} дней\n", days_since));
            
            // Индикаторы тренда
            if cycles_diff > 0 || coupling_diff > 0.1 || complexity_diff > 10 {
                output.push_str("⚠️ **ТРЕНД:** Деградация архитектуры\n");
            } else if cycles_diff < 0 && orphans_diff <= 0 && coupling_diff < 0.0 {
                output.push_str("✅ **ТРЕНД:** Улучшение архитектуры\n");
            }
            
            output.push('\n');
        }
        
        // Архитектурные слои - только топ проблемные
        output.push_str("## 🎯 АРХИТЕКТУРНАЯ КАРТА (TOP-10 КРИТИЧЕСКИХ)\n\n");
        
        let mut all_capsules: Vec<_> = graph.capsules.values().collect();
        all_capsules.sort_by(|a, b| {
            // Сортируем по "проблемности": сложность + связанность + предупреждения
            let score_a = a.complexity + (a.dependencies.len() + a.dependents.len()) as u32 * 2 + a.warnings.len() as u32 * 5;
            let score_b = b.complexity + (b.dependencies.len() + b.dependents.len()) as u32 * 2 + b.warnings.len() as u32 * 5;
            score_b.cmp(&score_a)
        });
        
        for (layer_name, capsule_ids) in &graph.layers {
            let layer_capsules: Vec<_> = capsule_ids.iter()
                .filter_map(|id| graph.capsules.get(id))
                .collect();
            
            // Фильтруем только проблемные капсулы в слое
            let problematic: Vec<_> = layer_capsules.iter()
                .filter(|c| c.complexity > 10 || c.dependencies.len() > 3 || !c.warnings.is_empty())
                .take(10)
                .collect();
            
            if problematic.is_empty() && !layer_name.contains("core") && !layer_name.contains("main") {
                continue; // Пропускаем неинтересные слои
            }
            
            let total_complexity: u32 = layer_capsules.iter().map(|c| c.complexity).sum();
            let warnings_count = layer_capsules.iter().map(|c| c.warnings.len()).sum::<usize>();
            
            output.push_str(&format!("### 📦 {} ", layer_name));
            if warnings_count > 0 {
                output.push_str(&format!("({}/{}⚠ ⚡{})\n", layer_capsules.len(), warnings_count, total_complexity));
            } else {
                output.push_str(&format!("({} ⚡{})\n", layer_capsules.len(), total_complexity));
            }
            
            // Показываем только критические элементы
            for capsule in problematic {
                let symbol = match capsule.capsule_type {
                    CapsuleType::Module => "📦",
                    CapsuleType::Function | CapsuleType::Method => "⚙️",
                    CapsuleType::Struct | CapsuleType::Enum => "🏗️",
                    CapsuleType::Class | CapsuleType::Interface => "🎯",
                    _ => "⚪"
                };
                
                let name = if capsule.name.len() > 20 {
                    format!("{}...", &capsule.name[..17])
                } else {
                    capsule.name.clone()
                };
                
                output.push_str(&format!("  {} {}({})", symbol, name, capsule.complexity));
                if capsule.complexity > 20 { output.push_str("🔥"); }
                if !capsule.warnings.is_empty() { output.push_str("⚠"); }
                if capsule.dependencies.len() > 5 { output.push_str("🕸️"); }
                output.push('\n');
            }
            output.push('\n');
        }
        
        // Критические связи - только проблемные
        output.push_str("## 🔗 ПРОБЛЕМНЫЕ СВЯЗИ\n\n");
        
        // Ищем циклические и перегруженные связи
        let mut problematic_relations: Vec<_> = graph.relations.iter()
            .filter(|r| {
                // Высокая сила связи ИЛИ потенциальный цикл ИЛИ неожиданная зависимость
                r.strength > 0.8 || 
                self.is_unexpected_dependency(graph, r) ||
                self.creates_coupling_issue(graph, r)
            })
            .collect();
        
        problematic_relations.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        
        for relation in problematic_relations.iter().take(10) {
            if let (Some(from_capsule), Some(to_capsule)) = 
                (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                
                let from_name = if from_capsule.name.len() > 15 {
                    format!("{}...", &from_capsule.name[..12])
                } else {
                    from_capsule.name.clone()
                };
                let to_name = if to_capsule.name.len() > 15 {
                    format!("{}...", &to_capsule.name[..12])
                } else {
                    to_capsule.name.clone()
                };
                
                let arrow = match relation.relation_type {
                    RelationType::Depends => "→",
                    RelationType::Uses => "⇒", 
                    RelationType::Implements => "⚡",
                    RelationType::Extends => "↗",
                    RelationType::Calls => "📞",
                    _ => "—"
                };
                
                output.push_str(&format!("{} {} {} ({:.2})", from_name, arrow, to_name, relation.strength));
                
                // Добавляем индикаторы проблем
                if relation.strength > 0.9 { output.push_str(" 🔥"); }
                if self.is_unexpected_dependency(graph, relation) { output.push_str(" ❓"); }
                if self.creates_coupling_issue(graph, relation) { output.push_str(" 🕸️"); }
                
                output.push('\n');
            }
        }
        
        // Рекомендации на основе паттернов
        output.push_str("\n## 💡 КРИТИЧЕСКИЕ РЕКОМЕНДАЦИИ\n\n");
        
        if !cycles.is_empty() {
            output.push_str("🔥 **КРИТИЧНО:** Разорвать циклические зависимости\n");
        }
        
        if !god_objects.is_empty() {
            output.push_str("⚠️ Декомпозировать God Objects на более мелкие модули\n");
        }
        
        if graph.metrics.coupling_index > 0.8 {
            output.push_str("🕸️ Снизить связанность через DI/интерфейсы\n");
        }
        
        if !orphans.is_empty() {
            output.push_str("🏝️ Интегрировать или удалить orphan модули\n");
        }
        
        if !no_tests.is_empty() {
            output.push_str("🧪 Добавить тесты к критическим модулям\n");
        }
        
        output.push_str("🏗️ Применить архитектурные границы (Clean Architecture)\n");
        
        output.push_str("\n---\n");
        output.push_str(&format!("📋 {} | Токенов: ~{} | Focus: критические проблемы\n", 
            graph.created_at.format("%Y-%m-%d %H:%M"),
            output.len() / 4
        ));
        
        Ok(output)
    }
    
    // Вспомогательные методы для анализа проблем
    fn detect_cycles(&self, graph: &CapsuleGraph) -> Vec<String> {
        let mut cycles = Vec::new();
        
        // Простая эвристика поиска циклов через анализ взаимных зависимостей
        for relation in &graph.relations {
            if let (Some(from), Some(to)) = (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
                // Ищем обратную связь
                if graph.relations.iter().any(|r| r.from_id == relation.to_id && r.to_id == relation.from_id) {
                    cycles.push(format!("{} <-> {}", from.name, to.name));
                }
            }
        }
        
        cycles.into_iter().take(5).collect()
    }
    
    fn is_unexpected_dependency(&self, graph: &CapsuleGraph, relation: &CapsuleRelation) -> bool {
        if let (Some(from), Some(to)) = (graph.capsules.get(&relation.from_id), graph.capsules.get(&relation.to_id)) {
            // UI зависит от Core/Domain - нормально
            // Core зависит от UI - подозрительно
            if let (Some(from_layer), Some(to_layer)) = (&from.layer, &to.layer) {
                return (from_layer.contains("core") || from_layer.contains("domain")) && 
                       (to_layer.contains("ui") || to_layer.contains("view"));
            }
        }
        false
    }
    
    fn creates_coupling_issue(&self, graph: &CapsuleGraph, relation: &CapsuleRelation) -> bool {
        // Высокая связанность если модуль имеет много входящих И исходящих связей
        let from_deps = graph.relations.iter().filter(|r| r.from_id == relation.from_id).count();
        let to_deps = graph.relations.iter().filter(|r| r.to_id == relation.to_id).count();
        
        from_deps > 8 || to_deps > 8
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