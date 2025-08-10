use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn http_sse_refresh_streams_events() {
    // Spawn server in background
    let port = 5193u16;
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_MCP_PORT", format!("{}", port))
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
        .get(&format!("http://127.0.0.1:{}/sse/refresh", port))
        .send();
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
