// Модуль анализа зависимостей

use crate::types::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Анализатор зависимостей
pub struct DependencyAnalyzer {
    import_patterns: HashMap<FileType, Regex>,
    export_patterns: HashMap<FileType, Regex>,
}

/// Результат анализа зависимостей
#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub imports: Vec<Dependency>,
    pub exports: Vec<Export>,
    pub circular_dependencies: Vec<CircularDependency>,
    pub unused_imports: Vec<String>,
    pub dependency_metrics: DependencyMetrics,
}

/// Зависимость между модулями
#[derive(Debug, Clone)]
pub struct Dependency {
    pub source: String,
    pub target: String,
    pub dependency_type: DependencyType,
    pub strength: f32,
}

/// Экспорт модуля
#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub export_type: ExportType,
    pub is_public: bool,
}

/// Циклическая зависимость
#[derive(Debug, Clone)]
pub struct CircularDependency {
    pub cycle_path: Vec<String>,
    pub severity: Priority,
}

/// Метрики зависимостей
#[derive(Debug, Clone)]
pub struct DependencyMetrics {
    pub total_imports: usize,
    pub total_exports: usize,
    pub coupling_factor: f32,
    pub cohesion_factor: f32,
    pub instability: f32,
    pub abstractness: f32,
}

/// Типы зависимостей
#[derive(Debug, Clone)]
pub enum DependencyType {
    Direct,
    Indirect,
    Circular,
    Optional,
    Dev,
}

