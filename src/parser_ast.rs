use std::path::Path;
use std::collections::HashMap;
use crate::types::{FileType, Result};
use regex::Regex;

// Продвинутый парсер с комбинированным подходом: tree-sitter + regex fallback
// Обеспечивает высокое качество анализа с максимальной совместимостью

/// Элемент AST (структурная единица кода)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ASTElement {
    pub id: uuid::Uuid,
    pub name: String,
    pub element_type: ASTElementType,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub complexity: u32,
    pub visibility: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub children: Vec<uuid::Uuid>,
    pub parent_id: Option<uuid::Uuid>,
    pub metadata: HashMap<String, String>,
}

/// Типы AST элементов
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ASTElementType {
    Module,
    Class,
    Interface,
    Struct,
    Enum,
    Function,
    Method,
    Variable,
    Constant,
    Import,
    Export,
    Comment,
    Other(String),
}

/// Продвинутый парсер AST с поддержкой множественных стратегий
#[derive(Debug)]
pub struct ParserAST {
    // Паттерны для различных языков (regex путь)
    rust_patterns: LanguagePatterns,
    js_patterns: LanguagePatterns,
    ts_patterns: LanguagePatterns,
    python_patterns: LanguagePatterns,
    java_patterns: LanguagePatterns,
    cpp_patterns: LanguagePatterns,
    go_patterns: LanguagePatterns,
    
    // Кеш для оптимизации
    pattern_cache: HashMap<String, Vec<ASTElement>>,
}

/// Паттерны для конкретного языка
#[derive(Debug)]
pub struct LanguagePatterns {
    pub functions: Regex,
    pub classes: Regex,
    pub structs: Regex,
    pub enums: Regex,
    pub interfaces: Regex,
    pub modules: Regex,
    pub imports: Regex,
    pub exports: Regex,
    pub variables: Regex,
    pub constants: Regex,
    pub comments: Regex,
    pub complexity_indicators: Vec<Regex>,
}

impl ParserAST {
    pub fn new() -> Result<Self> {
        Ok(Self {
            rust_patterns: Self::create_rust_patterns()?,
            js_patterns: Self::create_js_patterns()?,
            ts_patterns: Self::create_ts_patterns()?,
            python_patterns: Self::create_python_patterns()?,
            java_patterns: Self::create_java_patterns()?,
            cpp_patterns: Self::create_cpp_patterns()?,
            go_patterns: Self::create_go_patterns()?,
            pattern_cache: HashMap::new(),
        })
    }
    
