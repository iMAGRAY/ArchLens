use crate::types::{FileMetadata, FileType, CapsuleStatus, AnalysisError, Result};
use std::{fs, path::Path};
use walkdir::WalkDir;

/// Сканер файлов проекта
pub struct FileScanner {
    include_patterns: Vec<regex::Regex>,
    exclude_patterns: Vec<regex::Regex>,
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
        self.scan_files(project_path)
    }

    /// Сканирует файлы в директории (основной метод)
    pub fn scan_files(&self, project_path: &Path) -> Result<Vec<FileMetadata>> {
        let mut files = Vec::new();
        self.scan_directory_recursive(project_path, &mut files, 0)?;
        Ok(files)
    }

    /// Версия scan_files без параметров (для совместимости)
    pub fn scan_files_no_params(&self) -> Result<Vec<FileMetadata>> {
        Err(AnalysisError::GenericError("Не указан путь для сканирования".to_string()))
    }

    /// Рекурсивно сканирует директории
    fn scan_directory_recursive(&self, dir: &Path, files: &mut Vec<FileMetadata>, depth: usize) -> Result<()> {
        if let Some(max_depth) = self.max_depth {
            if depth >= max_depth {
                return Ok(());
            }
        }

        if !dir.is_dir() {
            return Ok(());
        }

        // Безопасное чтение директории с обработкой ошибок доступа
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("⚠️ Предупреждение: Не удалось получить доступ к директории {:?}: {}", dir, e);
                return Ok(()); // Пропускаем недоступные директории
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("⚠️ Предупреждение: Ошибка чтения элемента в {:?}: {}", dir, e);
                    continue; // Пропускаем проблемные элементы
                }
            };
            
            let path = entry.path();

            if path.is_dir() {
                // Рекурсивно сканируем поддиректории, но не прерываем работу при ошибках
                if let Err(e) = self.scan_directory_recursive(&path, files, depth + 1) {
                    eprintln!("⚠️ Предупреждение: Ошибка сканирования директории {:?}: {}", path, e);
                }
            } else {
                match self.extract_file_metadata(&path) {
                    Ok(metadata) => {
                        if self.should_include_file(&metadata) {
                            files.push(metadata);
                        }
                    }
                    Err(e) => {
                        // Более детальная информация об ошибках доступа к файлам
                        eprintln!("⚠️ Предупреждение: Не удалось прочитать файл {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Извлекает метаданные из файла
    fn extract_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                return Err(AnalysisError::GenericError(format!("Не удалось получить метаданные файла {:?}: {}", path, e)));
            }
        };
        
        let file_type = self.detect_file_type(path);
        
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                // Логируем ошибку, но не прерываем работу
                eprintln!("⚠️ Предупреждение: Не удалось прочитать содержимое файла {:?}: {}", path, e);
                String::new()
            }
        };
        
        let lines_count = content.lines().count();

        let last_modified = match metadata.modified() {
            Ok(time) => time.into(),
            Err(e) => {
                eprintln!("⚠️ Предупреждение: Не удалось получить время модификации файла {:?}: {}", path, e);
                std::time::SystemTime::now().into()
            }
        };

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
        if content_lower.contains("test") || content_lower.contains("#[test]") || content_lower.contains("describe(") {
            CapsuleStatus::Pending
        } else if content_lower.contains("deprecated") || content_lower.contains("@deprecated") {
            CapsuleStatus::Archived
        } else if content_lower.contains("todo") || content_lower.contains("fixme") {
            CapsuleStatus::Pending
        } else {
            CapsuleStatus::Active
        }
    }

    /// Извлекает импорты и экспорты
    fn extract_imports_exports(&self, content: &str, file_type: &FileType) -> (Vec<String>, Vec<String>) {
        match file_type {
            FileType::Rust => self.extract_rust_imports_exports(content),
            FileType::TypeScript | FileType::JavaScript => self.extract_ts_js_imports_exports(content),
            FileType::Python => self.extract_python_imports_exports(content),
            FileType::Java => self.extract_java_imports_exports(content),
            FileType::Cpp | FileType::C => self.extract_cpp_imports_exports(content),
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

    fn extract_java_imports_exports(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            // Java imports
            if trimmed.starts_with("import ") && trimmed.ends_with(";") {
                if let Some(import) = trimmed.strip_prefix("import ") {
                    let import_clean = import.trim_end_matches(';').trim();
                    if !import_clean.starts_with("static ") {
                        imports.push(import_clean.to_string());
                    }
                }
            }
            
            // Java public exports (classes, interfaces, methods)
            if trimmed.contains("public ") {
                if let Some(export) = extract_java_export_name(trimmed) {
                    exports.push(export);
                }
            }
        }

        (imports, exports)
    }

    fn extract_cpp_imports_exports(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            // C++ includes
            if trimmed.starts_with("#include ") {
                if let Some(include) = trimmed.strip_prefix("#include ") {
                    let include_clean = include.trim()
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .trim_start_matches('"')
                        .trim_end_matches('"');
                    imports.push(include_clean.to_string());
                }
            }
            
            // C++ exports (classes, functions, namespaces)
            if trimmed.contains("class ") || trimmed.contains("struct ") || 
               trimmed.contains("namespace ") || trimmed.contains("extern ") {
                if let Some(export) = extract_cpp_export_name(trimmed) {
                    exports.push(export);
                }
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

        let supported_extensions = ["rs", "js", "ts", "tsx", "jsx", "py", "java", "cpp", "cc", "cxx", "c", "h", "hpp", "hxx"];
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
fn glob_to_regex(pattern: &str) -> std::result::Result<regex::Regex, regex::Error> {
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
        format!(".*{regex_pattern}")
    } else {
        regex_pattern
    };
    
    regex::Regex::new(&final_pattern)
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

/// Извлекает имя экспорта из Java строки
fn extract_java_export_name(line: &str) -> Option<String> {
    if line.contains("public class ") {
        extract_name_after(line, "public class ")
    } else if line.contains("public interface ") {
        extract_name_after(line, "public interface ")
    } else if line.contains("public enum ") {
        extract_name_after(line, "public enum ")
    } else if line.contains("public static ") && line.contains("(") {
        // Public static methods
        if let Some(start) = line.find("public static") {
            let after_static = &line[start..];
            if let Some(method_start) = after_static.rfind(' ') {
                let method_part = &after_static[method_start + 1..];
                if let Some(paren_pos) = method_part.find('(') {
                    return Some(method_part[..paren_pos].trim().to_string());
                }
            }
        }
        None
    } else if line.contains("public ") && line.contains("(") {
        // Public methods
        if let Some(paren_pos) = line.find('(') {
            let before_paren = &line[..paren_pos];
            if let Some(method_start) = before_paren.rfind(' ') {
                return Some(before_paren[method_start + 1..].trim().to_string());
            }
        }
        None
    } else {
        None
    }
}

/// Извлекает имя экспорта из C++ строки
fn extract_cpp_export_name(line: &str) -> Option<String> {
    if line.contains("class ") {
        extract_name_after(line, "class ")
    } else if line.contains("struct ") {
        extract_name_after(line, "struct ")
    } else if line.contains("namespace ") {
        extract_name_after(line, "namespace ")
    } else if line.contains("extern ") {
        if line.contains("extern \"C\"") {
            // extern "C" functions
            if let Some(start) = line.find("extern \"C\"") {
                let after_extern = &line[start + 10..].trim();
                if let Some(paren_pos) = after_extern.find('(') {
                    let before_paren = &after_extern[..paren_pos];
                    if let Some(func_start) = before_paren.rfind(' ') {
                        return Some(before_paren[func_start + 1..].trim().to_string());
                    }
                }
            }
        }
        None
    } else {
        None
    }
}