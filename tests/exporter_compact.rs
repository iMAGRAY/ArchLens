use archlens::exporter::Exporter;
use archlens::types::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

fn build_test_graph() -> CapsuleGraph {
    // Create three capsules: A, B, Hub
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let id_hub = Uuid::new_v4();

    let cap_a = Capsule {
        id: id_a,
        name: "A".to_string(),
        capsule_type: CapsuleType::Module,
        file_path: std::path::PathBuf::from("/tmp/a.rs"),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 12,
        dependencies: vec![id_b],
        layer: Some("Core".to_string()),
        summary: None,
        description: None,
        warnings: vec![],
        status: CapsuleStatus::Active,
        priority: Priority::Medium,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.5,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let cap_b = Capsule {
        id: id_b,
        name: "B".to_string(),
        capsule_type: CapsuleType::Module,
        file_path: std::path::PathBuf::from("/tmp/b.rs"),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 7,
        dependencies: vec![id_a],
        layer: Some("Core".to_string()),
        summary: None,
        description: None,
        warnings: vec![],
        status: CapsuleStatus::Active,
        priority: Priority::Medium,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.6,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let cap_hub = Capsule {
        id: id_hub,
        name: "Hub".to_string(),
        capsule_type: CapsuleType::Module,
        file_path: std::path::PathBuf::from("/tmp/hub.rs"),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 3,
        dependencies: vec![id_a, id_b],
        layer: Some("Core".to_string()),
        summary: None,
        description: None,
        warnings: vec![],
        status: CapsuleStatus::Active,
        priority: Priority::Low,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.7,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let mut capsules = HashMap::new();
    capsules.insert(id_a, cap_a);
    capsules.insert(id_b, cap_b);
    capsules.insert(id_hub, cap_hub);

    let relations = vec![
        CapsuleRelation {
            from_id: id_a,
            to_id: id_b,
            relation_type: RelationType::Depends,
            strength: 0.8,
            description: Some("A->B".into()),
        },
        CapsuleRelation {
            from_id: id_b,
            to_id: id_a,
            relation_type: RelationType::Depends,
            strength: 0.8,
            description: Some("B->A".into()),
        },
        CapsuleRelation {
            from_id: id_hub,
            to_id: id_a,
            relation_type: RelationType::Depends,
            strength: 0.9,
            description: Some("Hub->A".into()),
        },
        CapsuleRelation {
            from_id: id_hub,
            to_id: id_b,
            relation_type: RelationType::Depends,
            strength: 0.9,
            description: Some("Hub->B".into()),
        },
    ];

    let mut layers = HashMap::new();
    layers.insert("Core".to_string(), vec![id_a, id_b, id_hub]);

    let metrics = GraphMetrics {
        total_capsules: 3,
        total_relations: relations.len(),
        complexity_average: (12 + 7 + 3) as f32 / 3.0,
        coupling_index: 0.75,
        cohesion_index: 0.25,
        cyclomatic_complexity: 6,
        depth_levels: 2,
    };

    CapsuleGraph {
        capsules,
        relations,
        layers,
        metrics,
        created_at: Utc::now(),
        previous_analysis: None,
    }
}

#[test]
fn test_ai_compact_contains_sections() {
    let graph = build_test_graph();
    let exporter = Exporter::new();
    let out = exporter.export_to_ai_compact(&graph).expect("export ok");
    assert!(out.contains("# AI Compact Analysis"));
    assert!(out.contains("## Summary"));
    assert!(out.contains("## Problems (Heuristic)"));
    let has_any_warnings = graph.capsules.values().any(|c| !c.warnings.is_empty());
    if has_any_warnings {
        assert!(out.contains("## Problems (Validated)"));
    }
    assert!(out.contains("## Cycles (Top)"));
    assert!(out.contains("## Top Coupling"));
    assert!(out.contains("## Top Complexity Components"));
}

#[test]
fn test_cycles_render_path() {
    let graph = build_test_graph();
    let exporter = Exporter::new();
    let out = exporter.export_to_ai_compact(&graph).expect("export ok");
    // Cycle A -> B -> A should be present
    assert!(out.contains("A -> B -> A") || out.contains("B -> A -> B"));
}

#[test]
fn test_mermaid_export() {
    let graph = build_test_graph();
    let exporter = Exporter::new();
    let out = exporter.export_to_mermaid(&graph).expect("mermaid ok");
    assert!(out.starts_with("graph TD"));
}
