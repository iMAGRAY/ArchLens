use archlens::exporter::Exporter;
use archlens::types::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

fn build_small_graph() -> CapsuleGraph {
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let cap_a = Capsule {
        id: id_a,
        name: "A".into(),
        capsule_type: CapsuleType::Module,
        file_path: "/tmp/a.rs".into(),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 5,
        dependencies: vec![id_b],
        layer: Some("Core".into()),
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
        name: "B".into(),
        capsule_type: CapsuleType::Module,
        file_path: "/tmp/b.rs".into(),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 7,
        dependencies: vec![id_a],
        layer: Some("Core".into()),
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
    let mut capsules = HashMap::new();
    capsules.insert(id_a, cap_a);
    capsules.insert(id_b, cap_b);
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
    ];
    let mut layers = HashMap::new();
    layers.insert("Core".to_string(), vec![id_a, id_b]);
    let metrics = GraphMetrics {
        total_capsules: 2,
        total_relations: relations.len(),
        complexity_average: (5 + 7) as f32 / 2.0,
        coupling_index: 0.75,
        cohesion_index: 0.25,
        cyclomatic_complexity: 4,
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
fn snapshot_ai_summary_json_matches_golden_ignoring_cycles() {
    let g = build_small_graph();
    let exporter = Exporter::new();
    let actual = exporter.export_to_ai_summary_json(&g).expect("ok");

    // Load golden
    let golden_text =
        std::fs::read_to_string("tests/golden/ai_summary_small.json").expect("read golden");
    let mut golden: serde_json::Value = serde_json::from_str(&golden_text).expect("parse golden");

    // Normalize: remove cycles_top for stability
    let mut actual_norm = actual.clone();
    if let Some(obj) = actual_norm.as_object_mut() {
        obj.remove("cycles_top");
    }
    if let Some(obj) = golden.as_object_mut() {
        obj.remove("cycles_top");
    }

    assert_eq!(
        actual_norm, golden,
        "summary_json should match golden (ignoring cycles_top)\nactual: {}\nexpected: {}",
        actual_norm, golden
    );
}
