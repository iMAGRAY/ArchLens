use std::{io::{self, BufRead, Write}, path::PathBuf, fs};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::Stream;
use axum::{routing::{get, post}, Router, response::sse::{Event, Sse}, extract::State, Json};
use std::time::Duration;

use archlens::{ensure_absolute_path, cli::{self, export, diagram, stats}};
use regex::Regex;

// =============== Types ===============
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeArgs {
    pub project_path: String,
    pub deep: Option<bool>,
    pub detail_level: Option<String>, // summary|standard|full
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportArgs {
    pub project_path: String,
    pub detail_level: Option<String>, // summary|standard|full
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructureArgs {
    pub project_path: String,
    pub detail_level: Option<String>, // summary|standard|full
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiagramArgs {
    pub project_path: String,
    pub diagram_type: Option<String>,
    pub detail_level: Option<String>, // summary|standard|full
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcParams {
    Analyze(AnalyzeArgs),
    Export(ExportArgs),
    Structure(StructureArgs),
    Diagram(DiagramArgs),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")] pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")] pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError { pub code: i32, pub message: String }

// =============== MCP-like Tool List ===============
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolDescription { pub name: String, pub description: String, pub input_schema: serde_json::Value }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceDescription { pub name: String, pub uri: String, #[serde(skip_serializing_if="Option::is_none")] pub mime: Option<String>, #[serde(skip_serializing_if="Option::is_none")] pub description: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceReadArgs { pub uri: String }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptMessage { pub role: String, pub content: String }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptDescription { pub name: String, pub description: String, pub messages: Vec<PromptMessage> }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptGetArgs { pub name: String }

#[derive(Clone)]
struct HttpState;

// =============== HTTP (Streamable) ===============
async fn sse_refresh(State(_): State<HttpState>) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, axum::Error>>(8);
    tokio::spawn(async move {
        let _ = tx.send(Ok(Event::default().data("event: refresh-start"))).await;
        let _ = tx.send(Ok(Event::default().data("event: refresh-done"))).await;
    });
    Sse::new(ReceiverStream::new(rx))
}

// Formatting limits
const SUMMARY_LIMIT_CHARS: usize = 30_000;
const MAX_OUTPUT_CHARS: usize = 1_000_000;

fn clamp_text(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars { return s.to_string(); }
    let mut out = s[..max_chars].to_string();
    out.push_str("\n... (truncated)");
    out
}

fn strip_code_blocks(md: &str) -> String {
    let re = Regex::new(r"(?s)```.*?```").ok();
    let mut out = md.to_string();
    if let Some(re) = re { out = re.replace_all(&out, "").to_string(); }
    // compress blank lines
    let re2 = Regex::new(r"\n{3,}").ok();
    if let Some(re2) = re2 { out = re2.replace_all(&out, "\n\n").to_string(); }
    out
}

fn level<'a>(opt: &'a Option<String>) -> &'a str {
    match opt.as_deref() { Some("standard") => "standard", Some("full") => "full", _ => "summary" }
}

fn format_analysis_result(project_path: &str, ps: &stats::ProjectStats, detail_level: &str) -> String {
    let mut out = String::new();
    out.push_str("# 🔍 PROJECT ANALYSIS\n");
    out.push_str(&format!("**Path:** {}\n", project_path));
    out.push_str(&format!("- Files: {}\n", ps.total_files));
    out.push_str(&format!("- Lines: {}\n", ps.total_lines));
    // file types sorted desc
    let mut types: Vec<(String, usize)> = ps.file_types.iter().map(|(k,v)|(k.clone(),*v)).collect();
    types.sort_by(|a,b| b.1.cmp(&a.1));
    let take = match detail_level { "full" => types.len(), "standard" => types.len().min(10), _ => types.len().min(5) };
    if take>0 {
        let list = types.into_iter().take(take).map(|(ext,c)| format!(".{}:{}", ext, c)).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("- Types: {}\n", list));
    }
    out
}