    fn create_rust_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*[^{]+)?\s*\{")?,
            classes: Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:struct|enum|union)\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*(?:pub\s+)?struct\s+(\w+)")?,
            enums: Regex::new(r"(?m)^[\s]*(?:pub\s+)?enum\s+(\w+)")?,
            interfaces: Regex::new(r"(?m)^[\s]*(?:pub\s+)?trait\s+(\w+)")?,
            modules: Regex::new(r"(?m)^[\s]*(?:pub\s+)?mod\s+(\w+)")?,
            imports: Regex::new(r"(?m)^[\s]*use\s+([^;]+);")?,
            exports: Regex::new(r"(?m)^[\s]*pub\s+(?:fn|struct|enum|trait|mod|const|static)\s+(\w+)")?,
            variables: Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:let|const|static)\s+(?:mut\s+)?(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*(?:pub\s+)?const\s+(\w+)")?,
            comments: Regex::new(r"(?m)^[\s]*(?://|/\*|\*|///).*")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\bmatch\b")?,
                Regex::new(r"\bloop\b")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }
    
    fn create_js_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>")?,
            classes: Regex::new(r"(?m)^[\s]*(?:export\s+)?class\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*(?:export\s+)?interface\s+(\w+)")?,
            enums: Regex::new(r"(?m)^[\s]*(?:export\s+)?enum\s+(\w+)")?,
            interfaces: Regex::new(r"(?m)^[\s]*(?:export\s+)?interface\s+(\w+)")?,
            modules: Regex::new(r"(?m)^[\s]*(?:export\s+)?namespace\s+(\w+)")?,
            imports: Regex::new(r#"(?m)^[\s]*import\s+[^from]*from\s+['"]([^'"]+)['"]"#)?,
            exports: Regex::new(r"(?m)^[\s]*export\s+(?:default\s+)?(?:function|class|const|let|var)\s+(\w+)")?,
            variables: Regex::new(r"(?m)^[\s]*(?:export\s+)?(?:const|let|var)\s+(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*(?:export\s+)?const\s+(\w+)")?,
            comments: Regex::new(r"(?m)^[\s]*(?://|/\*|\*)")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\bswitch\b")?,
                Regex::new(r"\bcase\b")?,
                Regex::new(r"\btry\b")?,
                Regex::new(r"\bcatch\b")?,
                Regex::new(r"\?\?")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }
    
    fn create_ts_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>")?,
            classes: Regex::new(r"(?m)^[\s]*(?:export\s+)?class\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*(?:export\s+)?type\s+(\w+)\s*=")?,
            enums: Regex::new(r"(?m)^[\s]*(?:export\s+)?enum\s+(\w+)")?,
            interfaces: Regex::new(r"(?m)^[\s]*(?:export\s+)?interface\s+(\w+)")?,
            modules: Regex::new(r"(?m)^[\s]*(?:export\s+)?namespace\s+(\w+)")?,
            imports: Regex::new(r#"(?m)^[\s]*import\s+[^from]*from\s+['"]([^'"]+)['"]"#)?,
            exports: Regex::new(r"(?m)^[\s]*export\s+(?:default\s+)?(?:function|class|const|let|var|interface|type|enum)\s+(\w+)")?,
            variables: Regex::new(r"(?m)^[\s]*(?:export\s+)?(?:const|let|var)\s+(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*(?:export\s+)?const\s+(\w+)")?,
            comments: Regex::new(r"(?m)^[\s]*(?://|/\*|\*)")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\bswitch\b")?,
                Regex::new(r"\bcase\b")?,
                Regex::new(r"\btry\b")?,
                Regex::new(r"\bcatch\b")?,
                Regex::new(r"\?\?")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }
    
    fn create_python_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:async\s+)?def\s+(\w+)\s*\(")?,
            classes: Regex::new(r"(?m)^[\s]*class\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*@dataclass\s+class\s+(\w+)")?,
            enums: Regex::new(r"(?m)^[\s]*class\s+(\w+)\s*\(.*Enum\)")?,
            interfaces: Regex::new(r"(?m)^[\s]*class\s+(\w+)\s*\(.*Protocol\)")?,
            modules: Regex::new(r"(?m)^[\s]*(?:from\s+\w+\s+)?import\s+(\w+)")?,
            imports: Regex::new(r"(?m)^[\s]*(?:from\s+([^\s]+)\s+)?import")?,
            exports: Regex::new(r"(?m)^[\s]*__all__\s*=")?,
            variables: Regex::new(r"(?m)^[\s]*(\w+)\s*=")?,
            constants: Regex::new(r"(?m)^[\s]*([A-Z_][A-Z0-9_]*)\s*=")?,
            comments: Regex::new(r"(?m)^[\s]*#")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\btry\b")?,
                Regex::new(r"\bexcept\b")?,
                Regex::new(r"\band\b")?,
                Regex::new(r"\bor\b")?,
                Regex::new(r"\bnot\b")?,
            ],
        })
    }
    
    fn create_java_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?[\w<>]+\s+(\w+)\s*\(")?,
            classes: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*(?:abstract\s+)?(?:final\s+)?class\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*record\s+(\w+)")?,
            enums: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*enum\s+(\w+)")?,
            interfaces: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*interface\s+(\w+)")?,
            modules: Regex::new(r"(?m)^[\s]*package\s+([^;]+)")?,
            imports: Regex::new(r"(?m)^[\s]*import\s+([^;]+)")?,
            exports: Regex::new(r"(?m)^[\s]*public\s+(?:class|interface|enum)\s+(\w+)")?,
            variables: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?[\w<>]+\s+(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*(?:public|private|protected)?\s*static\s+final\s+[\w<>]+\s+([A-Z_][A-Z0-9_]*)")?,
            comments: Regex::new(r"(?m)^[\s]*(?://|/\*|\*)")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\bswitch\b")?,
                Regex::new(r"\bcase\b")?,
                Regex::new(r"\btry\b")?,
                Regex::new(r"\bcatch\b")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }
    
    fn create_cpp_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*(?:inline\s+)?(?:virtual\s+)?(?:static\s+)?[\w:*&<>]+\s+(\w+)\s*\(")?,
            classes: Regex::new(r"(?m)^[\s]*(?:template\s*<[^>]*>\s*)?class\s+(\w+)")?,
            structs: Regex::new(r"(?m)^[\s]*(?:template\s*<[^>]*>\s*)?struct\s+(\w+)")?,
            enums: Regex::new(r"(?m)^[\s]*enum\s+(?:class\s+)?(\w+)")?,
            interfaces: Regex::new(r"(?m)^[\s]*(?:template\s*<[^>]*>\s*)?class\s+(\w+)")?,
            modules: Regex::new(r"(?m)^[\s]*namespace\s+(\w+)")?,
            imports: Regex::new(r#"(?m)^[\s]*#include\s+[<"]([^>"]+)[>"]"#)?,
            exports: Regex::new(r"(?m)^[\s]*(?:extern\s+)?(?:template\s*<[^>]*>\s*)?(?:class|struct|enum)\s+(\w+)")?,
            variables: Regex::new(r"(?m)^[\s]*(?:extern\s+)?(?:static\s+)?(?:const\s+)?[\w:*&<>]+\s+(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*(?:extern\s+)?(?:static\s+)?const\s+[\w:*&<>]+\s+([A-Z_][A-Z0-9_]*)")?,
            comments: Regex::new(r"(?m)^[\s]*(?://|/\*|\*)")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bwhile\b")?,
                Regex::new(r"\bswitch\b")?,
                Regex::new(r"\bcase\b")?,
                Regex::new(r"\btry\b")?,
                Regex::new(r"\bcatch\b")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }
    
    fn create_go_patterns() -> Result<LanguagePatterns> {
        Ok(LanguagePatterns {
            functions: Regex::new(r"(?m)^[\s]*func\s+(?:\([^)]*\)\s*)?(\w+)\s*\(")?,
            classes: Regex::new(r"(?m)^[\s]*type\s+(\w+)\s+struct")?,
            structs: Regex::new(r"(?m)^[\s]*type\s+(\w+)\s+struct")?,
            enums: Regex::new(r"(?m)^[\s]*type\s+(\w+)\s+\w+")?,
            interfaces: Regex::new(r"(?m)^[\s]*type\s+(\w+)\s+interface")?,
            modules: Regex::new(r"(?m)^[\s]*package\s+(\w+)")?,
            imports: Regex::new(r#"(?m)^[\s]*import\s+(?:\([^)]*\)|"([^"]+)")"#)?,
            exports: Regex::new(r"(?m)^[\s]*(?:func|type|var|const)\s+([A-Z]\w*)")?,
            variables: Regex::new(r"(?m)^[\s]*var\s+(\w+)")?,
            constants: Regex::new(r"(?m)^[\s]*const\s+(\w+)")?,
            comments: Regex::new(r"(?m)^[\s]*//")?,
            complexity_indicators: vec![
                Regex::new(r"\bif\b")?,
                Regex::new(r"\belse\b")?,
                Regex::new(r"\bfor\b")?,
                Regex::new(r"\bswitch\b")?,
                Regex::new(r"\bcase\b")?,
                Regex::new(r"\bselect\b")?,
                Regex::new(r"&&")?,
                Regex::new(r"\|\|")?,
            ],
        })
    }

    /// Парсит файл: если включён feature `tree_sitter`, используем парсер tree-sitter для поддерживаемых языков,
    /// иначе — regex fallback. На ошибки — безопасный откат к regex.
    pub fn parse_file(&mut self, file_path: &Path, content: &str, file_type: &FileType) -> Result<Vec<ASTElement>> {
        let cache_key = format!("{}:{}", file_path.display(), content.len());
        if let Some(cached) = self.pattern_cache.get(&cache_key) { return Ok(cached.clone()); }

        #[cfg(feature = "tree_sitter")]
        {
            if let Some(elements) = self.try_tree_sitter_parse(file_path, content, file_type)? {
                self.pattern_cache.insert(cache_key, elements.clone());
                return Ok(elements);
            }
        }
        // Fallback regex
        let elements = self.parse_file_regex(file_path, content, file_type)?;
        self.pattern_cache.insert(cache_key, elements.clone());
        Ok(elements)
    }

    #[cfg(feature = "tree_sitter")]
    fn try_tree_sitter_parse(&self, file_path: &Path, content: &str, file_type: &FileType) -> Result<Option<Vec<ASTElement>>> {
        use tree_sitter::{Parser, Node};
        // Сейчас реализуем Rust, остальные языки идут по fallback
        if !matches!(file_type, FileType::Rust) { return Ok(None); }
        if content.trim().is_empty() { return Ok(Some(Vec::new())); }

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language())
            .map_err(|e| crate::types::AnalysisError::Parse(format!("tree-sitter rust: {e:?}")))?;
        let tree = match parser.parse(content, None) { Some(t) => t, None => return Ok(None) };
        let mut cursor = tree.walk();

        let mut elements: Vec<ASTElement> = Vec::new();
        self.ts_collect_rust_nodes(content, file_path, tree.root_node(), &mut elements)?;

        Ok(Some(elements))
    }

    fn parse_file_regex(&self, file_path: &Path, content: &str, file_type: &FileType) -> Result<Vec<ASTElement>> {
        let patterns = match file_type {
            FileType::Rust => &self.rust_patterns,
            FileType::JavaScript => &self.js_patterns,
            FileType::TypeScript => &self.ts_patterns,
            FileType::Python => &self.python_patterns,
            FileType::Java => &self.java_patterns,
            FileType::Cpp => &self.cpp_patterns,
            FileType::Go => &self.go_patterns,
            _ => return Ok(vec![]),
        };
        let mut elements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(element) = self.parse_line_advanced(line, line_num + 1, patterns, file_type) {
                elements.push(element);
            }
        }
        for element in &mut elements { element.complexity = self.calculate_complexity(&element.content, patterns); }
        Ok(elements)
    }

    /// Продвинутый анализ строки с использованием regex паттернов
    fn parse_line_advanced(&self, line: &str, line_num: usize, patterns: &LanguagePatterns, file_type: &FileType) -> Option<ASTElement> {
        let trimmed = line.trim();
        if trimmed.is_empty() || patterns.comments.is_match(trimmed) {
            return None;
        }

        // Проверяем паттерны в порядке приоритета
        if let Some(caps) = patterns.functions.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Function,
                line,
                line_num,
                self.extract_visibility(trimmed),
                self.extract_parameters(trimmed),
                self.extract_return_type(trimmed, file_type),
            ));
        }

        if let Some(caps) = patterns.classes.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Class,
                line,
                line_num,
                self.extract_visibility(trimmed),
                vec![],
                None,
            ));
        }

        if let Some(caps) = patterns.structs.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Struct,
                line,
                line_num,
                self.extract_visibility(trimmed),
                vec![],
                None,
            ));
        }

        if let Some(caps) = patterns.enums.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Enum,
                line,
                line_num,
                self.extract_visibility(trimmed),
                vec![],
                None,
            ));
        }

        if let Some(caps) = patterns.interfaces.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Interface,
                line,
                line_num,
                self.extract_visibility(trimmed),
                vec![],
                None,
            ));
        }

        if let Some(caps) = patterns.modules.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Module,
                line,
                line_num,
                self.extract_visibility(trimmed),
                vec![],
                None,
            ));
        }

        if let Some(caps) = patterns.imports.captures(trimmed) {
            return Some(self.create_element(
                caps.get(1)?.as_str(),
                ASTElementType::Import,
                line,
                line_num,
                "public".to_string(),
                vec![],
                None,
            ));
        }

        None
    }

    /// Создает элемент AST
    fn create_element(&self, name: &str, element_type: ASTElementType, content: &str, line_num: usize, visibility: String, parameters: Vec<String>, return_type: Option<String>) -> ASTElement {
        ASTElement {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            element_type,
            content: content.to_string(),
            start_line: line_num,
            end_line: line_num,
            start_column: 0,
            end_column: content.len(),
            complexity: 1, // Будет пересчитано позже
            visibility,
            parameters,
            return_type,
            children: vec![],
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Извлекает видимость элемента
    fn extract_visibility(&self, line: &str) -> String {
        if line.contains("pub ") || line.contains("public ") {
            "public".to_string()
        } else if line.contains("private ") {
            "private".to_string()
        } else if line.contains("protected ") {
            "protected".to_string()
        } else {
            "public".to_string()
        }
    }

    /// Извлекает параметры функции
    fn extract_parameters(&self, line: &str) -> Vec<String> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(')') {
                let params_str = &line[start + 1..start + end];
                if params_str.trim().is_empty() {
                    return vec![];
                }
                return params_str.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        vec![]
    }

    /// Извлекает тип возвращаемого значения
    fn extract_return_type(&self, line: &str, file_type: &FileType) -> Option<String> {
        match file_type {
            FileType::Rust => {
                if let Some(arrow_pos) = line.find("->") {
                    let after_arrow = &line[arrow_pos + 2..];
                    if let Some(brace_pos) = after_arrow.find('{') {
                        return Some(after_arrow[..brace_pos].trim().to_string());
                    }
                }
            }
            FileType::TypeScript => {
                if let Some(colon_pos) = line.find(": ") {
                    let after_colon = &line[colon_pos + 2..];
                    if let Some(brace_pos) = after_colon.find('{') {
                        return Some(after_colon[..brace_pos].trim().to_string());
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// Вычисляет реальную сложность на основе содержимого
    fn calculate_complexity(&self, content: &str, patterns: &LanguagePatterns) -> u32 {
        let mut complexity = 1;
        
        for indicator in &patterns.complexity_indicators {
            complexity += indicator.find_iter(content).count() as u32;
        }
        
        // Добавляем сложность на основе других факторов
        let lines_count = content.lines().count() as u32;
        complexity += lines_count / 10; // Добавляем 1 за каждые 10 строк
        
        // Добавляем сложность для вложенных структур
        let nesting_level = self.calculate_nesting_level(content);
        complexity += nesting_level * 2;
        
        complexity
    }

    /// Вычисляет уровень вложенности
    fn calculate_nesting_level(&self, content: &str) -> u32 {
        let mut max_level = 0;
        let mut current_level: i32 = 0;
        
        for ch in content.chars() {
            match ch {
                '{' | '(' | '[' => {
                    current_level += 1;
                    max_level = max_level.max(current_level);
                }
                '}' | ')' | ']' => {
                    current_level = current_level.saturating_sub(1);
                }
                _ => {}
            }
        }
        
        max_level.max(0) as u32
    }
}

#[cfg(feature = "tree_sitter")]
impl ParserAST {
    fn ts_collect_rust_nodes(&self, content: &str, file_path: &Path, node: tree_sitter::Node, out: &mut Vec<ASTElement>) -> Result<()> {
        // DFS with ancestor flag: inside impl/trait
        let mut stack: Vec<(tree_sitter::Node, bool)> = vec![(node, false)];
        while let Some((n, in_impl_trait)) = stack.pop() {
            let kind = n.kind();
            let now_in_impl_trait = in_impl_trait || kind == "impl_item" || kind == "trait_item";
            match kind {
                "function_item" => {
                    if let Some(el) = self.ts_build_fn_element(content, &n, file_path, now_in_impl_trait)? { out.push(el); }
                },
                "struct_item" => { if let Some(el) = self.ts_build_named_element(content, &n, file_path, ASTElementType::Struct)? { out.push(el); } },
                "enum_item"   => { if let Some(el) = self.ts_build_named_element(content, &n, file_path, ASTElementType::Enum)? { out.push(el); } },
                "trait_item"  => { if let Some(el) = self.ts_build_named_element(content, &n, file_path, ASTElementType::Interface)? { out.push(el); } },
                "mod_item"    => { if let Some(el) = self.ts_build_named_element(content, &n, file_path, ASTElementType::Module)? { out.push(el); } },
                "use_declaration" => { if let Some(el) = self.ts_build_import_element(content, &n, file_path)? { out.push(el); } },
                _ => {}
            }
            for i in 0..n.child_count() {
                if let Some(ch) = n.child(i) { stack.push((ch, now_in_impl_trait)); }
            }
        }
        Ok(())
    }

    fn ts_text<'a>(&self, content: &'a str, node: &tree_sitter::Node) -> &'a str {
        let range = node.byte_range();
        let bytes = content.as_bytes();
        let start = range.start.min(bytes.len());
        let end = range.end.min(bytes.len());
        &content[start..end]
    }

    fn ts_find_child<'a>(&self, node: &'a tree_sitter::Node, kind: &str) -> Option<tree_sitter::Node<'a>> {
        for i in 0..node.child_count() { if let Some(ch) = node.child(i) { if ch.kind() == kind { return Some(ch); } } }
        None
    }

    fn ts_identifier<'a>(&self, node: &'a tree_sitter::Node) -> Option<tree_sitter::Node<'a>> {
        self.ts_find_child(node, "identifier").or_else(|| self.ts_find_child(node, "type_identifier"))
    }

    fn ts_visibility(&self, node: &tree_sitter::Node, content: &str) -> String {
        let text = self.ts_text(content, node);
        if text.trim_start().starts_with("pub") { "public".to_string() } else { "private".to_string() }
    }

    fn ts_build_named_element(&self, content: &str, node: &tree_sitter::Node, _file_path: &Path, etype: ASTElementType) -> Result<Option<ASTElement>> {
        let id_node = match self.ts_identifier(node) { Some(n) => n, None => return Ok(None) };
        let name = self.ts_text(content, &id_node).trim().to_string();
        let text = self.ts_text(content, node).to_string();
        let start = node.start_position();
        let end = node.end_position();
        let visibility = self.ts_visibility(node, content);
        let element = ASTElement {
            id: uuid::Uuid::new_v4(),
            name,
            element_type: etype,
            content: text.clone(),
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_column: start.column,
            end_column: end.column,
            complexity: 1,
            visibility,
            parameters: Vec::new(),
            return_type: None,
            children: Vec::new(),
            parent_id: None,
            metadata: HashMap::new(),
        };
        Ok(Some(element))
    }

    fn ts_build_fn_element(&self, content: &str, node: &tree_sitter::Node, _file_path: &Path, is_method: bool) -> Result<Option<ASTElement>> {
        let id_node = match self.ts_identifier(node) { Some(n) => n, None => return Ok(None) };
        let name = self.ts_text(content, &id_node).trim().to_string();
        let text = self.ts_text(content, node).to_string();
        let start = node.start_position();
        let end = node.end_position();
        let visibility = self.ts_visibility(node, content);

        // Parameters by substring between first '(' and matching ')'
        let mut parameters: Vec<String> = Vec::new();
        if let Some(lp) = text.find('(') { if let Some(rp) = text[lp..].find(')') { let inside = &text[lp+1 .. lp+rp]; parameters = inside.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(); } }
        // Return type by substring after '->'
        let return_type = if let Some(arrow) = text.find("->") { Some(text[arrow+2..].split('{').next().unwrap_or("").trim().to_string()) } else { None };

        let mut element = ASTElement {
            id: uuid::Uuid::new_v4(),
            name,
            element_type: if is_method { ASTElementType::Method } else { ASTElementType::Function },
            content: text.clone(),
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_column: start.column,
            end_column: end.column,
            complexity: 1,
            visibility,
            parameters,
            return_type,
            children: Vec::new(),
            parent_id: None,
            metadata: HashMap::new(),
        };
        // complexity from regex indicators
        let patterns = &self.rust_patterns;
        element.complexity = self.calculate_complexity(&element.content, patterns);
        Ok(Some(element))
    }

    fn ts_build_import_element(&self, content: &str, node: &tree_sitter::Node, _file_path: &Path) -> Result<Option<ASTElement>> {
        let text = self.ts_text(content, node).to_string();
        // Try to extract simple path after 'use'
        let name = text.trim().trim_start_matches("use").trim().trim_end_matches(';').split_whitespace().next().unwrap_or("").to_string();
        let start = node.start_position();
        let end = node.end_position();
        let element = ASTElement {
            id: uuid::Uuid::new_v4(),
            name,
            element_type: ASTElementType::Import,
            content: text,
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_column: start.column,
            end_column: end.column,
            complexity: 1,
            visibility: "public".to_string(),
            parameters: Vec::new(),
            return_type: None,
            children: Vec::new(),
            parent_id: None,
            metadata: HashMap::new(),
        };
        Ok(Some(element))
    }
}

