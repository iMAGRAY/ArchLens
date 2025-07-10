use std::path::PathBuf;
use crate::core::{Capsule, CapsuleType, Priority, CapsuleStatus, Result};
use crate::parser_ast::ASTElement;
use crate::metadata_extractor::MetadataExtractor;
use std::collections::HashMap;
use chrono::Utc;

/// Конструктор капсул - создает архитектурные капсулы из AST элементов
#[derive(Debug)]
pub struct CapsuleConstructor {
    // Настройки конструктора
    pub min_complexity_threshold: u32,
    pub max_capsule_size: usize,
}

impl CapsuleConstructor {
    pub fn new() -> Self {
        Self {
            min_complexity_threshold: 5,
            max_capsule_size: 1000,
        }
    }

    /// Создает капсулы из AST элементов
    pub fn create_capsules(
        &self,
        ast_elements: &[ASTElement],
        file_path: &PathBuf,
    ) -> Result<Vec<Capsule>> {
        let mut capsules = Vec::new();

        for element in ast_elements {
            if let Some(capsule) = self.create_capsule_from_element(element, file_path)? {
                capsules.push(capsule);
            }
        }

        Ok(capsules)
    }

    /// Создает капсулу из AST элемента
    fn create_capsule_from_element(
        &self,
        element: &ASTElement,
        file_path: &PathBuf,
    ) -> Result<Option<Capsule>> {
        // Фильтруем элементы по значимости
        if !self.is_significant_element(element) {
            return Ok(None);
        }

        let capsule_type = self.convert_ast_type_to_capsule_type(&element.element_type);
        let priority = self.calculate_priority(element);
        let status = self.determine_status(element);
        let layer = self.determine_layer(file_path);
        let slogan = self.generate_slogan(element);
        let warnings = self.analyze_warnings(element);

        let capsule = Capsule {
            id: element.id,
            name: element.name.clone(),
            capsule_type,
            file_path: file_path.to_path_buf(),
            line_start: element.start_line,
            line_end: element.end_line,
            complexity: element.complexity,
            priority,
            status,
            layer: Some(layer),
            slogan: Some(slogan),
            summary: None,
            warnings,
            dependencies: vec![],
            dependents: vec![],
            metadata: element.metadata.clone(),
            created_at: Utc::now(),
        };

        Ok(Some(capsule))
    }

    /// Проверяет значимость элемента
    fn is_significant_element(&self, element: &ASTElement) -> bool {
        match element.element_type {
            crate::parser_ast::ASTElementType::Function | crate::parser_ast::ASTElementType::Method => true,
            crate::parser_ast::ASTElementType::Class | crate::parser_ast::ASTElementType::Struct => true,
            crate::parser_ast::ASTElementType::Interface | crate::parser_ast::ASTElementType::Enum => true,
            crate::parser_ast::ASTElementType::Module => true,
            crate::parser_ast::ASTElementType::Constant => element.visibility == "public",
            crate::parser_ast::ASTElementType::Variable => element.visibility == "public",
            crate::parser_ast::ASTElementType::Import | crate::parser_ast::ASTElementType::Export => false,
            crate::parser_ast::ASTElementType::Comment => false,
            crate::parser_ast::ASTElementType::Other(_) => false,
        }
    }

    /// Конвертирует AST тип в тип капсулы
    fn convert_ast_type_to_capsule_type(&self, ast_type: &crate::parser_ast::ASTElementType) -> CapsuleType {
        match ast_type {
            crate::parser_ast::ASTElementType::Function => CapsuleType::Function,
            crate::parser_ast::ASTElementType::Method => CapsuleType::Method,
            crate::parser_ast::ASTElementType::Class => CapsuleType::Class,
            crate::parser_ast::ASTElementType::Struct => CapsuleType::Struct,
            crate::parser_ast::ASTElementType::Interface => CapsuleType::Interface,
            crate::parser_ast::ASTElementType::Enum => CapsuleType::Enum,
            crate::parser_ast::ASTElementType::Module => CapsuleType::Module,
            crate::parser_ast::ASTElementType::Constant => CapsuleType::Constant,
            crate::parser_ast::ASTElementType::Variable => CapsuleType::Variable,
            crate::parser_ast::ASTElementType::Import => CapsuleType::Import,
            crate::parser_ast::ASTElementType::Export => CapsuleType::Export,
            crate::parser_ast::ASTElementType::Comment => CapsuleType::Other,
            crate::parser_ast::ASTElementType::Other(_) => CapsuleType::Other,
        }
    }

    /// Вычисляет приоритет элемента
    fn calculate_priority(&self, element: &ASTElement) -> Priority {
        match element.element_type {
            crate::parser_ast::ASTElementType::Class | crate::parser_ast::ASTElementType::Interface => Priority::High,
            crate::parser_ast::ASTElementType::Struct | crate::parser_ast::ASTElementType::Enum => Priority::Medium,
            crate::parser_ast::ASTElementType::Function | crate::parser_ast::ASTElementType::Method => {
                if element.visibility == "public" {
                    Priority::Medium
                } else {
                    Priority::Low
                }
            }
            crate::parser_ast::ASTElementType::Module => Priority::High,
            _ => Priority::Low,
        }
    }

