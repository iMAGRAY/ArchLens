use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_export_times_out() {
    // Start server with tiny timeout and artificial delay
    let port = 5196u16;
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_TIMEOUT_MS", "100")
        .env("ARCHLENS_TEST_DELAY_MS", "500")
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http timeout e2e");
            return;
        }
    };

    thread::sleep(Duration::from_millis(300));
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

    let resp = client
        .post(&format!("http://127.0.0.1:{}/export/ai_compact", port))
        .json(&serde_json::json!({"project_path":"tests/fixtures/small_project"}))
        .send();

    assert!(resp.is_ok(), "request should complete");
    let status = resp.unwrap().status();
    assert_eq!(status.as_u16(), 408, "should return 408 REQUEST_TIMEOUT");

    let _ = child.kill();
}
