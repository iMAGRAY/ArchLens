use std::path::PathBuf;
use crate::types::{
    Capsule, CapsuleType, CapsuleStatus, Priority, Result, AnalysisError, AnalysisWarning
};
use crate::parser_ast::ASTElement;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

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
            size: element.end_line - element.start_line + 1,
            complexity: element.complexity,
            dependencies: vec![],
            layer: Some(layer.clone()),
            summary: None,
            description: Some(format!("Элемент {} типа {:?}", element.name, element.element_type)),
            warnings,
            status,
            priority,
            tags: vec![layer.to_lowercase()],
            metadata: element.metadata.clone(),
            quality_score: if element.complexity > 10 { 0.5 } else { 0.8 },
            slogan: Some(slogan),
            dependents: vec![],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        Ok(Some(capsule))
    }

    pub fn create_capsule_from_node(&self, node: &ASTElement, file_path: &PathBuf) -> Result<Capsule> {
        let id = Uuid::new_v4();
        
        let capsule = Capsule {
            id,
            name: node.name.clone(),
            capsule_type: CapsuleType::Module,  // Упрощаем
            file_path: file_path.clone(),
            line_start: node.start_line,
            line_end: node.end_line,
            size: node.end_line - node.start_line,
            complexity: node.complexity,
            dependencies: Vec::new(),
            layer: Some("default".to_string()),
            summary: None,
            description: None,
            warnings: Vec::new(),
            status: CapsuleStatus::Pending,
            priority: Priority::Medium,
            tags: Vec::new(),
            metadata: HashMap::new(),
            quality_score: 0.0,
            slogan: None,
            dependents: Vec::new(),
            created_at: None,
        };
        
        Ok(capsule)
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
            CapsuleStatus::Pending
        } else if content_lower.contains("@experimental") || content_lower.contains("experimental") {
            CapsuleStatus::Active
        } else if element.visibility == "private" {
            CapsuleStatus::Hidden
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
                        "api" | "controllers" | "routes" => "API".to_string(),
                        "ui" | "components" | "views" => "UI".to_string(),
                        "utils" | "helpers" | "tools" => "Utils".to_string(),
                        "models" | "entities" | "domain" => "Business".to_string(),
                        "services" | "business" => "Business".to_string(),
                        "data" | "database" | "db" => "Data".to_string(),
                        "tests" | "test" => "Tests".to_string(),
                        _ => "Other".to_string(),
                    };
                }
            }
        }
        "Core".to_string()
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
    fn analyze_warnings(&self, element: &ASTElement) -> Vec<AnalysisWarning> {
        let mut warnings = Vec::new();
        let content_lower = element.content.to_lowercase();

        // Проверка сложности
        if element.complexity > 10 {
            warnings.push(AnalysisWarning {
                level: Priority::High,
                message: format!("Высокая сложность: {}", element.complexity),
                category: "complexity".to_string(),
                capsule_id: None,
                suggestion: Some("Разбейте на более мелкие функции".to_string()),
            });
        }

        // Проверка размера
        let lines_count = element.end_line - element.start_line + 1;
        if lines_count > 100 {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: format!("Большой размер: {lines_count} строк"),
                category: "size".to_string(),
                capsule_id: None,
                suggestion: Some("Рассмотрите разбиение на несколько модулей".to_string()),
            });
        }

        // Проверка документации для публичных элементов
        if !matches!(element.element_type, crate::parser_ast::ASTElementType::Import | crate::parser_ast::ASTElementType::Export)
            && element.visibility == "public"
            && !content_lower.contains("///") && !content_lower.contains("/**") {
                warnings.push(AnalysisWarning {
                    level: Priority::Low,
                    message: "Публичный элемент без документации".to_string(),
                    category: "documentation".to_string(),
                    capsule_id: None,
                    suggestion: Some("Добавьте документацию к публичным интерфейсам".to_string()),
                });
            }

        // Проверка на TODO/FIXME
        if content_lower.contains("todo") {
            warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: "Содержит TODO".to_string(),
                category: "maintenance".to_string(),
                capsule_id: None,
                suggestion: Some("Завершите или запланируйте выполнение TODO".to_string()),
            });
        }
        if content_lower.contains("fixme") {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: "Содержит FIXME".to_string(),
                category: "maintenance".to_string(),
                capsule_id: None,
                suggestion: Some("Исправьте указанные проблемы".to_string()),
            });
        }

        // Проверка на дублирование кода
        if element.content.len() > 500 && self.has_repeated_patterns(&element.content) {
            warnings.push(AnalysisWarning {
                level: Priority::Medium,
                message: "Возможное дублирование кода".to_string(),
                category: "duplication".to_string(),
                capsule_id: None,
                suggestion: Some("Выделите общую логику в отдельные методы".to_string()),
            });
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

    /// Объединяет мелкие капсулы в более крупные
    fn merge_small_capsules(&self, capsules: &mut [Capsule]) -> Result<()> {
        let mut merged_indices = std::collections::HashSet::new();
        let mut merge_groups: Vec<Vec<usize>> = Vec::new();
        
        // Группируем мелкие капсулы по критериям объединения
        for i in 0..capsules.len() {
            if merged_indices.contains(&i) {
                continue;
            }
            
            let capsule = &capsules[i];
            
            // Проверяем критерии для объединения
            if self.should_merge_capsule(capsule) {
                let mut group = vec![i];
                merged_indices.insert(i);
                
                // Ищем похожие капсулы для объединения
                for j in (i + 1)..capsules.len() {
                    if merged_indices.contains(&j) {
                        continue;
                    }
                    
                    let other_capsule = &capsules[j];
                    
                    if self.should_merge_capsule(other_capsule) && 
                       self.can_merge_capsules(capsule, other_capsule) {
                        group.push(j);
                        merged_indices.insert(j);
                    }
                }
                
                // Если группа содержит более одной капсулы, добавляем её для объединения
                if group.len() > 1 {
                    merge_groups.push(group);
                }
            }
        }
        
        // Выполняем объединение групп (в обратном порядке индексов)
        for group in merge_groups.into_iter().rev() {
            if group.len() > 1 {
                self.merge_capsule_group(capsules, group)?;
            }
        }
        
        Ok(())
    }
    
    /// Проверяет, следует ли объединить капсулу
    fn should_merge_capsule(&self, capsule: &Capsule) -> bool {
        let size = capsule.line_end - capsule.line_start + 1;
        
        // Критерии для объединения:
        // - Размер < 10 строк
        // - Низкая сложность
        // - Утилитарные типы
        size < 10 || 
        capsule.complexity < 3 ||
        matches!(capsule.capsule_type, CapsuleType::Constant | CapsuleType::Variable | CapsuleType::Import | CapsuleType::Export)
    }
    
    /// Проверяет, можно ли объединить две капсулы
    fn can_merge_capsules(&self, capsule1: &Capsule, capsule2: &Capsule) -> bool {
        // Должны быть в одном файле
        if capsule1.file_path != capsule2.file_path {
            return false;
        }
        
        // Должны быть в одном слое
        if capsule1.layer != capsule2.layer {
            return false;
        }
        
        // Должны быть совместимых типов
        self.are_compatible_types(&capsule1.capsule_type, &capsule2.capsule_type)
    }
    
    /// Проверяет совместимость типов капсул
    fn are_compatible_types(&self, type1: &CapsuleType, type2: &CapsuleType) -> bool {
        match (type1, type2) {
            // Константы и переменные можно объединять
            (CapsuleType::Constant, CapsuleType::Constant) => true,
            (CapsuleType::Variable, CapsuleType::Variable) => true,
            (CapsuleType::Constant, CapsuleType::Variable) => true,
            (CapsuleType::Variable, CapsuleType::Constant) => true,
            
            // Импорты и экспорты можно объединять
            (CapsuleType::Import, CapsuleType::Import) => true,
            (CapsuleType::Export, CapsuleType::Export) => true,
            (CapsuleType::Import, CapsuleType::Export) => true,
            (CapsuleType::Export, CapsuleType::Import) => true,
            
            // Функции и методы можно объединять если они мелкие
            (CapsuleType::Function, CapsuleType::Function) => true,
            (CapsuleType::Method, CapsuleType::Method) => true,
            (CapsuleType::Function, CapsuleType::Method) => true,
            (CapsuleType::Method, CapsuleType::Function) => true,
            
            _ => false,
        }
    }
    
    /// Объединяет группу капсул в одну
    fn merge_capsule_group(&self, capsules: &mut [Capsule], group: Vec<usize>) -> Result<()> {
        if group.len() < 2 {
            return Ok(());
        }
        
        // Сортируем индексы для стабильности
        let mut sorted_group = group;
        sorted_group.sort();
        
        // Создаем объединенную капсулу на основе первой
        let main_index = sorted_group[0];
        let mut merged_capsule = capsules[main_index].clone();
        
        // Объединяем данные из остальных капсул
        let mut merged_names = vec![merged_capsule.name.clone()];
        let mut merged_warnings = merged_capsule.warnings.clone();
        let mut total_complexity = merged_capsule.complexity;
        let mut min_line = merged_capsule.line_start;
        let mut max_line = merged_capsule.line_end;
        
        for &index in &sorted_group[1..] {
            let capsule = &capsules[index];
            merged_names.push(capsule.name.clone());
            merged_warnings.extend(capsule.warnings.clone());
            total_complexity += capsule.complexity;
            min_line = min_line.min(capsule.line_start);
            max_line = max_line.max(capsule.line_end);
        }
        
        // Обновляем объединенную капсулу
        merged_capsule.name = format!("merged_{}", merged_names.join("_"));
        merged_capsule.slogan = Some(format!("Объединено: {}", merged_names.join(", ")));
        merged_capsule.warnings = merged_warnings;
        merged_capsule.complexity = total_complexity;
        merged_capsule.line_start = min_line;
        merged_capsule.line_end = max_line;
        
        // Обновляем приоритет исходя из новой сложности
        merged_capsule.priority = if total_complexity > 15 { 
            Priority::High 
        } else if total_complexity > 8 { 
            Priority::Medium 
        } else { 
            Priority::Low 
        };
        
        // Записываем объединенную капсулу
        capsules[main_index] = merged_capsule;
        
        // Помечаем остальные капсулы как объединенные (можно удалить позже)
        for &index in &sorted_group[1..] {
            capsules[index].status = CapsuleStatus::Deprecated;
            capsules[index].warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: "Объединено в другую капсулу".to_string(),
                category: "optimization".to_string(),
                capsule_id: Some(capsules[index].id),
                suggestion: Some("Капсула была автоматически объединена для оптимизации".to_string()),
            });
        }
        
        Ok(())
    }
}

impl Default for CapsuleConstructor {
    fn default() -> Self {
        Self::new()
    }
} 