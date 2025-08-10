use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_export_times_out() {
    // Start server with tiny timeout and artificial delay
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_TIMEOUT_MS", "100")
        .env("ARCHLENS_TEST_DELAY_MS", "500")
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

    let resp = client
        .post("http://127.0.0.1:5178/export/ai_compact")
        .json(&serde_json::json!({"project_path":"."}))
        .send();

    assert!(resp.is_ok(), "request should complete");
    let status = resp.unwrap().status();
    assert_eq!(status.as_u16(), 408, "should return 408 REQUEST_TIMEOUT");

    let _ = child.kill();
}
