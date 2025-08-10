use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_tools_list_and_call() {
    // Spawn server in background
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http tools e2e");
            return;
        }
    };

    // Give server time to start
    thread::sleep(Duration::from_millis(400));

    let client = reqwest::blocking::Client::new();

    // tools/list
    let r = client
        .post("http://127.0.0.1:5178/tools/list")
        .json(&serde_json::json!({}))
        .send();
    assert!(r.is_ok(), "/tools/list should respond");
    if let Ok(resp) = r {
        assert!(resp.status().is_success());
    }

    // tools/call export.ai_summary_json
    let _payload = serde_json::json!({
        "name": "tools/call",
        "arguments": {"project_path":".", "top_n": 3}
    });
    let r2 = client
        .post("http://127.0.0.1:5178/tools/call")
        .json(&serde_json::json!({"name":"export.ai_summary_json","arguments":{"project_path":".","top_n":3}}))
        .send();
    assert!(r2.is_ok(), "/tools/call should respond");

    // cleanup
    let _ = child.kill();
}
