use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub file_types: HashMap<String, usize>,
    pub project_path: String,
    pub scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectStructure {
    pub total_files: usize,
    pub file_types: HashMap<String, usize>,
    pub layers: Vec<String>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size: u64,
}

pub fn get_project_stats(project_path: &str) -> std::result::Result<ProjectStats, String> {
    if !Path::new(project_path).exists() {
        return Err("Путь не существует".to_string());
    }

    let mut file_types = HashMap::new();
    let mut total_files = 0;
    let mut total_lines = 0;

    let root_path = Path::new(project_path);
    scan_directory(
        root_path,
        &mut file_types,
        &mut total_files,
        &mut total_lines,
    )
    .map_err(|e| format!("Ошибка сканирования директории: {}", e))?;

    Ok(ProjectStats {
        total_files,
        total_lines,
        file_types,
        project_path: project_path.to_string(),
        scanned_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn scan_directory(
    dir: &Path,
    file_types: &mut HashMap<String, usize>,
    total_files: &mut usize,
    total_lines: &mut usize,
) -> std::result::Result<(), std::io::Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if !should_skip_directory(dir_name) {
                        scan_directory(&path, file_types, total_files, total_lines)?;
                    }
                }
            } else {
                *total_files += 1;

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    *file_types.entry(ext_lower.clone()).or_insert(0) += 1;

                    if is_code_file(&ext_lower) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            *total_lines += content.lines().count();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_project_structure(project_path: &str) -> std::result::Result<ProjectStructure, String> {
    if !Path::new(project_path).exists() {
        return Err("Путь не существует".to_string());
    }

    let mut file_types = HashMap::new();
    let mut total_files = 0;
    let mut files = Vec::new();
    let root_path = Path::new(project_path);

    scan_directory_structure(
        root_path,
        &mut file_types,
        &mut total_files,
        &mut files,
        root_path,
        0,
    )
    .map_err(|e| format!("Ошибка сканирования структуры: {}", e))?;

    let mut layers = Vec::new();
    for file in &files {
        let layer = determine_layer(Path::new(&file.path));
        if !layers.contains(&layer) {
            layers.push(layer);
        }
    }

    Ok(ProjectStructure {
        total_files,
        file_types,
        layers,
        files,
    })
}

fn scan_directory_structure(
    dir: &Path,
    file_types: &mut HashMap<String, usize>,
    total_files: &mut usize,
    files: &mut Vec<FileInfo>,
    root_path: &Path,
    depth: usize,
) -> std::result::Result<(), std::io::Error> {
    if depth > 10 || !dir.is_dir() {
        // Ограничиваем глубину
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(dir_name) {
                    scan_directory_structure(
                        &path,
                        file_types,
                        total_files,
                        files,
                        root_path,
                        depth + 1,
                    )?;
                }
            }
        } else {
            *total_files += 1;

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                *file_types.entry(ext_lower.clone()).or_insert(0) += 1;

                if is_code_file(&ext_lower) {
                    let relative_path = path
                        .strip_prefix(root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);

                    files.push(FileInfo {
                        path: relative_path,
                        name: file_name,
                        extension: ext_lower,
                        size,
                    });
                }
            }
        }
    }
    Ok(())
}

fn determine_layer(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("test") || path_str.contains("spec") {
        "Testing".to_string()
    } else if path_str.contains("cli") || path_str.contains("command") {
        "CLI".to_string()
    } else if path_str.contains("api")
        || path_str.contains("server")
        || path_str.contains("endpoint")
    {
        "API".to_string()
    } else if path_str.contains("service") || path_str.contains("logic") {
        "Service".to_string()
    } else if path_str.contains("model")
        || path_str.contains("entity")
        || path_str.contains("types")
    {
        "Model".to_string()
    } else if path_str.contains("util")
        || path_str.contains("helper")
        || path_str.contains("common")
    {
        "Utils".to_string()
    } else if path_str.contains("config") || path_str.contains("setting") {
        "Config".to_string()
    } else if path_str.contains("ui") || path_str.contains("view") || path_str.contains("component")
    {
        "UI".to_string()
    } else {
        "Core".to_string()
    }
}

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "node_modules"
            | "target"
            | ".git"
            | ".svn"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
            | "coverage"
            | "__pycache__"
            | "backup"
    )
}

fn is_code_file(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "js"
            | "ts"
            | "jsx"
            | "tsx"
            | "py"
            | "java"
            | "cpp"
            | "c"
            | "h"
            | "hpp"
            | "cs"
            | "php"
            | "rb"
            | "go"
            | "swift"
            | "kt"
            | "scala"
            | "clj"
            | "hs"
            | "ml"
            | "fs"
            | "dart"
            | "lua"
            | "r"
            | "m"
            | "mm"
            | "vb"
            | "pas"
            | "pl"
            | "pm"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "psm1"
            | "psd1"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "xml"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "styl"
            | "vue"
            | "svelte"
            | "elm"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
    )
}
