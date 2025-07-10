use std::path::Path;
use std::fs;
use regex::Regex;
use ignore::WalkBuilder;
use crate::core::{FileMetadata, FileType, CapsuleStatus, AnalysisError, Result};

/// Сканер файлов проекта
pub struct FileScanner {
    include_patterns: Vec<Regex>,
    exclude_patterns: Vec<Regex>,
    max_depth: Option<usize>,
}

impl FileScanner {
    pub fn new(
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        max_depth: Option<usize>,
    ) -> Result<Self> {
        let include_patterns = include_patterns
            .into_iter()
            .map(|p| glob_to_regex(&p))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AnalysisError::Parse(e.to_string()))?;

        let exclude_patterns = exclude_patterns
            .into_iter()
            .map(|p| glob_to_regex(&p))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AnalysisError::Parse(e.to_string()))?;

        Ok(Self {
            include_patterns,
            exclude_patterns,
            max_depth,
        })
    }

    /// Сканирует проект и возвращает метаданные всех подходящих файлов
    pub fn scan_project(&self, project_path: &Path) -> Result<Vec<FileMetadata>> {
        if !project_path.exists() {
            return Err(AnalysisError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Путь не существует: {}", project_path.display()),
            )));
        }

        let mut files = Vec::new();
        let mut walker = WalkBuilder::new(project_path);
        
        if let Some(depth) = self.max_depth {
            walker.max_depth(Some(depth));
        }

        for result in walker.build() {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        if let Ok(metadata) = self.extract_file_metadata(entry.path()) {
                            if self.should_include_file(&metadata) {
                                files.push(metadata);
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("Ошибка при сканировании файла: {}", err);
                }
            }
        }

        Ok(files)
    }

    /// Извлекает метаданные из файла
    fn extract_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = fs::metadata(path)?;
        let file_type = self.detect_file_type(path);
        
        let content = fs::read_to_string(path).unwrap_or_default();
        let lines_count = content.lines().count();

        let last_modified = metadata
            .modified()?
            .into();

        let (imports, exports) = self.extract_imports_exports(&content, &file_type);

        Ok(FileMetadata {
            path: path.to_path_buf(),
            file_type,
            size: metadata.len(),
            lines_count,
            last_modified,
            layer: self.detect_layer(path),
            slogan: self.extract_slogan(&content),
            status: self.detect_status(&content),
            dependencies: Vec::new(), // Будет заполнено позже
            exports,
            imports,
        })
    }

    /// Определяет тип файла по расширению
    fn detect_file_type(&self, path: &Path) -> FileType {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => FileType::Rust,
            Some("js") => FileType::JavaScript,
            Some("ts") => FileType::TypeScript,
            Some("tsx") => FileType::TypeScript,
            Some("jsx") => FileType::JavaScript,
            Some("py") => FileType::Python,
            Some("java") => FileType::Java,
            Some("go") => FileType::Go,
            Some("cpp") | Some("cxx") | Some("cc") => FileType::Cpp,
            Some("c") => FileType::C,
            Some(ext) => FileType::Other(ext.to_string()),
            None => FileType::Other("unknown".to_string()),
        }
    }

    /// Определяет архитектурный слой по пути файла
    fn detect_layer(&self, path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("domain") || path_str.contains("core") {
            Some("domain".to_string())
        } else if path_str.contains("infrastructure") || path_str.contains("infra") {
            Some("infrastructure".to_string())
        } else if path_str.contains("application") || path_str.contains("app") {
            Some("application".to_string())
        } else if path_str.contains("ui") || path_str.contains("view") || path_str.contains("component") {
            Some("presentation".to_string())
        } else if path_str.contains("test") {
            Some("test".to_string())
        } else {
            None
        }
    }

    /// Извлекает слоган из комментариев файла
    fn extract_slogan(&self, content: &str) -> Option<String> {
        let lines = content.lines().take(10);
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("//!") || trimmed.starts_with("///") {
                let comment = trimmed.trim_start_matches("//!").trim_start_matches("///").trim();
                if !comment.is_empty() && comment.len() < 100 {
                    return Some(comment.to_string());
                }
            }
            if trimmed.starts_with("/*") && trimmed.contains("*/") {
                let comment = trimmed
                    .trim_start_matches("/*")
                    .trim_end_matches("*/")
                    .trim();
                if !comment.is_empty() && comment.len() < 100 {
                    return Some(comment.to_string());
                }
            }
        }
        
        None
    }

    /// Определяет статус файла
    fn detect_status(&self, content: &str) -> CapsuleStatus {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("@deprecated") || content_lower.contains("deprecated") {
            CapsuleStatus::Deprecated
        } else if content_lower.contains("@experimental") || content_lower.contains("experimental") {
            CapsuleStatus::Experimental
        } else if content_lower.contains("@internal") || content_lower.contains("internal") {
            CapsuleStatus::Internal
        } else {
            CapsuleStatus::Public
        }
    }

    /// Извлекает импорты и экспорты
    fn extract_imports_exports(&self, content: &str, file_type: &FileType) -> (Vec<String>, Vec<String>) {
        match file_type {
            FileType::Rust => self.extract_rust_imports_exports(content),
            FileType::TypeScript | FileType::JavaScript => self.extract_ts_js_imports_exports(content),
            FileType::Python => self.extract_python_imports_exports(content),
            _ => (Vec::new(), Vec::new()),
        }
    }

    fn extract_rust_imports_exports(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                if let Some(import) = trimmed.strip_prefix("use ") {
                    if let Some(semicolon_pos) = import.find(';') {
                        imports.push(import[..semicolon_pos].trim().to_string());
                    }
                }
            }
            if trimmed.starts_with("pub ") {
                if let Some(export) = extract_rust_export_name(trimmed) {
                    exports.push(export);
                }
            }
        }

        (imports, exports)
    }

    fn extract_ts_js_imports_exports(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(from_pos) = trimmed.find(" from ") {
                    let import_part = &trimmed[7..from_pos]; // убираем "import "
                    imports.push(import_part.trim().to_string());
                }
            }
            if trimmed.starts_with("export ") {
                if let Some(export) = extract_ts_export_name(trimmed) {
                    exports.push(export);
                }
            }
        }

        (imports, exports)
    }

    fn extract_python_imports_exports(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let exports = Vec::new(); // Python экспорты сложнее определить статически

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(import) = trimmed.strip_prefix("import ") {
                    imports.push(import.trim().to_string());
                }
            }
            if trimmed.starts_with("from ") && trimmed.contains(" import ") {
                imports.push(trimmed.to_string());
            }
        }

        (imports, exports)
    }

    /// Проверяет, должен ли файл быть включен в анализ
    fn should_include_file(&self, metadata: &FileMetadata) -> bool {
        let path_str = metadata.path.to_string_lossy();

        // Проверяем exclude patterns
        for pattern in &self.exclude_patterns {
            if pattern.is_match(&path_str) {
                return false;
            }
        }

        // Упрощенная проверка: включаем файлы с нужными расширениями
        let file_extension = metadata.path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let supported_extensions = ["rs", "js", "ts", "tsx", "jsx", "py"];
        let extension_match = supported_extensions.contains(&file_extension);

        // Если есть include patterns, проверяем их
        if !self.include_patterns.is_empty() {
            let pattern_match = self.include_patterns.iter().any(|pattern| pattern.is_match(&path_str));
            extension_match && pattern_match
        } else {
            extension_match
        }
    }
}

