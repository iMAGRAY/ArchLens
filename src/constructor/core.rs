use crate::parser_ast::ASTElement;
use crate::types::{Capsule, CapsuleStatus, CapsuleType, Priority, Result};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Core capsule constructor - creates architectural capsules from AST elements
///
/// The `CapsuleConstructor` is responsible for converting Abstract Syntax Tree (AST)
/// elements into architectural capsules that can be analyzed and visualized.
/// It applies various heuristics to determine the significance, type, and properties
/// of code elements.
///
/// # Examples
///
/// ```rust
/// use archlens::constructor::CapsuleConstructor;
/// use std::path::PathBuf;
///
/// let constructor = CapsuleConstructor::new();
/// let capsules = constructor.create_capsules(&ast_elements, &PathBuf::from("src/main.rs"));
/// ```
#[derive(Debug)]
pub struct CapsuleConstructor {
    /// Minimum complexity threshold for capsule creation
    pub min_complexity_threshold: u32,
    /// Maximum allowed capsule size in lines
    pub max_capsule_size: usize,
}

impl CapsuleConstructor {
    /// Creates a new capsule constructor with default settings
    ///
    /// # Default Settings
    /// - `min_complexity_threshold`: 5
    /// - `max_capsule_size`: 1000
    pub fn new() -> Self {
        Self {
            min_complexity_threshold: 5,
            max_capsule_size: 1000,
        }
    }

    /// Creates capsules from a collection of AST elements
    ///
    /// This method processes each AST element and creates corresponding capsules
    /// for elements that meet the significance criteria.
    ///
    /// # Arguments
    ///
    /// * `ast_elements` - Slice of AST elements to process
    /// * `file_path` - Path to the source file being analyzed
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of created capsules, or an error if processing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use archlens::constructor::CapsuleConstructor;
    /// # use std::path::PathBuf;
    /// let constructor = CapsuleConstructor::new();
    /// let capsules = constructor.create_capsules(&ast_elements, &PathBuf::from("src/lib.rs"))?;
    /// ```
    pub fn create_capsules(
        &self,
        ast_elements: &[ASTElement],
        file_path: &Path,
    ) -> Result<Vec<Capsule>> {
        let mut capsules = Vec::new();

        for element in ast_elements {
            if let Some(capsule) = self.create_capsule_from_element(element, file_path)? {
                capsules.push(capsule);
            }
        }

        Ok(capsules)
    }

    /// Creates a capsule from a single AST element
    ///
    /// This method applies various analysis techniques to determine if an AST element
    /// should become a capsule and what its properties should be.
    fn create_capsule_from_element(
        &self,
        element: &ASTElement,
        file_path: &Path,
    ) -> Result<Option<Capsule>> {
        // Filter elements by significance
        if !self.is_significant_element(element) {
            return Ok(None);
        }

        let capsule_type = self.convert_ast_type_to_capsule_type(&element.element_type);
        let priority = self.calculate_priority(element);
        let status = self.determine_status(element);
        let layer = self.determine_layer(file_path);
        let slogan = self.generate_slogan(element);
        let warnings = super::warnings::WarningAnalyzer::analyze_warnings(element);

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
            description: Some(format!(
                "Element {} of type {:?}",
                element.name, element.element_type
            )),
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

    /// Creates a capsule from a node (simplified version)
    ///
    /// This is a simplified method for creating capsules when full AST analysis
    /// is not available or needed.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST element to convert
    /// * `file_path` - Path to the source file
    ///
    /// # Returns
    ///
    /// A `Result` containing the created capsule
    pub fn create_capsule_from_node(
        &self,
        node: &ASTElement,
        file_path: &Path,
    ) -> Result<Capsule> {
        let id = Uuid::new_v4();

        let capsule = Capsule {
            id,
            name: node.name.clone(),
            capsule_type: CapsuleType::Module,
            file_path: file_path.to_path_buf(),
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

    /// Checks if an AST element is significant enough to become a capsule
    ///
    /// Elements are considered significant if they represent important structural
    /// components like functions, classes, modules, or public constants/variables.
    fn is_significant_element(&self, element: &ASTElement) -> bool {
        match element.element_type {
            crate::parser_ast::ASTElementType::Function
            | crate::parser_ast::ASTElementType::Method => true,
            crate::parser_ast::ASTElementType::Class
            | crate::parser_ast::ASTElementType::Struct => true,
            crate::parser_ast::ASTElementType::Interface
            | crate::parser_ast::ASTElementType::Enum => true,
            crate::parser_ast::ASTElementType::Module => true,
            crate::parser_ast::ASTElementType::Constant => element.visibility == "public",
            crate::parser_ast::ASTElementType::Variable => element.visibility == "public",
            crate::parser_ast::ASTElementType::Import
            | crate::parser_ast::ASTElementType::Export => false,
            crate::parser_ast::ASTElementType::Comment => false,
            crate::parser_ast::ASTElementType::Other(_) => false,
        }
    }

    /// Converts AST element type to capsule type
    fn convert_ast_type_to_capsule_type(
        &self,
        ast_type: &crate::parser_ast::ASTElementType,
    ) -> CapsuleType {
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

    /// Calculates priority based on element type and visibility
    fn calculate_priority(&self, element: &ASTElement) -> Priority {
        match element.element_type {
            crate::parser_ast::ASTElementType::Class
            | crate::parser_ast::ASTElementType::Interface => Priority::High,
            crate::parser_ast::ASTElementType::Struct | crate::parser_ast::ASTElementType::Enum => {
                Priority::Medium
            }
            crate::parser_ast::ASTElementType::Function
            | crate::parser_ast::ASTElementType::Method => {
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

    /// Determines element status based on content analysis
    fn determine_status(&self, element: &ASTElement) -> CapsuleStatus {
        let content_lower = element.content.to_lowercase();

        if content_lower.contains("deprecated") || content_lower.contains("@deprecated") {
            CapsuleStatus::Deprecated
        } else if content_lower.contains("todo") || content_lower.contains("fixme") {
            CapsuleStatus::Pending
        } else if content_lower.contains("@experimental") || content_lower.contains("experimental")
        {
            CapsuleStatus::Active
        } else if element.visibility == "private" {
            CapsuleStatus::Hidden
        } else {
            CapsuleStatus::Active
        }
    }

    /// Determines architectural layer based on file path
    fn determine_layer(&self, file_path: &Path) -> String {
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

    /// Generates a human-readable slogan for the element
    fn generate_slogan(&self, element: &ASTElement) -> String {
        match element.element_type {
            crate::parser_ast::ASTElementType::Function => format!("Function {}", element.name),
            crate::parser_ast::ASTElementType::Method => format!("Method {}", element.name),
            crate::parser_ast::ASTElementType::Class => format!("Class {}", element.name),
            crate::parser_ast::ASTElementType::Struct => format!("Struct {}", element.name),
            crate::parser_ast::ASTElementType::Interface => format!("Interface {}", element.name),
            crate::parser_ast::ASTElementType::Enum => format!("Enum {}", element.name),
            crate::parser_ast::ASTElementType::Module => format!("Module {}", element.name),
            crate::parser_ast::ASTElementType::Constant => format!("Constant {}", element.name),
            crate::parser_ast::ASTElementType::Variable => format!("Variable {}", element.name),
            _ => format!("Element {}", element.name),
        }
    }
}

impl Default for CapsuleConstructor {
    fn default() -> Self {
        Self::new()
    }
}