fn format_structure_result(project_path: &str, st: &stats::ProjectStructure, detail_level: &str) -> String {
    let mut out = String::new();
    out.push_str("# 📁 STRUCTURE\n");
    out.push_str(&format!("**Path:** {}\n", project_path));
    out.push_str(&format!("- Files: {}\n", st.total_files));
    if !st.layers.is_empty() {
        out.push_str(&format!("- Layers: {}\n", st.layers.join(", ")));
    }
    // file types top N
    let mut types: Vec<(String, usize)> = st.file_types.iter().map(|(k,v)|(k.clone(),*v)).collect();
    types.sort_by(|a,b| b.1.cmp(&a.1));
    let take = match detail_level { "full" => types.len(), "standard" => types.len().min(10), _ => types.len().min(5) };
    if take>0 {
        let list = types.into_iter().take(take).map(|(ext,c)| format!(".{}:{}", ext, c)).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("- Types: {}\n", list));
    }
    if detail_level == "full" {
        // show first 25 files
        let files = st.files.iter().take(25).map(|f| format!("- `{}` ({}, {:.1}KB)", f.path, f.extension, (f.size as f64)/1024.0)).collect::<Vec<_>>().join("\n");
        if !files.is_empty() { out.push_str("\n"); out.push_str(&files); }
    }
    out
}

fn format_export_markdown(md: String, detail_level: &str) -> String {
    if detail_level == "full" {
        return clamp_text(&md, MAX_OUTPUT_CHARS);
    }
    let stripped = strip_code_blocks(&md);
    if detail_level == "standard" {
        clamp_text(&stripped, SUMMARY_LIMIT_CHARS * 2)
    } else {
        clamp_text(&stripped, SUMMARY_LIMIT_CHARS)
    }
}

fn format_diagram_text(mmd: String, project_path: &str, detail_level: &str) -> String {
    let mut limit = 120_000usize;
    if detail_level == "summary" { limit = 15_000; } else if detail_level == "standard" { limit = 40_000; }
    let body = clamp_text(&mmd, limit);
    format!("# 📊 DIAGRAM\nPath: {}\nType: mermaid\n\n```mermaid\n{}\n```", project_path, body)
}

fn env_timeout_ms() -> u64 {
    std::env::var("ARCHLENS_TIMEOUT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(60_000)
}

async fn post_export(Json(args): Json<ExportArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    let lv = level(&args.detail_level).to_string();
    let timeout = Duration::from_millis(env_timeout_ms());
    let handle = tokio::task::spawn_blocking(move || export::generate_ai_compact(path.to_string_lossy().as_ref()));
    match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(md)) => {
                let txt = format_export_markdown(md, &lv);
                Ok(Json(serde_json::json!({"status":"ok","output": txt})))
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    }
}

async fn post_structure(Json(args): Json<StructureArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    let lv = level(&args.detail_level).to_string();
    let timeout = Duration::from_millis(env_timeout_ms());
    let handle = tokio::task::spawn_blocking(move || stats::get_project_structure(path.to_string_lossy().as_ref()));
    match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(structure)) => {
                let text = format_structure_result(path.to_string_lossy().as_ref(), &structure, &lv);
                Ok(Json(serde_json::json!({"status":"ok","text": text})))
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    }
}

async fn post_diagram(Json(args): Json<DiagramArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    let kind = args.diagram_type.as_deref().unwrap_or("mermaid");
    if kind != "mermaid" { return Err(axum::http::StatusCode::BAD_REQUEST); }
    let lv = level(&args.detail_level).to_string();
    let timeout = Duration::from_millis(env_timeout_ms());
    let p = path.clone();
    let handle = tokio::task::spawn_blocking(move || {
        // Prefer full graph-based mermaid; fallback to simple import-based
        cli::handlers::build_graph_mermaid(p.to_string_lossy().as_ref())
            .or_else(|_| diagram::generate_mermaid_diagram(p.to_string_lossy().as_ref()))
    });
    match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(mmd)) => {
                let text = format_diagram_text(mmd, path.to_string_lossy().as_ref(), &lv);
                Ok(Json(serde_json::json!({"status":"ok","diagram_type":"mermaid","text": text})))
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    }
}

