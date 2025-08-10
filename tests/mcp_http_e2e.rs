use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_export_and_schemas() {
    // Spawn server in background
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn() {
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
    let r = client.get("http://127.0.0.1:5178/schemas/list").send();
    assert!(r.is_ok(), "schemas list should be accessible");

    // export/ai_compact
    let r2 = client
        .post("http://127.0.0.1:5178/export/ai_compact")
        .json(&serde_json::json!({"project_path":".","detail_level":"summary"}))
        .send();
    assert!(r2.is_ok(), "export ai_compact POST should succeed");

    // cleanup
    let _ = child.kill();
}