use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_ai_summary_json_and_presets() {
    // Spawn server in background
    let port = 5198u16;
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http ai_summary_json e2e");
            return;
        }
    };

    thread::sleep(Duration::from_millis(400));

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(120_000))
        .build()
        .expect("client");

    // /presets/list should return array
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
    let presets = client
        .get(&format!("http://127.0.0.1:{}/presets/list", port))
        .send();
    assert!(presets.is_ok(), "presets list should be accessible");
    let arr = presets.unwrap().json::<serde_json::Value>().ok();
    assert!(arr.is_some(), "presets list json");

    // /export/ai_summary_json
    let resp = client
        .post(&format!("http://127.0.0.1:{}/export/ai_summary_json", port))
        .json(&serde_json::json!({"project_path":".","top_n":3,"max_output_chars":20000}))
        .send();
    assert!(resp.is_ok(), "ai_summary_json POST should succeed");
    let payload = resp
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap_or(serde_json::json!({}));
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        "ok"
    );
    let js = payload
        .get("json")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    assert!(js.get("summary").is_some(), "json summary present");

    let _ = child.kill();
}
