#[cfg(test)]
mod tests {
    use crate::types::*;
    use std::path::PathBuf;

    #[test]
    fn test_basic_types() {
        // Test basic types creation
        let _capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            file_path: PathBuf::from("test.rs"),
            capsule_type: CapsuleType::Module,
            layer: Some("core".to_string()),
            size: 100,
            complexity: 5,
            line_start: 1,
            line_end: 100,
            status: CapsuleStatus::Active,
            dependencies: Vec::new(),
            description: Some("Test capsule".to_string()),
            priority: Priority::Medium,
            tags: Vec::new(),
            quality_score: 75.0,
            slogan: None,
            dependents: Vec::new(),
            metadata: std::collections::HashMap::new(),
            warnings: Vec::new(),
            summary: Some("Test summary".to_string()),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_export_formats() {
        // Test that export formats are recognized
        let formats = vec![ExportFormat::JSON, ExportFormat::YAML];

        assert_eq!(formats.len(), 2);
    }

    #[test]
    fn test_priority_ordering() {
        // Test priority ordering (Critical=0, High=1, Medium=2, Low=3)
        assert!((Priority::Critical as u8) < (Priority::High as u8));
        assert!((Priority::High as u8) < (Priority::Medium as u8));
        assert!((Priority::Medium as u8) < (Priority::Low as u8));
    }

    #[test]
    fn test_capsule_types() {
        // Test capsule types
        let types = vec![
            CapsuleType::Module,
            CapsuleType::Function,
            CapsuleType::Class,
            CapsuleType::Interface,
            CapsuleType::Struct,
            CapsuleType::Enum,
            CapsuleType::Constant,
            CapsuleType::Variable,
        ];

        assert_eq!(types.len(), 8);
    }
}
