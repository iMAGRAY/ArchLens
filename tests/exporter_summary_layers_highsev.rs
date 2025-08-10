use archlens::exporter::Exporter;
use archlens::types::*;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

fn build_graph_layers_highsev() -> CapsuleGraph {
    // Four modules: A,B,C,D with specific complexities and layers
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();
    let id_c = Uuid::new_v4();
    let id_d = Uuid::new_v4();

    let cap_a = Capsule {
        id: id_a,
        name: "A".into(),
        capsule_type: CapsuleType::Module,
        file_path: "/tmp/a.rs".into(),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 7,
        dependencies: vec![id_b, id_c, id_d],
        layer: Some("Core".into()),
        summary: None,
        description: None,
        warnings: vec![AnalysisWarning{ message: "High complexity".into(), level: Priority::High, category: "complexity".into(), capsule_id: Some(id_a), suggestion: Some("reduce complexity".into()) }],
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
        complexity: 5,
        dependencies: vec![id_c, id_d],
        layer: Some("Infra".into()),
        summary: None,
        description: None,
        warnings: vec![AnalysisWarning{ message: "Tight coupling".into(), level: Priority::High, category: "coupling".into(), capsule_id: Some(id_b), suggestion: Some("decouple".into()) }],
        status: CapsuleStatus::Active,
        priority: Priority::Medium,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.6,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let cap_c = Capsule {
        id: id_c,
        name: "C".into(),
        capsule_type: CapsuleType::Module,
        file_path: "/tmp/c.rs".into(),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 9,
        dependencies: vec![id_d],
        layer: Some("Core".into()),
        summary: None,
        description: None,
        warnings: vec![AnalysisWarning{ message: "Complexity rising".into(), level: Priority::Medium, category: "complexity".into(), capsule_id: Some(id_c), suggestion: None }],
        status: CapsuleStatus::Active,
        priority: Priority::Medium,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.7,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let cap_d = Capsule {
        id: id_d,
        name: "D".into(),
        capsule_type: CapsuleType::Module,
        file_path: "/tmp/d.rs".into(),
        line_start: 1,
        line_end: 10,
        size: 10,
        complexity: 3,
        dependencies: vec![id_a, id_b],
        layer: Some("Core".into()),
        summary: None,
        description: None,
        warnings: vec![],
        status: CapsuleStatus::Active,
        priority: Priority::Low,
        tags: vec![],
        metadata: HashMap::new(),
        quality_score: 0.8,
        slogan: None,
        dependents: vec![],
        created_at: Some(Utc::now().to_rfc3339()),
    };

    let mut capsules = HashMap::new();
    capsules.insert(id_a, cap_a);
    capsules.insert(id_b, cap_b);
    capsules.insert(id_c, cap_c);
    capsules.insert(id_d, cap_d);

    let relations = vec![
        CapsuleRelation { from_id: id_a, to_id: id_b, relation_type: RelationType::Depends, strength: 0.9, description: Some("A->B".into()) },
        CapsuleRelation { from_id: id_a, to_id: id_c, relation_type: RelationType::Depends, strength: 0.8, description: Some("A->C".into()) },
        CapsuleRelation { from_id: id_a, to_id: id_d, relation_type: RelationType::Depends, strength: 0.7, description: Some("A->D".into()) },
        CapsuleRelation { from_id: id_b, to_id: id_c, relation_type: RelationType::Depends, strength: 0.6, description: Some("B->C".into()) },
        CapsuleRelation { from_id: id_c, to_id: id_d, relation_type: RelationType::Depends, strength: 0.5, description: Some("C->D".into()) },
        CapsuleRelation { from_id: id_d, to_id: id_a, relation_type: RelationType::Depends, strength: 0.9, description: Some("D->A".into()) },
        CapsuleRelation { from_id: id_d, to_id: id_b, relation_type: RelationType::Depends, strength: 0.9, description: Some("D->B".into()) },
    ];

    let mut layers = HashMap::new();
    layers.insert("Core".to_string(), vec![id_a, id_c, id_d]);
    layers.insert("Infra".to_string(), vec![id_b]);

    let metrics = GraphMetrics {
        total_capsules: 4,
        total_relations: relations.len(),
        complexity_average: (7 + 5 + 9 + 3) as f32 / 4.0,
        coupling_index: 0.6,
        cohesion_index: 0.4,
        cyclomatic_complexity: 7,
        depth_levels: 3,
    };

    CapsuleGraph { capsules, relations, layers, metrics, created_at: Utc::now(), previous_analysis: None }
}

fn normalize(mut v: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = v.as_object_mut() { obj.remove("cycles_top"); }
    if let Some(summary) = v.get_mut("summary").and_then(|s| s.as_object_mut()) {
        if let Some(layers) = summary.get_mut("layers").and_then(|l| l.as_array_mut()) {
            layers.sort_by(|a, b| a.get("name").and_then(|x| x.as_str()).cmp(&b.get("name").and_then(|x| x.as_str())));
        }
    }
    if let Some(arr) = v.get_mut("problems_validated").and_then(|x| x.as_array_mut()) {
        for e in arr.iter_mut() {
            if let Some(tc) = e.get_mut("top_components").and_then(|x| x.as_array_mut()) {
                tc.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
            }
        }
        arr.sort_by(|a, b| a.get("category").and_then(|x| x.as_str()).cmp(&b.get("category").and_then(|x| x.as_str())));
    }
    if let Some(arr) = v.get_mut("top_coupling").and_then(|x| x.as_array_mut()) {
        arr.sort_by(|a, b| a.get("component").and_then(|x| x.as_str()).cmp(&b.get("component").and_then(|x| x.as_str())));
    }
    v
}

#[test]
fn snapshot_ai_summary_json_layers_highsev_matches_golden_norm() {
    let g = build_graph_layers_highsev();
    let exporter = Exporter::new();
    let actual = exporter.export_to_ai_summary_json(&g).expect("ok");
    let actual_norm = normalize(actual);

    let golden_text = std::fs::read_to_string("tests/golden/ai_summary_layers_highsev.json").expect("read golden");
    let golden: serde_json::Value = serde_json::from_str(&golden_text).expect("parse golden");
    let golden_norm = normalize(golden);

    assert_eq!(actual_norm, golden_norm, "summary_json should match golden (normalized)\nactual: {}\nexpected: {}", actual_norm, golden_norm);
}