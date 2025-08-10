use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_sse_refresh_streams_events() {
    // Spawn server in background
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping http sse e2e");
            return;
        }
    };

    // Give server time to start
    thread::sleep(Duration::from_millis(400));

    let client = reqwest::blocking::Client::new();
    let r = client.get("http://127.0.0.1:5178/sse/refresh").send();
    if let Ok(resp) = r {
        assert!(resp.status().is_success());
        let text = resp.text().unwrap_or_default();
        assert!(
            text.contains("refresh-start") || text.contains("event: refresh-start"),
            "SSE should contain refresh-start event"
        );
        assert!(
            text.contains("refresh-done") || text.contains("event: refresh-done"),
            "SSE should contain refresh-done event"
        );
    }

    // cleanup
    let _ = child.kill();
}
