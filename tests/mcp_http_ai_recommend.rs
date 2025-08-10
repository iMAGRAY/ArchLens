use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_ai_recommend_with_summary() {
    // Spawn server in background
    let port = 5191u16; // unique port for this test to avoid conflicts
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
        .env("ARCHLENS_TIMEOUT_MS", "240000")
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

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(240_000))
        .build()
        .expect("client");

    // Wait until server is ready
    for _ in 0..20 {
        if let Ok(resp) = client
            .get(&format!("http://127.0.0.1:{}/schemas/list", port))
            .send()
        {
            if resp.status().is_success() {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    // First, get ai_summary_json
    let sum_resp = client
        .post(&format!(
            "http://127.0.0.1:{}/export/ai_summary_json",
            port
        ))
        .json(&serde_json::json!({"project_path":"src","top_n":3}))
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
        .post(&format!("http://127.0.0.1:{}/ai/recommend", port))
        .json(&serde_json::json!({"project_path":"src","json": summary}))
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
