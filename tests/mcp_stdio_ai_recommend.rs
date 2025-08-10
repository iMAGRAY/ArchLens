use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn stdio_ai_recommend_starts_with_summary() {
    let mut child = match Command::new(env!("CARGO_BIN_EXE_archlens-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!("archlens-mcp not built; skipping stdio ai.recommend e2e");
            return;
        }
    };

    {
        let mut stdin = child.stdin.take().unwrap();
        let call = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ai.recommend","arguments":{"project_path":"."}}}
"#;
        stdin.write_all(call.as_bytes()).unwrap();
    }

    let mut out = String::new();
    let _ = child.stdout.take().unwrap().read_to_string(&mut out);
    assert!(
        out.contains("recommendations"),
        "should return recommendations, got: {}",
        out
    );

    let _ = child.kill();
}
