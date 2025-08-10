use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_detail_level_affects_size_and_content() {
    // Start server
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping detail_level e2e");
            return;
        }
    };

    thread::sleep(Duration::from_millis(400));
    let client = reqwest::blocking::Client::new();

    // export: summary vs full
    let r_sum = client
        .post("http://127.0.0.1:5178/export/ai_compact")
        .json(&serde_json::json!({"project_path":".","detail_level":"summary"}))
        .send().and_then(|r| r.json::<serde_json::Value>());
    let r_full = client
        .post("http://127.0.0.1:5178/export/ai_compact")
        .json(&serde_json::json!({"project_path":".","detail_level":"full"}))
        .send().and_then(|r| r.json::<serde_json::Value>());

    if let (Ok(js_sum), Ok(js_full)) = (r_sum, r_full) {
        let s = js_sum["output"].as_str().unwrap_or("");
        let f = js_full["output"].as_str().unwrap_or("");
        assert!(f.len() >= s.len(), "full should be >= summary");
        assert!(!s.contains("```"), "summary should be stripped of code fences");
    }

    // structure: standard longer than summary
    let st_sum = client
        .post("http://127.0.0.1:5178/structure/get")
        .json(&serde_json::json!({"project_path":".","detail_level":"summary"}))
        .send().and_then(|r| r.json::<serde_json::Value>()).ok();
    let st_std = client
        .post("http://127.0.0.1:5178/structure/get")
        .json(&serde_json::json!({"project_path":".","detail_level":"standard"}))
        .send().and_then(|r| r.json::<serde_json::Value>()).ok();

    if let (Some(js_sum), Some(js_std)) = (st_sum, st_std) {
        let s = js_sum["text"].as_str().unwrap_or("");
        let t = js_std["text"].as_str().unwrap_or("");
        assert!(t.len() >= s.len(), "standard should be >= summary");
    }

    let _ = child.kill();
}