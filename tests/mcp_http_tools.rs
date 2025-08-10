use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_tools_list_and_call() {
    // Spawn server in background
    let port = 5194u16;
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
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

    // tools/list
    let r = client
        .post(&format!("http://127.0.0.1:{}/tools/list", port))
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
        .post(&format!("http://127.0.0.1:{}/tools/call", port))
        .json(&serde_json::json!({"name":"export.ai_summary_json","arguments":{"project_path":".","top_n":3}}))
        .send();
    assert!(r2.is_ok(), "/tools/call should respond");

    // cleanup
    let _ = child.kill();
}
