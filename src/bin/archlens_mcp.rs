use std::{io::{self, BufRead, Write}, path::PathBuf, fs};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::Stream;
use axum::{routing::{get, post}, Router, response::sse::{Event, Sse}, extract::State, Json};

use archlens::{ensure_absolute_path, cli::{self, export, diagram, stats}};

// =============== Types ===============
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeArgs {
    pub project_path: String,
    pub deep: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportArgs {
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructureArgs {
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiagramArgs {
    pub project_path: String,
    pub diagram_type: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDescription { pub name: String, pub uri: String, #[serde(skip_serializing_if="Option::is_none")] pub mime: Option<String>, #[serde(skip_serializing_if="Option::is_none")] pub description: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage { pub role: String, pub content: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDescription { pub name: String, pub description: String, pub messages: Vec<PromptMessage> }

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

async fn post_export(Json(args): Json<ExportArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    match export::generate_ai_compact(path.to_string_lossy().as_ref()) {
        Ok(md) => Ok(Json(serde_json::json!({"status":"ok","output": md}))),
        Err(e) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

async fn post_structure(Json(args): Json<StructureArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    match stats::get_project_structure(path.to_string_lossy().as_ref()) {
        Ok(structure) => Ok(Json(serde_json::json!({"status":"ok","structure": structure}))),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

async fn post_diagram(Json(args): Json<DiagramArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let path = ensure_absolute_path(&args.project_path);
    let kind = args.diagram_type.as_deref().unwrap_or("mermaid");
    if kind != "mermaid" { return Err(axum::http::StatusCode::BAD_REQUEST); }
    match diagram::generate_mermaid_diagram(path.to_string_lossy().as_ref()) {
        Ok(mmd) => Ok(Json(serde_json::json!({"status":"ok","diagram_type":"mermaid","diagram": mmd}))),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

async fn get_schemas() -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let list = list_schema_resources();
    Ok(Json(serde_json::json!({"resources": list})))
}

fn build_http_router() -> Router {
    Router::new()
        .route("/sse/refresh", get(sse_refresh))
        .route("/export/ai_compact", post(post_export))
        .route("/structure/get", post(post_structure))
        .route("/diagram/generate", post(post_diagram))
        .route("/schemas/list", get(get_schemas))
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
        "prompts/list" => {
            let prompts = list_prompts();
            Ok(serde_json::json!({"prompts": prompts}))
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
                    Ok(serde_json::json!({"content":[{"type":"text","text": out}]}))
                }
                "structure.get" => {
                    let args: StructureArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let st = stats::get_project_structure(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({"content":[{"type":"text","text": serde_json::to_string_pretty(&st).unwrap()}]}))
                }
                "graph.build" => {
                    let args: DiagramArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let mmd = diagram::generate_mermaid_diagram(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    Ok(serde_json::json!({"content":[{"type":"text","text": format!("```mermaid\n{}\n```", mmd)}]}))
                }
                "analyze.project" => {
                    let args: AnalyzeArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    if args.deep.unwrap_or(false) {
                        let res = cli::handlers::run_deep_pipeline(path.to_string_lossy().as_ref())
                            .map_err(|e| e.to_string())?;
                        Ok(serde_json::json!({"content":[{"type":"text","text": serde_json::to_string_pretty(&res).unwrap_or_else(|_|"{}".into())}]}))
                    } else {
                        let s = stats::get_project_stats(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                        Ok(serde_json::json!({"content":[{"type":"text","text": serde_json::to_string_pretty(&s).unwrap()}]}))
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

    // 2) HTTP сервер (Streamable)
    let app = build_http_router();
    let http = axum::Server::bind(&"0.0.0.0:5178".parse()?)
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
                    let res = handle_call(&r.method, r.params);
                    match res {
                        Ok(val) => write_json_line(id, Some(val), None),
                        Err(msg) => write_json_line(id, Option::<serde_json::Value>::None, Some(RpcError{code:-32603, message: msg})),
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