    /// Определяет статус элемента
    fn determine_status(&self, element: &ASTElement) -> CapsuleStatus {
        let content_lower = element.content.to_lowercase();
        
        if content_lower.contains("deprecated") || content_lower.contains("@deprecated") {
            CapsuleStatus::Deprecated
        } else if content_lower.contains("todo") || content_lower.contains("fixme") {
            CapsuleStatus::Unstable
        } else if content_lower.contains("@experimental") || content_lower.contains("experimental") {
            CapsuleStatus::Experimental
        } else if element.visibility == "private" {
            CapsuleStatus::Internal
        } else {
            CapsuleStatus::Active
        }
    }

    /// Определяет архитектурный слой
    fn determine_layer(&self, file_path: &PathBuf) -> String {
        if let Some(parent) = file_path.parent() {
            if let Some(dir_name) = parent.file_name() {
                if let Some(dir_str) = dir_name.to_str() {
                    return match dir_str {
                        "src" | "lib" => "Core".to_string(),
                        "api" | "routes" => "API".to_string(),
                        "models" | "entities" => "Domain".to_string(),
                        "services" => "Business".to_string(),
                        "utils" | "helpers" => "Utils".to_string(),
                        "components" => "UI".to_string(),
                        "tests" => "Testing".to_string(),
                        _ => "Application".to_string(),
                    };
                }
            }
        }
        "Application".to_string()
    }

    /// Генерирует слоган для элемента
    fn generate_slogan(&self, element: &ASTElement) -> String {
        match element.element_type {
            crate::parser_ast::ASTElementType::Function => format!("Функция {}", element.name),
            crate::parser_ast::ASTElementType::Method => format!("Метод {}", element.name),
            crate::parser_ast::ASTElementType::Class => format!("Класс {}", element.name),
            crate::parser_ast::ASTElementType::Struct => format!("Структура {}", element.name),
            crate::parser_ast::ASTElementType::Interface => format!("Интерфейс {}", element.name),
            crate::parser_ast::ASTElementType::Enum => format!("Перечисление {}", element.name),
            crate::parser_ast::ASTElementType::Module => format!("Модуль {}", element.name),
            crate::parser_ast::ASTElementType::Constant => format!("Константа {}", element.name),
            crate::parser_ast::ASTElementType::Variable => format!("Переменная {}", element.name),
            _ => format!("Элемент {}", element.name),
        }
    }

    /// Анализирует предупреждения для элемента
    fn analyze_warnings(&self, element: &ASTElement) -> Vec<String> {
        let mut warnings = Vec::new();
        let content_lower = element.content.to_lowercase();

        // Проверка сложности
        if element.complexity > 10 {
            warnings.push(format!("Высокая сложность: {}", element.complexity));
        }

        // Проверка размера
        let lines_count = element.end_line - element.start_line + 1;
        if lines_count > 100 {
            warnings.push(format!("Большой размер: {} строк", lines_count));
        }

        // Проверка документации для публичных элементов
        if !matches!(element.element_type, crate::parser_ast::ASTElementType::Import | crate::parser_ast::ASTElementType::Export)
            && element.visibility == "public" {
            if !content_lower.contains("///") && !content_lower.contains("/**") {
                warnings.push("Публичный элемент без документации".to_string());
            }
        }

        // Проверка на TODO/FIXME
        if content_lower.contains("todo") {
            warnings.push("Содержит TODO".to_string());
        }
        if content_lower.contains("fixme") {
            warnings.push("Содержит FIXME".to_string());
        }

        // Проверка на дублирование кода
        if element.content.len() > 500 && self.has_repeated_patterns(&element.content) {
            warnings.push("Возможное дублирование кода".to_string());
        }

        warnings
    }

    /// Проверяет на повторяющиеся паттерны в коде
    fn has_repeated_patterns(&self, content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        let mut pattern_count = HashMap::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.len() > 20 {
                *pattern_count.entry(trimmed).or_insert(0) += 1;
            }
        }
        
        pattern_count.values().any(|&count| count > 3)
    }

    /// Оптимизирует капсулы
    pub fn optimize_capsules(&self, capsules: &mut Vec<Capsule>) -> Result<()> {
        // Удаляем дублирующиеся капсулы
        self.remove_duplicates(capsules);
        
        // Объединяем мелкие капсулы
        self.merge_small_capsules(capsules)?;
        
        // Сортируем по приоритету
        capsules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }

    /// Удаляет дублирующиеся капсулы
    fn remove_duplicates(&self, capsules: &mut Vec<Capsule>) {
        let mut seen = std::collections::HashSet::new();
        capsules.retain(|capsule| {
            let key = (capsule.name.clone(), capsule.file_path.clone(), capsule.line_start);
            seen.insert(key)
        });
    }

    /// Объединяет мелкие капсулы
    fn merge_small_capsules(&self, _capsules: &mut Vec<Capsule>) -> Result<()> {
        // #ДОДЕЛАТЬ: Реализовать логику объединения мелких капсул
        // Например, объединить функции с complexity < 2 в одну капсулу
        Ok(())
    }
}

impl Default for CapsuleConstructor {
    fn default() -> Self {
        Self::new()
    }
} 