async fn get_schemas() -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let list = list_schema_resources();
    Ok(Json(serde_json::json!({"resources": list})))
}

async fn post_schema_read(Json(args): Json<ResourceReadArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    match read_resource_uri(&args.uri) {
        Ok((mime, text)) => Ok(Json(serde_json::json!({"uri": args.uri, "mime": mime, "text": text}))),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

fn build_http_router() -> Router {
    Router::new()
        .route("/sse/refresh", get(sse_refresh))
        .route("/export/ai_compact", post(post_export))
        .route("/structure/get", post(post_structure))
        .route("/diagram/generate", post(post_diagram))
        .route("/schemas/list", get(get_schemas))
        .route("/schemas/read", post(post_schema_read))
        .with_state(HttpState)
}

// =============== STDIO JSON-RPC ===============
fn write_json_line<T: Serialize>(id: serde_json::Value, result: Option<T>, error: Option<RpcError>) {
    let resp = RpcResponse { jsonrpc: "2.0".into(), id, result, error };
    let line = serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{{\"code\":-32603,\"message\":\"{}\"}}}}", e));
    let mut stdout = io::stdout();
    let _ = stdout.write_all(line.as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

fn tool_list_schema() -> Vec<ToolDescription> {
    let analyze_schema = schemars::schema_for!(AnalyzeArgs);
    let export_schema = schemars::schema_for!(ExportArgs);
    let structure_schema = schemars::schema_for!(StructureArgs);
    let diagram_schema = schemars::schema_for!(DiagramArgs);

    vec![
        ToolDescription { name: "arch.refresh".into(), description: "Refresh analysis context (noop placeholder)".into(), input_schema: serde_json::json!({"type":"object"}) },
        ToolDescription { name: "graph.build".into(), description: "Build architecture diagram (mermaid)".into(), input_schema: serde_json::to_value(diagram_schema.schema).unwrap() },
        ToolDescription { name: "export.ai_compact".into(), description: "Export AI compact analysis".into(), input_schema: serde_json::to_value(export_schema.schema).unwrap() },
        ToolDescription { name: "structure.get".into(), description: "Get project structure".into(), input_schema: serde_json::to_value(structure_schema.schema).unwrap() },
        ToolDescription { name: "analyze.project".into(), description: "Analyze project (shallow or deep)".into(), input_schema: serde_json::to_value(analyze_schema.schema).unwrap() },
    ]
}

fn list_schema_resources() -> Vec<ResourceDescription> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dir = root.join("out").join("schemas");
    let mut resources = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()) == Some("json") {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let abs = p.canonicalize().unwrap_or(p.clone());
                let uri = format!("file://{}", abs.to_string_lossy());
                resources.push(ResourceDescription { name, uri, mime: Some("application/schema+json".into()), description: Some("JSON Schema for tool input".into()) });
            }
        }
    }
    resources
}

fn read_resource_uri(uri: &str) -> Result<(String, String), String> {
    if let Some(path) = uri.strip_prefix("file://") {
        let p = PathBuf::from(path);
        let text = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let mime = if p.extension().and_then(|e| e.to_str()) == Some("json") { "application/schema+json" } else { "text/plain" };
        Ok((mime.to_string(), text))
    } else {
        Err("unsupported uri".into())
    }
}