/// Типы экспорта
#[derive(Debug, Clone)]
pub enum ExportType {
    Function,
    Class,
    Module,
    Constant,
    Type,
    Variable,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            import_patterns: Self::create_import_patterns(),
            export_patterns: Self::create_export_patterns(),
        }
    }
    
    pub fn analyze_dependencies(&self, content: &str, file_type: &FileType, file_path: &Path) -> Result<DependencyAnalysis> {
        let imports = self.extract_imports(content, file_type)?;
        let exports = self.extract_exports(content, file_type)?;
        let unused_imports = self.find_unused_imports(content, &imports)?;
        let dependency_metrics = self.calculate_dependency_metrics(&imports, &exports);
        
        Ok(DependencyAnalysis {
            imports,
            exports,
            circular_dependencies: Vec::new(), // Потребует анализа всего графа
            unused_imports,
            dependency_metrics,
        })
    }
    
    fn extract_imports(&self, content: &str, file_type: &FileType) -> Result<Vec<Dependency>> {
        let mut imports = Vec::new();
        
        if let Some(pattern) = self.import_patterns.get(file_type) {
            for cap in pattern.captures_iter(content) {
                let import_path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let import_name = cap.get(2).map(|m| m.as_str()).unwrap_or(import_path);
                
                imports.push(Dependency {
                    source: "current_file".to_string(),
                    target: import_name.to_string(),
                    dependency_type: DependencyType::Direct,
                    strength: 1.0,
                });
            }
        }
        
        Ok(imports)
    }
    
    fn extract_exports(&self, content: &str, file_type: &FileType) -> Result<Vec<Export>> {
        let mut exports = Vec::new();
        
        if let Some(pattern) = self.export_patterns.get(file_type) {
            for cap in pattern.captures_iter(content) {
                let export_type_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let export_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                
                let export_type = match export_type_str {
                    "fn" | "function" => ExportType::Function,
                    "struct" | "class" => ExportType::Class,
                    "mod" | "module" => ExportType::Module,
                    "const" | "static" => ExportType::Constant,
                    "type" | "trait" => ExportType::Type,
                    _ => ExportType::Variable,
                };
                
                exports.push(Export {
                    name: export_name.to_string(),
                    export_type,
                    is_public: true,
                });
            }
        }
        
        Ok(exports)
    }
    
    fn find_unused_imports(&self, content: &str, imports: &[Dependency]) -> Result<Vec<String>> {
        let mut unused = Vec::new();
        
        for import in imports {
            let import_name = import.target.split("::").last().unwrap_or(&import.target);
            
            // Проверяем, используется ли импорт в коде
            let usage_patterns = vec![
                format!(r"\b{}\s*\(", import_name),
                format!(r"\b{}::", import_name),
                format!(r"\b{}\s*\{{", import_name),
                format!(r"\b{}\s*;", import_name),
            ];
            
            let mut used = false;
            for pattern_str in usage_patterns {
                if let Ok(pattern) = Regex::new(&pattern_str) {
                    if pattern.is_match(content) {
                        used = true;
                        break;
                    }
                }
            }
            
            if !used {
                unused.push(import.target.clone());
            }
        }
        
        Ok(unused)
    }
    
    fn calculate_dependency_metrics(&self, imports: &[Dependency], exports: &[Export]) -> DependencyMetrics {
        let total_imports = imports.len();
        let total_exports = exports.len();
        
        // Фактор связности (coupling)
        let coupling_factor = if total_imports > 0 {
            total_imports as f32 / (total_imports + total_exports) as f32
        } else {
            0.0
        };
        
        // Фактор сплоченности (cohesion) - упрощенная версия
        let cohesion_factor = if total_exports > 0 {
            1.0 - (total_imports as f32 / (total_imports + total_exports) as f32)
        } else {
            0.0
        };
        
        // Нестабильность (instability)
        let instability = if total_imports + total_exports > 0 {
            total_imports as f32 / (total_imports + total_exports) as f32
        } else {
            0.0
        };
        
        // Абстрактность (abstractness) - упрощенная версия
        let abstract_exports = exports.iter()
            .filter(|e| matches!(e.export_type, ExportType::Type))
            .count();
        
        let abstractness = if total_exports > 0 {
            abstract_exports as f32 / total_exports as f32
        } else {
            0.0
        };
        
        DependencyMetrics {
            total_imports,
            total_exports,
            coupling_factor,
            cohesion_factor,
            instability,
            abstractness,
        }
    }
    
    pub fn detect_circular_dependencies(&self, dependencies: &[Dependency]) -> Vec<CircularDependency> {
        let mut circular_deps = Vec::new();
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        
        // Строим граф зависимостей
        for dep in dependencies {
            graph.entry(dep.source.clone())
                .or_insert_with(HashSet::new)
                .insert(dep.target.clone());
        }
        
        // Ищем циклы с помощью DFS
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        
        for node in graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) = self.find_cycle(&graph, node, &mut visited, &mut recursion_stack) {
                    circular_deps.push(CircularDependency {
                        cycle_path: cycle,
                        severity: Priority::High,
                    });
                }
            }
        }
        
        circular_deps
    }
    
    fn find_cycle(&self, graph: &HashMap<String, HashSet<String>>, node: &str, visited: &mut HashSet<String>, recursion_stack: &mut HashSet<String>) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.find_cycle(graph, neighbor, visited, recursion_stack) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(neighbor) {
                    // Найден цикл
                    return Some(vec![node.to_string(), neighbor.to_string()]);
                }
            }
        }
        
        recursion_stack.remove(node);
        None
    }
    
    fn create_import_patterns() -> HashMap<FileType, Regex> {
        let mut patterns = HashMap::new();
        
        // Rust
        patterns.insert(
            FileType::Rust,
            Regex::new(r"use\s+(?:([^:]+)::)?([^;]+);").unwrap()
        );
        
        // JavaScript/TypeScript
        let js_import = Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]|import\s+(['"]([^'"]+)['"])"#).unwrap();
        patterns.insert(FileType::JavaScript, js_import.clone());
        patterns.insert(FileType::TypeScript, js_import);
        
        // Python
        patterns.insert(
            FileType::Python,
            Regex::new(r"(?:from\s+(\S+)\s+)?import\s+([^#\n]+)").unwrap()
        );
        
        patterns
    }
    
    fn create_export_patterns() -> HashMap<FileType, Regex> {
        let mut patterns = HashMap::new();
        
        // Rust
        patterns.insert(
            FileType::Rust,
            Regex::new(r"pub\s+(fn|struct|enum|mod|trait|const|static|type)\s+(\w+)").unwrap()
        );
        
        // JavaScript/TypeScript
        let js_export = Regex::new(r"export\s+(function|class|const|let|var|default|type|interface)\s+(\w+)").unwrap();
        patterns.insert(FileType::JavaScript, js_export.clone());
        patterns.insert(FileType::TypeScript, js_export);
        
        // Python
        patterns.insert(
            FileType::Python,
            Regex::new(r"^(def|class)\s+(\w+)").unwrap()
        );
        
        patterns
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 