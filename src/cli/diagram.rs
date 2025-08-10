use std::collections::HashMap;
use std::fs;
/// Diagram generation module - creates various architectural diagrams
use std::path::Path;

#[derive(Debug)]
pub struct ModuleDependency {
    pub from_module: String,
    pub to_module: String,
    pub dependency_type: String,
}

#[derive(Debug)]
enum NodeType {
    Core,
    Service,
    Utility,
    Config,
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Generates Mermaid diagram for project architecture
pub fn generate_mermaid_diagram(project_path: &str) -> std::result::Result<String, String> {
    if !Path::new(project_path).exists() {
        return Err("Path does not exist".to_string());
    }

    let mut output = String::new();

    // Diagram header
    add_diagram_header(&mut output);

    // Analyze module dependencies
    let dependencies = analyze_module_dependencies(project_path)?;

    // Generate nodes
    let nodes = collect_nodes(&dependencies);
    add_node_definitions(&mut output, &nodes);

    // Add connections
    add_connections(&mut output, &dependencies);

    // Add subgraphs for grouping
    add_subgraphs(&mut output, &nodes);

    Ok(output)
}

/// Analyzes module dependencies in the project
pub fn analyze_module_dependencies(
    project_path: &str,
) -> std::result::Result<Vec<ModuleDependency>, String> {
    let mut dependencies = Vec::new();

    scan_for_dependencies(Path::new(project_path), &mut dependencies, "root")?;

    // Remove duplicates
    dependencies.sort_by(|a, b| {
        a.from_module
            .cmp(&b.from_module)
            .then_with(|| a.to_module.cmp(&b.to_module))
    });
    dependencies.dedup_by(|a, b| a.from_module == b.from_module && a.to_module == b.to_module);

    Ok(dependencies)
}

// ============================================================================
// DIAGRAM GENERATION
// ============================================================================

fn add_diagram_header(output: &mut String) {
    output.push_str("graph TD\n");
    output.push_str("    %% Project architecture diagram\n");
    output.push_str("    classDef default fill:#f9f9f9,stroke:#333,stroke-width:2px\n");
    output.push_str("    classDef core fill:#ff6b6b,stroke:#d63031,stroke-width:2px\n");
    output.push_str("    classDef service fill:#4ecdc4,stroke:#00b894,stroke-width:2px\n");
    output.push_str("    classDef utility fill:#ffe66d,stroke:#fdcb6e,stroke-width:2px\n");
    output.push_str("    classDef config fill:#a29bfe,stroke:#6c5ce7,stroke-width:2px\n");
    output.push_str("\n");
}

fn collect_nodes(dependencies: &[ModuleDependency]) -> HashMap<String, String> {
    let mut nodes = HashMap::new();
    for dep in dependencies {
        let from_sanitized = sanitize_node_name(&dep.from_module);
        let to_sanitized = sanitize_node_name(&dep.to_module);

        nodes.insert(from_sanitized.clone(), dep.from_module.clone());
        nodes.insert(to_sanitized.clone(), dep.to_module.clone());
    }
    nodes
}

fn add_node_definitions(output: &mut String, nodes: &HashMap<String, String>) {
    for (node_id, node_name) in nodes {
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
}

fn add_connections(output: &mut String, dependencies: &[ModuleDependency]) {
    for dep in dependencies {
        let from_sanitized = sanitize_node_name(&dep.from_module);
        let to_sanitized = sanitize_node_name(&dep.to_module);

        let arrow_type = match dep.dependency_type.as_str() {
            "import" => "-->",
            "use" => "-.->",
            "mod" => "==>",
            _ => "-->",
        };

        output.push_str(&format!(
            "    {} {} {}\n",
            from_sanitized, arrow_type, to_sanitized
        ));
    }
}

fn add_subgraphs(output: &mut String, nodes: &HashMap<String, String>) {
    let mut core_nodes = Vec::new();
    let mut service_nodes = Vec::new();
    let mut utility_nodes = Vec::new();
    let mut config_nodes = Vec::new();

    for (node_id, node_name) in nodes {
        match classify_node_type(node_name) {
            NodeType::Core => core_nodes.push(node_id),
            NodeType::Service => service_nodes.push(node_id),
            NodeType::Utility => utility_nodes.push(node_id),
            NodeType::Config => config_nodes.push(node_id),
        }
    }

    output.push_str("\n    %% Module grouping\n");

    if !core_nodes.is_empty() {
        output.push_str("    subgraph Core[\"🔧 Core Modules\"]\n");
        for node_id in core_nodes {
            output.push_str(&format!("        {}\n", node_id));
        }
        output.push_str("    end\n");
    }

    if !service_nodes.is_empty() {
        output.push_str("    subgraph Services[\"🚀 Services\"]\n");
        for node_id in service_nodes {
            output.push_str(&format!("        {}\n", node_id));
        }
        output.push_str("    end\n");
    }

    if !utility_nodes.is_empty() {
        output.push_str("    subgraph Utils[\"🛠️ Utilities\"]\n");
        for node_id in utility_nodes {
            output.push_str(&format!("        {}\n", node_id));
        }
        output.push_str("    end\n");
    }

    if !config_nodes.is_empty() {
        output.push_str("    subgraph Config[\"⚙️ Configuration\"]\n");
        for node_id in config_nodes {
            output.push_str(&format!("        {}\n", node_id));
        }
        output.push_str("    end\n");
    }
}

// ============================================================================
// DEPENDENCY ANALYSIS
// ============================================================================

fn scan_for_dependencies(
    dir: &Path,
    dependencies: &mut Vec<ModuleDependency>,
    current_module: &str,
) -> std::result::Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to process entry: {}", e))?;
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
                    "js" | "ts" | "jsx" | "tsx" => {
                        analyze_js_file(&path, dependencies, current_module)?
                    }
                    "py" => analyze_python_file(&path, dependencies, current_module)?,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// FILE ANALYZERS
// ============================================================================

fn analyze_rust_file(
    file_path: &Path,
    dependencies: &mut Vec<ModuleDependency>,
    current_module: &str,
) -> std::result::Result<(), String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let file_name = file_path
        .file_stem()
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

fn analyze_js_file(
    file_path: &Path,
    dependencies: &mut Vec<ModuleDependency>,
    current_module: &str,
) -> std::result::Result<(), String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let file_name = file_path
        .file_stem()
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

fn analyze_python_file(
    file_path: &Path,
    dependencies: &mut Vec<ModuleDependency>,
    current_module: &str,
) -> std::result::Result<(), String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let file_name = file_path
        .file_stem()
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

// ============================================================================
// IMPORT EXTRACTORS
// ============================================================================

fn extract_rust_import(line: &str) -> Option<String> {
    if line.starts_with("use ") {
        let import_part = line.strip_prefix("use ")?;
        let import_part = import_part.trim_end_matches(';');
        Some(import_part.to_string())
    } else {
        None
    }
}

fn extract_js_import(line: &str) -> Option<String> {
    if line.starts_with("import ") {
        if let Some(from_pos) = line.find(" from ") {
            let import_part = &line[from_pos + 6..];
            let import_part = import_part.trim_matches('"').trim_matches('\'');
            Some(import_part.to_string())
        } else {
            None
        }
    } else if line.starts_with("const ") && line.contains(" = require(") {
        if let Some(start) = line.find("require(") {
            let require_part = &line[start + 8..];
            if let Some(end) = require_part.find(')') {
                let import_part = &require_part[..end];
                let import_part = import_part.trim_matches('"').trim_matches('\'');
                Some(import_part.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_python_import(line: &str) -> Option<String> {
    if line.starts_with("import ") {
        let import_part = line.strip_prefix("import ")?;
        Some(import_part.to_string())
    } else if line.starts_with("from ") {
        if let Some(import_pos) = line.find(" import ") {
            let from_part = &line[5..import_pos];
            Some(from_part.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

fn sanitize_node_name(name: &str) -> String {
    name.replace("/", "_")
        .replace("-", "_")
        .replace(".", "_")
        .replace(" ", "_")
}

fn classify_node_type(node_name: &str) -> NodeType {
    let name_lower = node_name.to_lowercase();

    if name_lower.contains("service")
        || name_lower.contains("api")
        || name_lower.contains("handler")
    {
        NodeType::Service
    } else if name_lower.contains("util")
        || name_lower.contains("helper")
        || name_lower.contains("tool")
    {
        NodeType::Utility
    } else if name_lower.contains("config")
        || name_lower.contains("setting")
        || name_lower.contains("env")
    {
        NodeType::Config
    } else {
        NodeType::Core
    }
}

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "node_modules" | "target" | ".git" | ".idea" | ".vscode" | "dist" | "build"
    )
}