fn list_prompts() -> Vec<PromptDescription> {
    vec![
        PromptDescription {
            name: "ai.compact.summarize".into(),
            description: "Summarize AI compact analysis into 5 bullet points".into(),
            messages: vec![ PromptMessage{ role: "system".into(), content: "You are an expert software architect.".into() },
                            PromptMessage{ role: "user".into(), content: "Summarize the following analysis into 5 bullets with actionable items.".into() } ]
        },
        PromptDescription {
            name: "graph.explain.cycles".into(),
            description: "Explain detected cycles and propose fixes".into(),
            messages: vec![ PromptMessage{ role: "system".into(), content: "You explain architecture issues clearly.".into() },
                            PromptMessage{ role: "user".into(), content: "Explain cycles and propose refactoring strategies.".into() } ]
        }
    ]
}

fn get_prompt_by_name(name: &str) -> Option<PromptDescription> {
    list_prompts().into_iter().find(|p| p.name == name)
}

fn handle_call(method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    match method {
        "tools/list" => {
            let tools = tool_list_schema();
            Ok(serde_json::json!({"tools": tools}))
        }
        "resources/list" => {
            let resources = list_schema_resources();
            Ok(serde_json::json!({"resources": resources}))
        }
        "resources/read" => {
            let args: ResourceReadArgs = serde_json::from_value(params.ok_or("missing params")?)
                .map_err(|e| e.to_string())?;
            let (mime, text) = read_resource_uri(&args.uri)?;
            Ok(serde_json::json!({"resource": {"uri": args.uri, "mime": mime, "text": text}}))
        }
        "prompts/list" => {
            let prompts = list_prompts();
            Ok(serde_json::json!({"prompts": prompts}))
        }
        "prompts/get" => {
            let args: PromptGetArgs = serde_json::from_value(params.ok_or("missing params")?)
                .map_err(|e| e.to_string())?;
            let p = get_prompt_by_name(&args.name).ok_or("prompt not found")?;
            Ok(serde_json::json!({"prompt": p}))
        }
        "tools/call" => {
            let obj = params.ok_or("missing params")?;
            let name = obj.get("name").and_then(|v| v.as_str()).ok_or("missing name")?;
            let args = obj.get("arguments").cloned().unwrap_or(serde_json::json!({}));
            match name {
                "export.ai_compact" => {
                    let args: ExportArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let out = export::generate_ai_compact(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    let txt = format_export_markdown(out, level(&args.detail_level));
                    Ok(serde_json::json!({"content":[{"type":"text","text": txt}]}))
                }
                "structure.get" => {
                    let args: StructureArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let st = stats::get_project_structure(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    let txt = format_structure_result(path.to_string_lossy().as_ref(), &st, level(&args.detail_level));
                    Ok(serde_json::json!({"content":[{"type":"text","text": txt}]}))
                }
                "graph.build" => {
                    let args: DiagramArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    // Prefer graph-based; fallback to simple
                    let mmd = cli::handlers::build_graph_mermaid(path.to_string_lossy().as_ref())
                        .or_else(|_| diagram::generate_mermaid_diagram(path.to_string_lossy().as_ref()))?;
                    let txt = format_diagram_text(mmd, path.to_string_lossy().as_ref(), level(&args.detail_level));
                    Ok(serde_json::json!({"content":[{"type":"text","text": txt}]}))
                }
                "analyze.project" => {
                    let args: AnalyzeArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    if args.deep.unwrap_or(false) {
                        let res = cli::handlers::run_deep_pipeline(path.to_string_lossy().as_ref())
                            .map_err(|e| e.to_string())?;
                        let lv = level(&args.detail_level);
                        let txt = clamp_text(&res, if lv=="full" { MAX_OUTPUT_CHARS } else if lv=="standard" { SUMMARY_LIMIT_CHARS * 2 } else { SUMMARY_LIMIT_CHARS });
                        Ok(serde_json::json!({"content":[{"type":"text","text": txt}]}))
                    } else {
                        let s = stats::get_project_stats(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                        let lv = level(&args.detail_level);
                        let txt = format_analysis_result(path.to_string_lossy().as_ref(), &s, lv);
                        Ok(serde_json::json!({"content":[{"type":"text","text": txt}]}))
                    }
                }
                "arch.refresh" => {
                    Ok(serde_json::json!({"content":[{"type":"text","text": "ok"}]}))
                }
                _ => Err(format!("unknown tool: {}", name)),
            }
        }
        _ => Err("unknown method".into())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Генерация JSON схем во время запуска (можно вынести в build.rs при необходимости)
    let schemas_dir = PathBuf::from("out/schemas");
    fs::create_dir_all(&schemas_dir).ok();
    let write_schema = |name: &str, schema: schemars::schema::RootSchema| {
        let p = schemas_dir.join(format!("{}.schema.json", name));
        let _ = fs::write(p, serde_json::to_vec_pretty(&schema).unwrap());
    };
    write_schema("analyze_args", schemars::schema_for!(AnalyzeArgs));
    write_schema("export_args", schemars::schema_for!(ExportArgs));
    write_schema("structure_args", schemars::schema_for!(StructureArgs));
    write_schema("diagram_args", schemars::schema_for!(DiagramArgs));
    write_schema("resource_read_args", schemars::schema_for!(ResourceReadArgs));
    write_schema("prompt_get_args", schemars::schema_for!(PromptGetArgs));

    // 2) HTTP сервер (Streamable)
    let port: u16 = std::env::var("ARCHLENS_MCP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(5178);
    let addr = format!("0.0.0.0:{}", port);
    let app = build_http_router();
    let http = axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service());

    // 3) STDIO JSON-RPC петля
    let stdio = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();
        while let Some(Ok(line)) = lines.next() {
            if line.trim().is_empty() { continue; }
            let req: Result<RpcRequest, _> = serde_json::from_str(&line);
            match req {
                Ok(r) => {
                    let id = r.id.clone();
                    // Detect heavy tools to apply timeout
                    let mut handled_with_timeout = false;
                    if r.method == "tools/call" {
                        if let Some(params) = r.params.clone() {
                            let name_opt = params.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                            if let Some(tool_name) = name_opt {
                                let is_heavy = matches!(tool_name.as_str(),
                                    "export.ai_compact" | "structure.get" | "graph.build" | "analyze.project");
                                if is_heavy {
                                    handled_with_timeout = true;
                                    let timeout = Duration::from_millis(env_timeout_ms());
                                    let method = r.method.clone();
                                    let pclone = r.params.clone();
                                    let handle = tokio::task::spawn_blocking(move || handle_call(&method, pclone));
                                    match tokio::time::timeout(timeout, handle).await {
                                        Ok(joined) => match joined {
                                            Ok(Ok(val)) => write_json_line(id, Some(val), None),
                                            Ok(Err(msg)) => write_json_line(id, Option::<serde_json::Value>::None, Some(RpcError{code:-32603, message: msg})),
                                            Err(e) => write_json_line(id, Option::<serde_json::Value>::None, Some(RpcError{code:-32603, message: format!("join error: {}", e)})),
                                        },
                                        Err(_) => write_json_line(id, Option::<serde_json::Value>::None, Some(RpcError{code:-32000, message: "timeout".into()})),
                                    }
                                }
                            }
                        }
                    }
                    if !handled_with_timeout {
                        let res = handle_call(&r.method, r.params);
                        match res {
                            Ok(val) => write_json_line(id, Some(val), None),
                            Err(msg) => write_json_line(id, Option::<serde_json::Value>::None, Some(RpcError{code:-32603, message: msg})),
                        }
                    }
                }
                Err(e) => write_json_line(serde_json::json!(null), Option::<serde_json::Value>::None, Some(RpcError{code:-32700, message: e.to_string()})),
            }
        }
    });

    tokio::select!{
        _ = http => Ok(()),
        _ = stdio => Ok(()),
    }
}