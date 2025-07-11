use std::path::Path;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ModuleDependency {
    pub from_module: String,
    pub to_module: String,
    pub dependency_type: String,
}

pub fn generate_mermaid_diagram(project_path: &str) -> std::result::Result<String, String> {
    if !Path::new(project_path).exists() {
        return Err("Путь не существует".to_string());
    }
    
    let mut output = String::new();
    
    // Заголовок диаграммы
    output.push_str("graph TD\n");
    output.push_str("    %% Архитектурная диаграмма проекта\n");
    output.push_str("    classDef default fill:#f9f9f9,stroke:#333,stroke-width:2px\n");
    output.push_str("    classDef core fill:#ff6b6b,stroke:#d63031,stroke-width:2px\n");
    output.push_str("    classDef service fill:#4ecdc4,stroke:#00b894,stroke-width:2px\n");
    output.push_str("    classDef utility fill:#ffe66d,stroke:#fdcb6e,stroke-width:2px\n");
    output.push_str("    classDef config fill:#a29bfe,stroke:#6c5ce7,stroke-width:2px\n");
    output.push_str("\n");
    
    // Анализ зависимостей между модулями
    let dependencies = analyze_module_dependencies(project_path)?;
    
    // Генерация узлов
    let mut nodes = HashMap::new();
    for dep in &dependencies {
        let from_sanitized = sanitize_node_name(&dep.from_module);
        let to_sanitized = sanitize_node_name(&dep.to_module);
        
        nodes.insert(from_sanitized.clone(), dep.from_module.clone());
        nodes.insert(to_sanitized.clone(), dep.to_module.clone());
    }
    
    // Добавление определений узлов
    for (node_id, node_name) in &nodes {
        let node_type = classify_node_type(node_name);
        output.push_str(&format!("    {}[\"{}\"]\n", node_id, node_name));
        
        match node_type {
            NodeType::Core => output.push_str(&format!("    class {} core\n", node_id)),
            NodeType::Service => output.push_str(&format!("    class {} service\n", node_id)),
            NodeType::Utility => output.push_str(&format!("    class {} utility\n", node_id)),
            NodeType::Config => output.push_str(&format!("    class {} config\n", node_id)),
        }
    }
    
    output.push_str("\n");
    
    // Добавление связей
    for dep in &dependencies {
        let from_sanitized = sanitize_node_name(&dep.from_module);
        let to_sanitized = sanitize_node_name(&dep.to_module);
        
        let arrow_type = match dep.dependency_type.as_str() {
            "import" => "-->",
            "use" => "-.->",
            "mod" => "==>",
            _ => "-->",
        };
        
        output.push_str(&format!("    {} {} {}\n", from_sanitized, arrow_type, to_sanitized));
    }
    
    // Добавление подграфов для группировки
    add_subgraphs(&mut output, &nodes);
    
    Ok(output)
}

pub fn analyze_module_dependencies(project_path: &str) -> std::result::Result<Vec<ModuleDependency>, String> {
    let mut dependencies = Vec::new();
    
    scan_for_dependencies(Path::new(project_path), &mut dependencies, "root")?;
    
    // Удаление дубликатов
    dependencies.sort_by(|a, b| {
        a.from_module.cmp(&b.from_module)
            .then_with(|| a.to_module.cmp(&b.to_module))
    });
    dependencies.dedup_by(|a, b| {
        a.from_module == b.from_module && a.to_module == b.to_module
    });
    
    Ok(dependencies)
}

fn scan_for_dependencies(dir: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(dir_name) {
                    let module_name = format!("{}/{}", current_module, dir_name);
                    scan_for_dependencies(&path, dependencies, &module_name)?;
                }
            }
        } else {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => analyze_rust_file(&path, dependencies, current_module)?,
                    "js" | "ts" | "jsx" | "tsx" => analyze_js_file(&path, dependencies, current_module)?,
                    "py" => analyze_python_file(&path, dependencies, current_module)?,
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

fn analyze_rust_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Ошибка чтения файла: {}", e))?;
    
    let file_name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let module_name = format!("{}/{}", current_module, file_name);
    
    for line in content.lines() {
        let line = line.trim();
        
        if let Some(import) = extract_rust_import(line) {
            dependencies.push(ModuleDependency {
                from_module: module_name.clone(),
                to_module: import,
                dependency_type: "use".to_string(),
            });
        }
        
        if line.starts_with("mod ") {
            if let Some(mod_name) = line.split_whitespace().nth(1) {
                let mod_name = mod_name.trim_end_matches(';');
                dependencies.push(ModuleDependency {
                    from_module: module_name.clone(),
                    to_module: format!("{}/{}", current_module, mod_name),
                    dependency_type: "mod".to_string(),
                });
            }
        }
    }
    
    Ok(())
}

