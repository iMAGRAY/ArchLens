use assert_cmd::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn stdio_tools_list_and_structure() {
    // Try to run archlens-mcp from cargo target or PATH
    let mut cmd = match Command::cargo_bin("archlens-mcp") {
        Ok(b) => b,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping e2e");
            return;
        }
    };
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");

    let mut stdin = child.stdin.take().unwrap();
    let tools = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}
"#;
    stdin.write_all(tools.as_bytes()).unwrap();

    let call = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"structure.get\",\"arguments\":{\"project_path\":\".\",\"detail_level\":\"summary\"}}}\n".to_string();
    stdin.write_all(call.as_bytes()).unwrap();

    drop(stdin);

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Minimal assertions
    assert!(
        stdout.contains("\"tools\":"),
        "tools/list response missing tools"
    );
    assert!(
        stdout.contains("STRUCTURE"),
        "structure.get should return STRUCTURE header, got: {}",
        stdout
    );
}