impl Default for ParserAST {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| ParserAST {
            rust_patterns: LanguagePatterns {
                functions: Regex::new(r"fn\s+(\w+)").unwrap(),
                classes: Regex::new(r"struct\s+(\w+)").unwrap(),
                structs: Regex::new(r"struct\s+(\w+)").unwrap(),
                enums: Regex::new(r"enum\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"trait\s+(\w+)").unwrap(),
                modules: Regex::new(r"mod\s+(\w+)").unwrap(),
                imports: Regex::new(r"use\s+([^;]+)").unwrap(),
                exports: Regex::new(r"pub\s+\w+\s+(\w+)").unwrap(),
                variables: Regex::new(r"let\s+(\w+)").unwrap(),
                constants: Regex::new(r"const\s+(\w+)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            js_patterns: LanguagePatterns {
                functions: Regex::new(r"function\s+(\w+)").unwrap(),
                classes: Regex::new(r"class\s+(\w+)").unwrap(),
                structs: Regex::new(r"interface\s+(\w+)").unwrap(),
                enums: Regex::new(r"enum\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"interface\s+(\w+)").unwrap(),
                modules: Regex::new(r"namespace\s+(\w+)").unwrap(),
                imports: Regex::new(r"import.*from").unwrap(),
                exports: Regex::new(r"export.*(\w+)").unwrap(),
                variables: Regex::new(r"(?:const|let|var)\s+(\w+)").unwrap(),
                constants: Regex::new(r"const\s+(\w+)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            ts_patterns: LanguagePatterns {
                functions: Regex::new(r"function\s+(\w+)").unwrap(),
                classes: Regex::new(r"class\s+(\w+)").unwrap(),
                structs: Regex::new(r"interface\s+(\w+)").unwrap(),
                enums: Regex::new(r"enum\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"interface\s+(\w+)").unwrap(),
                modules: Regex::new(r"namespace\s+(\w+)").unwrap(),
                imports: Regex::new(r"import.*from").unwrap(),
                exports: Regex::new(r"export.*(\w+)").unwrap(),
                variables: Regex::new(r"(?:const|let|var)\s+(\w+)").unwrap(),
                constants: Regex::new(r"const\s+(\w+)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            python_patterns: LanguagePatterns {
                functions: Regex::new(r"def\s+(\w+)").unwrap(),
                classes: Regex::new(r"class\s+(\w+)").unwrap(),
                structs: Regex::new(r"class\s+(\w+)").unwrap(),
                enums: Regex::new(r"class\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"class\s+(\w+)").unwrap(),
                modules: Regex::new(r"import\s+(\w+)").unwrap(),
                imports: Regex::new(r"import").unwrap(),
                exports: Regex::new(r"__all__").unwrap(),
                variables: Regex::new(r"(\w+)\s*=").unwrap(),
                constants: Regex::new(r"([A-Z_][A-Z0-9_]*)\s*=").unwrap(),
                comments: Regex::new(r"#").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            java_patterns: LanguagePatterns {
                functions: Regex::new(r"(?:public|private|protected)?\s*\w+\s+(\w+)\s*\(").unwrap(),
                classes: Regex::new(r"class\s+(\w+)").unwrap(),
                structs: Regex::new(r"record\s+(\w+)").unwrap(),
                enums: Regex::new(r"enum\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"interface\s+(\w+)").unwrap(),
                modules: Regex::new(r"package\s+(\w+)").unwrap(),
                imports: Regex::new(r"import\s+(\w+)").unwrap(),
                exports: Regex::new(r"public.*(\w+)").unwrap(),
                variables: Regex::new(r"\w+\s+(\w+)").unwrap(),
                constants: Regex::new(r"final\s+\w+\s+([A-Z_][A-Z0-9_]*)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            cpp_patterns: LanguagePatterns {
                functions: Regex::new(r"\w+\s+(\w+)\s*\(").unwrap(),
                classes: Regex::new(r"class\s+(\w+)").unwrap(),
                structs: Regex::new(r"struct\s+(\w+)").unwrap(),
                enums: Regex::new(r"enum\s+(\w+)").unwrap(),
                interfaces: Regex::new(r"class\s+(\w+)").unwrap(),
                modules: Regex::new(r"namespace\s+(\w+)").unwrap(),
                imports: Regex::new(r#"#include\s+[<"]([^>"]+)[>"]"#).unwrap(),
                exports: Regex::new(r"extern.*(\w+)").unwrap(),
                variables: Regex::new(r"\w+\s+(\w+)").unwrap(),
                constants: Regex::new(r"const\s+\w+\s+([A-Z_][A-Z0-9_]*)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bwhile\b").unwrap(),
                ],
            },
            go_patterns: LanguagePatterns {
                functions: Regex::new(r"func\s+(\w+)").unwrap(),
                classes: Regex::new(r"type\s+(\w+)\s+struct").unwrap(),
                structs: Regex::new(r"type\s+(\w+)\s+struct").unwrap(),
                enums: Regex::new(r"type\s+(\w+)\s+\w+").unwrap(),
                interfaces: Regex::new(r"type\s+(\w+)\s+interface").unwrap(),
                modules: Regex::new(r"package\s+(\w+)").unwrap(),
                imports: Regex::new(r#"import.*"([^"]+)""#).unwrap(),
                exports: Regex::new(r"(?:func|type|var|const)\s+([A-Z]\w*)").unwrap(),
                variables: Regex::new(r"var\s+(\w+)").unwrap(),
                constants: Regex::new(r"const\s+(\w+)").unwrap(),
                comments: Regex::new(r"//").unwrap(),
                complexity_indicators: vec![
                    Regex::new(r"\bif\b").unwrap(),
                    Regex::new(r"\bfor\b").unwrap(),
                    Regex::new(r"\bswitch\b").unwrap(),
                ],
            },
            pattern_cache: HashMap::new(),
        })
    }
} 
