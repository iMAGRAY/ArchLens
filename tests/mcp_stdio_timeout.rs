use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn stdio_export_times_out() {
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .env("ARCHLENS_TIMEOUT_MS", "100")
        .env("ARCHLENS_TEST_DELAY_MS", "500")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping stdio timeout e2e");
            return;
        }
    };

    // Send tools/call for heavy tool
    {
        let mut stdin = child.stdin.take().unwrap();
        let call = r#"{"jsonrpc":"2.0","id":99,"method":"tools/call","params":{"name":"export.ai_compact","arguments":{"project_path":"."}}}
"#;
        stdin.write_all(call.as_bytes()).unwrap();
    }

    // Read output
    let mut out = String::new();
    let _ = child.stdout.take().unwrap().read_to_string(&mut out);

    assert!(out.contains("\"error\":"), "should return error");
    assert!(out.contains("timeout"), "error message should mention timeout. Got: {}", out);

    let _ = child.kill();
}