fn analyze_js_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Ошибка чтения файла: {}", e))?;
    
    let file_name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let module_name = format!("{}/{}", current_module, file_name);
    
    for line in content.lines() {
        let line = line.trim();
        
        if let Some(import) = extract_js_import(line) {
            dependencies.push(ModuleDependency {
                from_module: module_name.clone(),
                to_module: import,
                dependency_type: "import".to_string(),
            });
        }
    }
    
    Ok(())
}

fn analyze_python_file(file_path: &Path, dependencies: &mut Vec<ModuleDependency>, current_module: &str) -> std::result::Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Ошибка чтения файла: {}", e))?;
    
    let file_name = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let module_name = format!("{}/{}", current_module, file_name);
    
    for line in content.lines() {
        let line = line.trim();
        
        if let Some(import) = extract_python_import(line) {
            dependencies.push(ModuleDependency {
                from_module: module_name.clone(),
                to_module: import,
                dependency_type: "import".to_string(),
            });
        }
    }
    
    Ok(())
}

fn extract_rust_import(line: &str) -> Option<String> {
    if line.starts_with("use ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let import_path = parts[1].trim_end_matches(';');
            // Упрощаем путь импорта
            let simplified = import_path.split("::").next().unwrap_or(import_path);
            return Some(simplified.to_string());
        }
    }
    None
}

fn extract_js_import(line: &str) -> Option<String> {
    if line.starts_with("import ") {
        if let Some(from_pos) = line.find("from ") {
            let import_path = &line[from_pos + 5..];
            let import_path = import_path.trim().trim_matches('"').trim_matches('\'');
            return Some(import_path.to_string());
        }
    } else if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
        if line.contains("require(") {
            if let Some(start) = line.find("require(") {
                if let Some(end) = line[start..].find(")") {
                    let import_path = &line[start + 8..start + end];
                    let import_path = import_path.trim().trim_matches('"').trim_matches('\'');
                    return Some(import_path.to_string());
                }
            }
        }
    }
    None
}

fn extract_python_import(line: &str) -> Option<String> {
    if line.starts_with("import ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].split('.').next().unwrap_or(parts[1]).to_string());
        }
    } else if line.starts_with("from ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].split('.').next().unwrap_or(parts[1]).to_string());
        }
    }
    None
}

fn sanitize_node_name(name: &str) -> String {
    name.replace("/", "_")
        .replace("-", "_")
        .replace(".", "_")
        .replace(" ", "_")
        .replace(":", "_")
        .replace("@", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

#[derive(Debug)]
enum NodeType {
    Core,
    Service,
    Utility,
    Config,
}

fn classify_node_type(node_name: &str) -> NodeType {
    let name_lower = node_name.to_lowercase();
    
    if name_lower.contains("main") || name_lower.contains("lib") || name_lower.contains("core") {
        NodeType::Core
    } else if name_lower.contains("service") || name_lower.contains("api") || name_lower.contains("handler") {
        NodeType::Service
    } else if name_lower.contains("util") || name_lower.contains("helper") || name_lower.contains("common") {
        NodeType::Utility
    } else if name_lower.contains("config") || name_lower.contains("setting") || name_lower.contains("env") {
        NodeType::Config
    } else {
        NodeType::Core
    }
}

fn add_subgraphs(output: &mut String, nodes: &HashMap<String, String>) {
    output.push_str("\n    %% Группировка модулей\n");
    
    let mut core_modules = Vec::new();
    let mut service_modules = Vec::new();
    let mut utility_modules = Vec::new();
    let mut config_modules = Vec::new();
    
    for (node_id, node_name) in nodes {
        match classify_node_type(node_name) {
            NodeType::Core => core_modules.push(node_id),
            NodeType::Service => service_modules.push(node_id),
            NodeType::Utility => utility_modules.push(node_id),
            NodeType::Config => config_modules.push(node_id),
        }
    }
    
    if !core_modules.is_empty() {
        output.push_str("    subgraph Core[\"🔧 Core Modules\"]\n");
        for module in core_modules {
            output.push_str(&format!("        {}\n", module));
        }
        output.push_str("    end\n");
    }
    
    if !service_modules.is_empty() {
        output.push_str("    subgraph Services[\"🚀 Services\"]\n");
        for module in service_modules {
            output.push_str(&format!("        {}\n", module));
        }
        output.push_str("    end\n");
    }
    
    if !utility_modules.is_empty() {
        output.push_str("    subgraph Utils[\"🛠️ Utilities\"]\n");
        for module in utility_modules {
            output.push_str(&format!("        {}\n", module));
        }
        output.push_str("    end\n");
    }
    
    if !config_modules.is_empty() {
        output.push_str("    subgraph Config[\"⚙️ Configuration\"]\n");
        for module in config_modules {
            output.push_str(&format!("        {}\n", module));
        }
        output.push_str("    end\n");
    }
}

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(dir_name, "node_modules" | "target" | ".git" | ".svn" | "dist" | "build" | 
                      ".next" | ".nuxt" | "coverage" | "__pycache__" | "backup")
} 