/// Конвертирует glob паттерн в regex
fn glob_to_regex(pattern: &str) -> std::result::Result<Regex, regex::Error> {
    let mut regex_pattern = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        match chars[i] {
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                // ** означает любое количество директорий
                if i + 2 < chars.len() && chars[i + 2] == '/' {
                    regex_pattern.push_str("(?:.*[\\\\/])?");
                    i += 3;
                } else {
                    regex_pattern.push_str(".*");
                    i += 2;
                }
            }
            '*' => {
                // * означает любые символы кроме разделителя пути
                regex_pattern.push_str("[^\\\\/]*");
                i += 1;
            }
            '?' => {
                regex_pattern.push_str("[^\\\\/]");
                i += 1;
            }
            '.' => {
                regex_pattern.push_str("\\.");
                i += 1;
            }
            '[' => {
                regex_pattern.push('[');
                i += 1;
            }
            ']' => {
                regex_pattern.push(']');
                i += 1;
            }
            '\\' => {
                regex_pattern.push_str("\\\\");
                i += 1;
            }
            '/' => {
                regex_pattern.push_str("[\\\\/]");
                i += 1;
            }
            c => {
                if c.is_ascii_alphanumeric() || "_-".contains(c) {
                    regex_pattern.push(c);
                } else {
                    regex_pattern.push_str(&regex::escape(&c.to_string()));
                }
                i += 1;
            }
        }
    }
    
    // Делаем паттерн более гибким - он может совпадать в любом месте пути
    let final_pattern = if pattern.starts_with("**/") {
        format!(".*{}", regex_pattern)
    } else {
        regex_pattern
    };
    
    Regex::new(&final_pattern)
}

/// Извлекает имя экспорта из Rust строки
fn extract_rust_export_name(line: &str) -> Option<String> {
    if line.contains("pub fn ") {
        extract_function_name(line, "pub fn ")
    } else if line.contains("pub struct ") {
        extract_name_after(line, "pub struct ")
    } else if line.contains("pub enum ") {
        extract_name_after(line, "pub enum ")
    } else if line.contains("pub mod ") {
        extract_name_after(line, "pub mod ")
    } else {
        None
    }
}

/// Извлекает имя экспорта из TypeScript/JavaScript строки
fn extract_ts_export_name(line: &str) -> Option<String> {
    if line.contains("export function ") {
        extract_function_name(line, "export function ")
    } else if line.contains("export class ") {
        extract_name_after(line, "export class ")
    } else if line.contains("export interface ") {
        extract_name_after(line, "export interface ")
    } else if line.contains("export const ") {
        extract_name_after(line, "export const ")
    } else {
        None
    }
}

fn extract_function_name(line: &str, prefix: &str) -> Option<String> {
    if let Some(start) = line.find(prefix) {
        let after_prefix = &line[start + prefix.len()..];
        if let Some(paren_pos) = after_prefix.find('(') {
            return Some(after_prefix[..paren_pos].trim().to_string());
        }
    }
    None
}

fn extract_name_after(line: &str, prefix: &str) -> Option<String> {
    if let Some(start) = line.find(prefix) {
        let after_prefix = &line[start + prefix.len()..];
        let name = after_prefix
            .split_whitespace()
            .next()?
            .trim_matches('{')
            .trim_matches('<');
        Some(name.to_string())
    } else {
        None
    }
} 