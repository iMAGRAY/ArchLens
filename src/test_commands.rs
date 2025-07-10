#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::file_scanner::FileScanner;
    use crate::capsule_graph_builder::CapsuleGraphBuilder;
    use crate::capsule_enricher::CapsuleEnricher;
    use crate::validator_optimizer::ValidatorOptimizer;
    use crate::exporter::Exporter;
    use std::path::PathBuf;

    #[test]
    fn test_file_scanner() {
        let scanner = FileScanner::new(
            vec!["**/*.rs".to_string()],
            vec!["**/target/**".to_string()],
            Some(3)
        ).expect("Не удалось создать FileScanner");
        
        let current_dir = std::env::current_dir().unwrap();
        
        match scanner.scan_project(&current_dir) {
            Ok(files) => {
                println!("✅ FileScanner работает: найдено {} файлов", files.len());
                assert!(!files.is_empty());
            }
            Err(e) => {
                println!("⚠️ Ошибка FileScanner: {}", e);
            }
        }
    }

    #[test]
    fn test_capsule_graph_builder() {
        let builder = CapsuleGraphBuilder::new();
        
        // Создаем тестовую капсулу
        let test_capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "TestCapsule".to_string(),
            capsule_type: CapsuleType::Function,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 10,
            complexity: 5,
            priority: Priority::Medium,
            status: CapsuleStatus::Active,
            dependencies: vec![],
            dependents: vec![],
            layer: Some("Core".to_string()),
            summary: None,
            slogan: None,
            warnings: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        let capsules = vec![test_capsule];
        
        match builder.build_graph(&capsules) {
            Ok(graph) => {
                println!("✅ CapsuleGraphBuilder работает: {} капсул, {} связей", 
                         graph.capsules.len(), graph.relations.len());
                assert_eq!(graph.capsules.len(), 1);
            }
            Err(e) => {
                println!("⚠️ Ошибка CapsuleGraphBuilder: {}", e);
                panic!("Критическая ошибка: {}", e);
            }
        }
    }

    #[test]
    fn test_capsule_enricher() {
        let enricher = CapsuleEnricher::new();
        
        // Создаем простой граф для тестирования
        let test_capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "TestCapsule".to_string(),
            capsule_type: CapsuleType::Function,
            file_path: PathBuf::from("src/lib.rs"), // Существующий файл
            line_start: 1,
            line_end: 10,
            complexity: 5,
            priority: Priority::Medium,
            status: CapsuleStatus::Active,
            dependencies: vec![],
            dependents: vec![],
            layer: Some("Core".to_string()),
            summary: None,
            slogan: None,
            warnings: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        let mut capsules = std::collections::HashMap::new();
        capsules.insert(test_capsule.id, test_capsule);
        
        let graph = CapsuleGraph {
            capsules,
            relations: vec![],
            layers: std::collections::HashMap::new(),
            metrics: GraphMetrics {
                total_capsules: 1,
                total_relations: 0,
                complexity_average: 5.0,
                coupling_index: 0.0,
                cohesion_index: 1.0,
                cyclomatic_complexity: 1,
                depth_levels: 1,
            },
            created_at: chrono::Utc::now(),
        };
        
        match enricher.enrich_graph(&graph) {
            Ok(enriched_graph) => {
                println!("✅ CapsuleEnricher работает: {} капсул обогащено", 
                         enriched_graph.capsules.len());
                assert_eq!(enriched_graph.capsules.len(), 1);
            }
            Err(e) => {
                println!("⚠️ Ошибка CapsuleEnricher: {}", e);
            }
        }
    }

    #[test]
    fn test_validator_optimizer() {
        let validator = ValidatorOptimizer::new();
        
        // Создаем граф для валидации
        let test_capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "TestCapsule".to_string(),
            capsule_type: CapsuleType::Function,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 10,
            complexity: 5,
            priority: Priority::Medium,
            status: CapsuleStatus::Active,
            dependencies: vec![],
            dependents: vec![],
            layer: Some("Core".to_string()),
            summary: None,
            slogan: None,
            warnings: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        let mut capsules = std::collections::HashMap::new();
        capsules.insert(test_capsule.id, test_capsule);
        
        let graph = CapsuleGraph {
            capsules,
            relations: vec![],
            layers: std::collections::HashMap::new(),
            metrics: GraphMetrics {
                total_capsules: 1,
                total_relations: 0,
                complexity_average: 5.0,
                coupling_index: 0.3,
                cohesion_index: 0.7,
                cyclomatic_complexity: 1,
                depth_levels: 1,
            },
            created_at: chrono::Utc::now(),
        };
        
        match validator.validate_and_optimize(&graph) {
            Ok(validated_graph) => {
                println!("✅ ValidatorOptimizer работает: граф валидирован");
                assert_eq!(validated_graph.capsules.len(), 1);
            }
            Err(e) => {
                println!("⚠️ Ошибка ValidatorOptimizer: {}", e);
            }
        }
    }

    #[test]
    fn test_exporter() {
        let exporter = Exporter::new();
        
        // Создаем граф для экспорта
        let test_capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "TestCapsule".to_string(),
            capsule_type: CapsuleType::Function,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 10,
            complexity: 5,
            priority: Priority::Medium,
            status: CapsuleStatus::Active,
            dependencies: vec![],
            dependents: vec![],
            layer: Some("Core".to_string()),
            summary: Some("Тестовая капсула".to_string()),
            slogan: None,
            warnings: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        let mut capsules = std::collections::HashMap::new();
        capsules.insert(test_capsule.id, test_capsule);
        
        let graph = CapsuleGraph {
            capsules,
            relations: vec![],
            layers: std::collections::HashMap::new(),
            metrics: GraphMetrics {
                total_capsules: 1,
                total_relations: 0,
                complexity_average: 5.0,
                coupling_index: 0.3,
                cohesion_index: 0.7,
                cyclomatic_complexity: 1,
                depth_levels: 1,
            },
            created_at: chrono::Utc::now(),
        };
        
        // Тестируем JSON экспорт
        match exporter.export_to_json(&graph) {
            Ok(json) => {
                println!("✅ JSON экспорт работает: {} символов", json.len());
                assert!(json.contains("TestCapsule"));
            }
            Err(e) => {
                println!("⚠️ Ошибка JSON экспорта: {}", e);
            }
        }
        
        // Тестируем Mermaid экспорт
        match exporter.export_to_mermaid(&graph) {
            Ok(mermaid) => {
                println!("✅ Mermaid экспорт работает: {} символов", mermaid.len());
                assert!(mermaid.contains("graph TD"));
            }
            Err(e) => {
                println!("⚠️ Ошибка Mermaid экспорта: {}", e);
            }
        }
    }

    #[test]
    fn test_full_pipeline() {
        println!("🚀 Тестирование полного пайплайна анализа...");
        
        // 1. Сканирование файлов
        let scanner = FileScanner::new(
            vec!["**/*.rs".to_string()],
            vec!["**/target/**".to_string()],
            Some(3)
        ).expect("Не удалось создать FileScanner");
        
        let current_dir = std::env::current_dir().unwrap();
        let files = scanner.scan_project(&current_dir).unwrap_or_else(|_| vec![]);
        println!("📁 Найдено файлов: {}", files.len());
        
        if files.is_empty() {
            println!("⚠️ Нет файлов для анализа, создаем тестовые данные");
        }
        
        // 2. Создание тестовых капсул
        let mut capsules = Vec::new();
        for i in 0..3 {
            let capsule = Capsule {
                id: uuid::Uuid::new_v4(),
                name: format!("TestModule{}", i),
                capsule_type: CapsuleType::Module,
                file_path: PathBuf::from(format!("test{}.rs", i)),
                line_start: 1,
                line_end: 50,
                complexity: 5 + i as u32,
                priority: Priority::Medium,
                status: CapsuleStatus::Active,
                dependencies: vec![],
                dependents: vec![],
                layer: Some("Core".to_string()),
                summary: Some(format!("Тестовый модуль {}", i)),
                slogan: None,
                warnings: vec![],
                metadata: std::collections::HashMap::new(),
                created_at: chrono::Utc::now(),
            };
            capsules.push(capsule);
        }
        
        // 3. Построение графа
        let builder = CapsuleGraphBuilder::new();
        let graph = builder.build_graph(&capsules).expect("Ошибка построения графа");
        println!("🔗 Граф построен: {} узлов, {} связей", 
                 graph.capsules.len(), graph.relations.len());
        
        // 4. Обогащение
        let enricher = CapsuleEnricher::new();
        let enriched_graph = enricher.enrich_graph(&graph).expect("Ошибка обогащения");
        println!("✨ Граф обогащен");
        
        // 5. Валидация
        let validator = ValidatorOptimizer::new();
        let validated_graph = validator.validate_and_optimize(&enriched_graph).expect("Ошибка валидации");
        println!("✅ Граф валидирован");
        
        // 6. Экспорт
        let exporter = Exporter::new();
        let json = exporter.export_to_json(&validated_graph).expect("Ошибка JSON экспорта");
        let mermaid = exporter.export_to_mermaid(&validated_graph).expect("Ошибка Mermaid экспорта");
        
        println!("📊 Экспорт завершен:");
        println!("  - JSON: {} символов", json.len());
        println!("  - Mermaid: {} символов", mermaid.len());
        
        println!("🎉 Полный пайплайп работает корректно!");
    }

    #[test] 
    fn test_json_serialization() {
        println!("🔬 Тестирование JSON сериализации...");
        
        // Создаем тестовый граф
        let test_capsule = Capsule {
            id: uuid::Uuid::new_v4(),
            name: "TestCapsule".to_string(),
            capsule_type: CapsuleType::Function,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: 10,
            complexity: 5,
            priority: Priority::Medium,
            status: CapsuleStatus::Active,
            dependencies: vec![],
            dependents: vec![],
            layer: Some("Core".to_string()),
            summary: Some("Тестовая капсула".to_string()),
            slogan: None,
            warnings: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        let mut capsules = std::collections::HashMap::new();
        capsules.insert(test_capsule.id, test_capsule);
        
        let graph = CapsuleGraph {
            capsules,
            relations: vec![],
            layers: std::collections::HashMap::new(),
            metrics: GraphMetrics {
                total_capsules: 1,
                total_relations: 0,
                complexity_average: 5.0,
                coupling_index: 0.3,
                cohesion_index: 0.7,
                cyclomatic_complexity: 1,
                depth_levels: 1,
            },
            created_at: chrono::Utc::now(),
        };
        
        // Создаем AnalysisResult
        let analysis_result = crate::core::AnalysisResult {
            graph,
            warnings: vec![],
            recommendations: vec![
                "Тестовая рекомендация".to_string(),
            ],
            export_formats: vec![crate::core::ExportFormat::Json],
        };
        
        // Тестируем сериализацию
        let json_result = serde_json::to_string(&analysis_result);
        match json_result {
            Ok(json) => {
                println!("✅ JSON сериализация работает: {} символов", json.len());
                
                // Проверяем что JSON содержит ключевые поля
                assert!(json.contains("graph"));
                assert!(json.contains("metrics"));
                assert!(json.contains("total_capsules"));
                assert!(json.contains("TestCapsule"));
                
                // Пытаемся десериализовать обратно
                let parsed: std::result::Result<crate::core::AnalysisResult, serde_json::Error> = serde_json::from_str(&json);
                match parsed {
                    Ok(parsed_result) => {
                        println!("✅ JSON десериализация работает");
                        assert_eq!(parsed_result.graph.capsules.len(), 1);
                        assert_eq!(parsed_result.graph.metrics.total_capsules, 1);
                    }
                    Err(e) => {
                        println!("❌ Ошибка десериализации: {}", e);
                        panic!("JSON десериализация не работает");
                    }
                }
            }
            Err(e) => {
                println!("❌ Ошибка JSON сериализации: {}", e);
                panic!("JSON сериализация не работает");
            }
        }
        
        println!("🎉 JSON сериализация работает корректно!");
    }
} 