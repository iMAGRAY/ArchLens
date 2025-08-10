use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_ai_summary_json_and_presets() {
    // Spawn server in background
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
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

    let client = reqwest::blocking::Client::new();

    // /presets/list should return array
    let presets = client.get("http://127.0.0.1:5178/presets/list").send();
    assert!(presets.is_ok(), "presets list should be accessible");
    let arr = presets.unwrap().json::<serde_json::Value>().ok();
    assert!(arr.is_some(), "presets list json");

    // /export/ai_summary_json
    let resp = client
        .post("http://127.0.0.1:5178/export/ai_summary_json")
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
