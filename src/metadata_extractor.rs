use crate::parser_ast::{ASTElement, ASTElementType};
use crate::types::Result;
use std::collections::HashMap;
use std::path::Path;

/// Экстрактор метаданных - извлекает дополнительную информацию из элементов
#[derive(Debug)]
pub struct MetadataExtractor {
    patterns: HashMap<String, Vec<String>>,
}

impl Default for MetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataExtractor {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Паттерны для поиска TODO/FIXME
        patterns.insert(
            "todo".to_string(),
            vec!["TODO".to_string(), "todo".to_string(), "@todo".to_string()],
        );

        patterns.insert(
            "fixme".to_string(),
            vec![
                "FIXME".to_string(),
                "fixme".to_string(),
                "@fixme".to_string(),
            ],
        );

        patterns.insert(
            "deprecated".to_string(),
            vec![
                "deprecated".to_string(),
                "@deprecated".to_string(),
                "#[deprecated]".to_string(),
            ],
        );

        Self { patterns }
    }

    /// Извлекает метаданные из AST элемента
    pub fn extract_metadata(
        &self,
        element: &ASTElement,
        _file_path: &Path,
    ) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Базовая информация
        metadata.insert("type".to_string(), format!("{:?}", element.element_type));
        metadata.insert("visibility".to_string(), element.visibility.clone());
        metadata.insert("complexity".to_string(), element.complexity.to_string());
        metadata.insert(
            "line_count".to_string(),
            (element.end_line - element.start_line + 1).to_string(),
        );

        // Извлекаем паттерны
        for (pattern_name, patterns) in &self.patterns {
            let found = patterns
                .iter()
                .any(|pattern| element.content.contains(pattern) || element.name.contains(pattern));
            if found {
                metadata.insert(format!("has_{pattern_name}"), "true".to_string());
            }
        }

        // Дополнительная информация по типам
        self.extract_type_specific_metadata(element, &mut metadata)?;

        Ok(metadata)
    }

    /// Извлекает метаданные специфичные для типа элемента
    fn extract_type_specific_metadata(
        &self,
        element: &ASTElement,
        metadata: &mut HashMap<String, String>,
    ) -> Result<()> {
        match element.element_type {
            ASTElementType::Function | ASTElementType::Method => {
                self.extract_function_metadata(element, metadata)?;
            }
            ASTElementType::Class | ASTElementType::Struct => {
                self.extract_class_metadata(element, metadata)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Извлекает метаданные функции
    fn extract_function_metadata(
        &self,
        element: &ASTElement,
        metadata: &mut HashMap<String, String>,
    ) -> Result<()> {
        metadata.insert(
            "parameters_count".to_string(),
            element.parameters.len().to_string(),
        );

        // Попытка извлечь информацию о возвращаемом типе
        if let Some(return_type) = &element.return_type {
            metadata.insert("return_type".to_string(), return_type.clone());
        }

        // Проверяем на сложные функции
        if element.complexity > 10 {
            metadata.insert("high_complexity".to_string(), "true".to_string());
        }

        Ok(())
    }

    /// Извлекает метаданные класса/структуры
    fn extract_class_metadata(
        &self,
        element: &ASTElement,
        metadata: &mut HashMap<String, String>,
    ) -> Result<()> {
        // Считаем количество детей (методов, полей)
        metadata.insert(
            "children_count".to_string(),
            element.children.len().to_string(),
        );

        // Определяем является ли интерфейсом
        if element.element_type == ASTElementType::Interface {
            metadata.insert("is_interface".to_string(), "true".to_string());
        }

        Ok(())
    }

    /// Генерирует автоматический слоган для элемента
    pub fn generate_auto_slogan(&self, element: &ASTElement) -> Result<String> {
        let name = &element.name;
        let element_type = &element.element_type;

        // Проверяем на существующие комментарии
        if let Some(existing_slogan) = self.extract_doc_comment(&element.content) {
            return Ok(existing_slogan);
        }

        // Генерируем автоматически
        let auto_slogan = match element_type {
            ASTElementType::Function => {
                if name.starts_with("get_") {
                    format!(
                        "Получает данные: {}",
                        name.strip_prefix("get_").unwrap_or(name)
                    )
                } else if name.starts_with("set_") {
                    format!(
                        "Устанавливает: {}",
                        name.strip_prefix("set_").unwrap_or(name)
                    )
                } else if name.starts_with("create_") {
                    format!("Создает: {}", name.strip_prefix("create_").unwrap_or(name))
                } else if name.starts_with("delete_") {
                    format!("Удаляет: {}", name.strip_prefix("delete_").unwrap_or(name))
                } else if name.starts_with("validate_") {
                    format!(
                        "Валидирует: {}",
                        name.strip_prefix("validate_").unwrap_or(name)
                    )
                } else if name.starts_with("process_") {
                    format!(
                        "Обрабатывает: {}",
                        name.strip_prefix("process_").unwrap_or(name)
                    )
                } else {
                    format!("Функция: {name}")
                }
            }
            ASTElementType::Method => format!("Метод: {name}"),
            ASTElementType::Struct => format!("Структура данных: {name}"),
            ASTElementType::Enum => format!("Перечисление: {name}"),
            ASTElementType::Class => format!("Класс: {name}"),
            ASTElementType::Interface => format!("Интерфейс: {name}"),
            ASTElementType::Module => format!("Модуль: {name}"),
            _ => format!("Элемент: {name}"),
        };

        Ok(auto_slogan)
    }

    /// Извлекает документационный комментарий
    fn extract_doc_comment(&self, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().take(10).collect();

        for line in lines {
            let trimmed = line.trim();

            // Rust doc comments
            if trimmed.starts_with("///") {
                let comment = trimmed.trim_start_matches("///").trim();
                if !comment.is_empty() && comment.len() > 5 {
                    return Some(comment.to_string());
                }
            }

            // Multi-line comments
            if trimmed.starts_with("/**") {
                let comment = trimmed
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .trim();
                if !comment.is_empty() && comment.len() > 5 {
                    return Some(comment.to_string());
                }
            }

            // JavaScript/TypeScript comments
            if trimmed.starts_with("// ") {
                let comment = trimmed.trim_start_matches("// ").trim();
                if !comment.is_empty() && comment.len() > 5 {
                    return Some(comment.to_string());
                }
            }
        }

        None
    }

    /// Анализирует качество документации
    pub fn analyze_documentation_quality(&self, element: &ASTElement) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Проверка наличия документации для публичных элементов
        if element.visibility == "public"
            && !matches!(element.element_type, ASTElementType::Import)
            && self.extract_doc_comment(&element.content).is_none()
        {
            issues.push("Публичный элемент без документации".to_string());
        }

        // Проверка качества имен
        if element.name.len() < 3 {
            issues.push("Слишком короткое имя".to_string());
        }

        if element.name.chars().all(|c| c.is_lowercase() || c == '_') && element.name.len() > 20 {
            issues.push("Длинное имя без разделителей".to_string());
        }

        Ok(issues)
    }
}
