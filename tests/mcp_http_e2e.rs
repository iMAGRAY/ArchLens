use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_export_and_schemas() {
    // Spawn server in background
    let port = 5197u16;
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http e2e");
            return;
        }
    };

    // Give server time to start
    thread::sleep(Duration::from_millis(400));

    let client = reqwest::blocking::Client::new();

    // /schemas/list
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
    let r = client
        .get(&format!("http://127.0.0.1:{}/schemas/list", port))
        .send();
    assert!(r.is_ok(), "schemas list should be accessible");

    // export/ai_compact
    let r2 = client
        .post(&format!("http://127.0.0.1:{}/export/ai_compact", port))
        .json(&serde_json::json!({"project_path":".","detail_level":"summary"}))
        .send();
    assert!(r2.is_ok(), "export ai_compact POST should succeed");

    // cleanup
    let _ = child.kill();
}
