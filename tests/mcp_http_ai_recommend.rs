use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_ai_recommend_with_summary() {
    // Spawn server in background
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http ai.recommend e2e");
            return;
        }
    };

    thread::sleep(Duration::from_millis(400));

    let client = reqwest::blocking::Client::new();

    // First, get ai_summary_json
    let sum_resp = client
        .post("http://127.0.0.1:5178/export/ai_summary_json")
        .json(&serde_json::json!({"project_path":".","top_n":3}))
        .send();
    assert!(sum_resp.is_ok(), "summary POST should succeed");
    let sum_json = sum_resp
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap_or(serde_json::json!({}));
    let summary = sum_json
        .get("json")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Then, call recommend with that summary
    let rec_resp = client
        .post("http://127.0.0.1:5178/ai/recommend")
        .json(&serde_json::json!({"project_path":".","json": summary}))
        .send();
    assert!(rec_resp.is_ok(), "recommend POST should succeed");
    let rec_json = rec_resp
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap_or(serde_json::json!({}));

    assert_eq!(
        rec_json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "ok",
        "status ok"
    );
    let recs = rec_json
        .get("recommendations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !recs.is_empty(),
        "should return at least one recommendation"
    );

    let _ = child.kill();
}
