use std::{io::{self, BufRead, Write}, path::PathBuf, fs};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::Stream;
use axum::{routing::{get, post}, Router, response::sse::{Event, Sse}, extract::State, Json};
use std::time::{Duration, Instant};
use std::thread;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::{Read, Write};
use std::path::Path; // added
use std::process::Command; // added

use archlens::{ensure_absolute_path, cli::{self, export, diagram, stats}};
use regex::Regex;

// =============== Types ===============
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeArgs {
    pub project_path: String,
    pub deep: Option<bool>,
    pub detail_level: Option<String>, // summary|standard|full
    pub max_output_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportArgs {
    pub project_path: String,
    pub detail_level: Option<String>, // summary|standard|full
    pub max_output_chars: Option<usize>,
    pub sections: Option<Vec<String>>, // e.g., ["summary","problems_validated","cycles"] or exact headers
    pub top_n: Option<usize>,          // limit list items in sections
    pub etag: Option<String>,
    pub use_cache: Option<bool>,       // default true
    pub cache_ttl_ms: Option<u64>,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StructureArgs {
    pub project_path: String,
    pub detail_level: Option<String>, // summary|standard|full
    pub max_output_chars: Option<usize>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiagramArgs {
    pub project_path: String,
    pub diagram_type: Option<String>,
    pub detail_level: Option<String>, // summary|standard|full
    pub max_output_chars: Option<usize>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AISummaryArgs {
    pub project_path: String,
    pub top_n: Option<usize>,
    pub max_output_chars: Option<usize>,
    pub etag: Option<String>,
    pub use_cache: Option<bool>,
    pub cache_ttl_ms: Option<u64>,
    pub force: Option<bool>,
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
pub struct ToolDescription { pub name: String, pub description: String, pub input_schema: serde_json::Value, #[serde(skip_serializing_if="Option::is_none")] pub schema_uri: Option<String> }

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIRecommendArgs {
    pub project_path: String,
    #[serde(default)] pub json: Option<serde_json::Value>,
    #[serde(default)] pub focus: Option<String>,
}

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

async fn http_tools_list(State(_): State<HttpState>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let tools = tool_list_schema();
    Ok(Json(serde_json::json!({"tools": tools})))
}

async fn http_tools_call(State(_): State<HttpState>, Json(obj): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if name.is_empty() { return Err(axum::http::StatusCode::BAD_REQUEST); }
    let is_heavy = matches!(name.as_str(), "export.ai_compact" | "structure.get" | "graph.build" | "analyze.project");
    if is_heavy {
        let timeout = Duration::from_millis(env_timeout_ms());
        let payload = obj.clone();
        let handle = tokio::task::spawn_blocking(move || handle_call("tools/call", Some(payload)));
        match tokio::time::timeout(timeout, handle).await {
            Ok(joined) => match joined {
                Ok(Ok(val)) => Ok(Json(val)),
                Ok(Err(_e)) => Err(axum::http::StatusCode::BAD_REQUEST),
                Err(_join) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            },
            Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT),
        }
    } else {
        match handle_call("tools/call", Some(obj)) {
            Ok(val) => Ok(Json(val)),
            Err(_e) => Err(axum::http::StatusCode::BAD_REQUEST),
        }
    }
}

async fn sse_tools_call(State(_): State<HttpState>, Json(obj): Json<serde_json::Value>) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, axum::Error>>(8);
    tokio::spawn(async move {
        let _ = tx.send(Ok(Event::default().event("start").data("tools-call-start"))).await;
        let timeout = Duration::from_millis(env_timeout_ms());
        let payload = obj.clone();
        let handle = tokio::task::spawn_blocking(move || handle_call("tools/call", Some(payload)));
        let res = tokio::time::timeout(timeout, handle).await;
        match res {
            Ok(joined) => match joined {
                Ok(Ok(val)) => {
                    let text = serde_json::to_string(&val).unwrap_or("{}".into());
                    let _ = tx.send(Ok(Event::default().event("result").data(text))).await;
                    let _ = tx.send(Ok(Event::default().event("done").data("ok"))).await;
                }
                Ok(Err(e)) => {
                    let _ = tx.send(Ok(Event::default().event("error").data(e))).await;
                }
                Err(e) => {
                    let _ = tx.send(Ok(Event::default().event("error").data(format!("join error: {}", e)))).await;
                }
            },
            Err(_) => {
                let _ = tx.send(Ok(Event::default().event("error").data("timeout"))).await;
            }
        }
    });
    Sse::new(ReceiverStream::new(rx))
}

fn build_http_router() -> Router {
    Router::new()
        .route("/sse/refresh", get(sse_refresh))
        .route("/export/ai_compact", post(post_export))
        .route("/export/ai_summary_json", post(post_export_summary))
        .route("/structure/get", post(post_structure))
        .route("/diagram/generate", post(post_diagram))
        .route("/schemas/list", get(get_schemas))
        .route("/schemas/read", post(post_schema_read))
        .route("/presets/list", get(get_presets))
        .route("/ai/recommend", post(get_recommendations))
        // Official MCP-style HTTP endpoints
        .route("/tools/list", post(http_tools_list))
        .route("/tools/call", post(http_tools_call))
        .route("/tools/call/stream", post(sse_tools_call))
        .with_state(HttpState)
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

fn clamp_text_with_limit(s: &str, req_limit: Option<usize>) -> String {
    let hard = MAX_OUTPUT_CHARS;
    let eff = req_limit.map(|v| v.min(hard)).unwrap_or(hard);
    clamp_text(s, eff)
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

fn canonical_section_key(name: &str) -> &str {
    let n = name.trim().to_lowercase();
    match n.as_str() {
        "summary" => "## summary",
        "problems" | "problems_validated" => "## problems (validated)",
        "problems_heuristic" => "## problems (heuristic)",
        "cycles" | "cycles (top)" => "## cycles (top)",
        "coupling" | "top coupling" => "## top coupling",
        "complexity" | "top complexity components" => "## top complexity components",
        "layers" => "## layers",
        other => other,
    }
}

fn filter_markdown_sections(md: &str, sections: &Option<Vec<String>>) -> String {
    if sections.is_none() { return md.to_string(); }
    let wanted: Vec<String> = sections.as_ref().unwrap().iter().map(|s| canonical_section_key(s).to_string()).collect();
    let mut out = String::new();
    let mut include = false;
    for line in md.lines() {
        if line.starts_with("# ") {
            // always keep top title
            if out.is_empty() { out.push_str(line); out.push('\n'); }
            continue;
        }
        if line.starts_with("## ") {
            let key = canonical_section_key(line).to_string();
            include = wanted.iter().any(|w| key.starts_with(&w));
            if include { out.push_str(line); out.push('\n'); }
            continue;
        }
        if include { out.push_str(line); out.push('\n'); }
    }
    if out.trim().is_empty() { md.to_string() } else { out }
}

fn trim_bullets_in_sections(md: &str, top_n: Option<usize>) -> String {
    if top_n.is_none() { return md.to_string(); }
    let limit = top_n.unwrap_or(0);
    if limit == 0 { return md.to_string(); }
    let mut out = String::new();
    let mut current_is_bullet_section = false;
    let mut count = 0usize;
    for line in md.lines() {
        if line.starts_with("## ") {
            current_is_bullet_section = line.contains("Top Coupling") || line.contains("Top Complexity Components") || line.contains("Cycles (Top)");
            count = 0;
            out.push_str(line); out.push('\n');
            continue;
        }
        if current_is_bullet_section && line.trim_start().starts_with("-") {
            if count < limit { out.push_str(line); out.push('\n'); }
            count += 1;
            continue;
        }
        out.push_str(line); out.push('\n');
    }
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

fn format_export_markdown_with_controls(md: String, detail_level: &str, sections: &Option<Vec<String>>, top_n: Option<usize>, max_chars: Option<usize>) -> String {
    // filter first to reduce size
    let mut content = filter_markdown_sections(&md, sections);
    content = trim_bullets_in_sections(&content, top_n);
    // then apply standard formatting
    let formatted = format_export_markdown(content, detail_level);
    clamp_text_with_limit(&formatted, max_chars)
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

fn env_cache_ttl_ms() -> u64 {
    std::env::var("ARCHLENS_CACHE_TTL_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(120_000)
}

fn env_test_delay_ms() -> Option<u64> {
    std::env::var("ARCHLENS_TEST_DELAY_MS").ok().and_then(|s| s.parse().ok())
}

// Recommendation thresholds (configurable via env)
#[derive(Clone, Copy, Debug)]
struct RecoThresholds {
    complexity_avg: f64,
    coupling_index: f64,
    cohesion_index: f64,
    layer_imbalance_pct: u8,
    high_sev_cats: usize,
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}
fn env_u8(name: &str, default: u8) -> u8 {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}
fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn reco_thresholds_from_env() -> RecoThresholds {
    RecoThresholds {
        complexity_avg: env_f64("ARCHLENS_TH_COMPLEXITY_AVG", 8.0),
        coupling_index: env_f64("ARCHLENS_TH_COUPLING_INDEX", 0.7),
        cohesion_index: env_f64("ARCHLENS_TH_COHESION_INDEX", 0.3),
        layer_imbalance_pct: env_u8("ARCHLENS_TH_LAYER_IMBALANCE_PCT", 60),
        high_sev_cats: env_usize("ARCHLENS_TH_HIGH_SEV_CATS", 2),
    }
}

async fn post_export(Json(args): Json<ExportArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let t0 = Instant::now();
    let abspath = ensure_absolute_path(&args.project_path);
    let lv = level(&args.detail_level).to_string();
    let use_cache = args.use_cache.unwrap_or(true) && !args.force.unwrap_or(false);
    let ttl = args.cache_ttl_ms.unwrap_or_else(env_cache_ttl_ms);
    let key = export_cache_key(&abspath.to_string_lossy(), &lv, &args.sections, args.top_n, args.max_output_chars);

    if use_cache {
        if let Some((etag_cached, output_cached)) = cache_get(&key, ttl) {
            if args.etag.as_deref() == Some(&etag_cached) {
                tracing::info!(target="archlens_mcp", action="export_ai_compact_cache", hit=true, not_modified=true);
                return Ok(Json(serde_json::json!({"status":"not_modified","etag": etag_cached})));
            } else {
                tracing::info!(target="archlens_mcp", action="export_ai_compact_cache", hit=true, not_modified=false);
                return Ok(Json(serde_json::json!({"status":"ok","etag": etag_cached, "output": output_cached})));
            }
        }
    }

    let timeout = Duration::from_millis(env_timeout_ms());
    let delay = env_test_delay_ms();
    let path_clone = abspath.clone();
    let lv_clone = lv.clone();
    let sections_clone = args.sections.clone();
    let topn_clone = args.top_n;
    let max_clone = args.max_output_chars;
    let handle = tokio::task::spawn_blocking(move || {
        if let Some(ms) = delay { thread::sleep(Duration::from_millis(ms)); }
        export::generate_ai_compact(path_clone.to_string_lossy().as_ref()).map(|md| format_export_markdown_with_controls(md, &lv_clone, &sections_clone, topn_clone, max_clone))
    });
    let out = match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(txt)) => {
                let etag = content_etag(&txt);
                if args.use_cache.unwrap_or(true) { cache_put(&key, &etag, &txt); }
                if args.etag.as_deref() == Some(&etag) {
                    Ok(Json(serde_json::json!({"status":"not_modified","etag": etag})))
                } else {
                    Ok(Json(serde_json::json!({"status":"ok","etag": etag, "output": txt})))
                }
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    };
    tracing::info!(target: "archlens_mcp", action = "export_ai_compact", ms = %t0.elapsed().as_millis());
    out
}

async fn post_export_summary(Json(args): Json<AISummaryArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let t0 = Instant::now();
    let abspath = ensure_absolute_path(&args.project_path);
    let use_cache = args.use_cache.unwrap_or(true) && !args.force.unwrap_or(false);
    let ttl = args.cache_ttl_ms.unwrap_or_else(env_cache_ttl_ms);
    let key = export_cache_key(&abspath.to_string_lossy(), "json_summary", &Some(vec!["__json_summary__".into()]), args.top_n, args.max_output_chars);
    if use_cache {
        if let Some((etag_cached, output_cached)) = cache_get(&key, ttl) {
            if args.etag.as_deref() == Some(&etag_cached) {
                return Ok(Json(serde_json::json!({"status":"not_modified","etag": etag_cached})));
            } else {
                return Ok(Json(serde_json::json!({"status":"ok","etag": etag_cached, "json": serde_json::from_str::<serde_json::Value>(&output_cached).unwrap_or(serde_json::json!({})) })));
            }
        }
    }
    let timeout = Duration::from_millis(env_timeout_ms());
    let delay = env_test_delay_ms();
    let path_clone = abspath.clone();
    let topn = args.top_n;
    let handle = tokio::task::spawn_blocking(move || {
        if let Some(ms) = delay { thread::sleep(Duration::from_millis(ms)); }
        let graph = build_graph_for_path(path_clone.to_string_lossy().as_ref())?;
        let exporter = archlens::exporter::Exporter::new();
        let mut json = exporter.export_to_ai_summary_json(&graph).map_err(|e| e.to_string())?;
        json = trim_ai_summary_json(json, topn);
        let txt = serde_json::to_string_pretty(&json).unwrap_or("{}".into());
        Ok::<serde_json::Value, String>(json)
    });
    let out = match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(val)) => {
                let txt = serde_json::to_string_pretty(&val).unwrap_or("{}".into());
                let etag = content_etag(&txt);
                if args.use_cache.unwrap_or(true) { cache_put(&key, &etag, &txt); }
                if args.etag.as_deref() == Some(&etag) {
                    Ok(serde_json::json!({"status":"not_modified","etag": etag})))
                } else {
                    let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                    Ok(serde_json::json!({"status":"ok","etag": etag, "json": serde_json::from_str::<serde_json::Value>(&txt).unwrap_or(val)})))
                }
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    };
    tracing::info!(target: "archlens_mcp", action = "export_ai_summary_json", ms = %t0.elapsed().as_millis());
    out
}

async fn post_structure(Json(args): Json<StructureArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let t0 = Instant::now();
    let path = ensure_absolute_path(&args.project_path);
    let lv = level(&args.detail_level).to_string();
    let timeout = Duration::from_millis(env_timeout_ms());
    let delay = env_test_delay_ms();
    let handle = tokio::task::spawn_blocking(move || {
        if let Some(ms) = delay { thread::sleep(Duration::from_millis(ms)); }
        stats::get_project_structure(path.to_string_lossy().as_ref())
    });
    let out = match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(structure)) => {
                let text = format_structure_result(path.to_string_lossy().as_ref(), &structure, &lv);
                let text = clamp_text_with_limit(&text, args.max_output_chars);
                let etag = content_etag(&text);
                if args.etag.as_deref() == Some(&etag) {
                    Ok(serde_json::json!({"status":"not_modified","etag": etag})))
                } else {
                    Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                }
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    };
    tracing::info!(target: "archlens_mcp", action = "structure_get", ms = %t0.elapsed().as_millis());
    out
}

async fn post_diagram(Json(args): Json<DiagramArgs>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let t0 = Instant::now();
    let path = ensure_absolute_path(&args.project_path);
    let kind = args.diagram_type.as_deref().unwrap_or("mermaid");
    if kind != "mermaid" { return Err(axum::http::StatusCode::BAD_REQUEST); }
    let lv = level(&args.detail_level).to_string();
    let timeout = Duration::from_millis(env_timeout_ms());
    let delay = env_test_delay_ms();
    let p = path.clone();
    let handle = tokio::task::spawn_blocking(move || {
        if let Some(ms) = delay { thread::sleep(Duration::from_millis(ms)); }
        cli::handlers::build_graph_mermaid(p.to_string_lossy().as_ref())
            .or_else(|_| diagram::generate_mermaid_diagram(p.to_string_lossy().as_ref()))?;
    });
    let out = match tokio::time::timeout(timeout, handle).await {
        Ok(joined) => match joined {
            Ok(Ok(mmd)) => {
                let text = format_diagram_text(mmd, path.to_string_lossy().as_ref(), &lv);
                let text = clamp_text_with_limit(&text, args.max_output_chars);
                let etag = content_etag(&text);
                if args.etag.as_deref() == Some(&etag) {
                    Ok(serde_json::json!({"status":"not_modified","etag": etag})))
                } else {
                    Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                }
            },
            _ => Err(axum::http::StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(axum::http::StatusCode::REQUEST_TIMEOUT)
    };
    tracing::info!(target: "archlens_mcp", action = "diagram_generate", ms = %t0.elapsed().as_millis());
    out
}

async fn get_schemas() -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let list = list_schema_resources();
    Ok(Json(serde_json::json!({"resources": list})))
}

async fn get_presets() -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let resources = list_schema_resources().into_iter().filter(|r| r.description.as_deref() == Some("AI preset (recommended tool args)")).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({"resources": resources})))
}

async fn get_recommendations(Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let project_path = payload.get("project_path").and_then(|v| v.as_str()).unwrap_or(".");
    let json_opt = payload.get("json");
    let focus_opt = payload.get("focus").and_then(|v| v.as_str());
    let result = compute_recommendations(project_path, json_opt, focus_opt);
    Ok(Json(result))
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
    let ai_summary_schema = schemars::schema_for!(AISummaryArgs);
    let ai_recommend_schema = schemars::schema_for!(AIRecommendArgs);

    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let schemas_dir = root.join("out").join("schemas");
    let to_uri = |name: &str| -> Option<String> {
        let p = schemas_dir.join(format!("{}.schema.json", name));
        let abs = p.canonicalize().ok()?;
        Some(format!("file://{}", abs.to_string_lossy()))
    };

    vec![
        ToolDescription { name: "arch.refresh".into(), description: "Refresh analysis context (noop placeholder)".into(), input_schema: serde_json::json!({"type":"object"}), schema_uri: None },
        ToolDescription { name: "graph.build".into(), description: "Build architecture diagram (mermaid)".into(), input_schema: serde_json::to_value(diagram_schema.schema).unwrap(), schema_uri: to_uri("diagram_args") },
        ToolDescription { name: "export.ai_compact".into(), description: "Export AI compact analysis".into(), input_schema: serde_json::to_value(export_schema.schema).unwrap(), schema_uri: to_uri("export_args") },
        ToolDescription { name: "export.ai_summary_json".into(), description: "Export minimal structured JSON summary for AI (facts only).".into(), input_schema: serde_json::to_value(ai_summary_schema.schema).unwrap(), schema_uri: to_uri("ai_summary_args") },
        ToolDescription { name: "structure.get".into(), description: "Get project structure".into(), input_schema: serde_json::to_value(structure_schema.schema).unwrap(), schema_uri: to_uri("structure_args") },
        ToolDescription { name: "analyze.project".into(), description: "Analyze project (shallow or deep)".into(), input_schema: serde_json::to_value(analyze_schema.schema).unwrap(), schema_uri: to_uri("analyze_args") },
        ToolDescription { name: "ai.recommend".into(), description: "Suggest next best MCP calls based on ai_summary_json.".into(), input_schema: serde_json::to_value(ai_recommend_schema.schema).unwrap(), schema_uri: to_uri("ai_recommend_args") },
    ]
}

fn content_etag(s: &str) -> String {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn cache_dir() -> PathBuf { std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("out").join("cache") }

fn env_cache_max_entries() -> Option<usize> {
    std::env::var("ARCHLENS_CACHE_MAX_ENTRIES").ok().and_then(|s| s.parse().ok()).filter(|v| *v > 0)
}
fn env_cache_max_bytes() -> Option<u64> {
    std::env::var("ARCHLENS_CACHE_MAX_BYTES").ok().and_then(|s| s.parse().ok()).filter(|v| *v > 0)
}

fn cache_trim_lru(dir: &Path, max_entries: Option<usize>, max_bytes: Option<u64>) {
    if max_entries.is_none() && max_bytes.is_none() { return; }
    let rd = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    let mut files: Vec<(PathBuf, u64, u64)> = Vec::new(); // (path, mtime_sec, size)
    for ent in rd.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
        if let Ok(meta) = ent.metadata() {
            if meta.is_file() {
                let size = meta.len();
                let mtime = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                files.push((p, mtime, size));
            }
        }
    }
    if files.is_empty() { return; }
    // Sort by mtime asc (oldest first)
    files.sort_by_key(|(_, m, _)| *m);
    let mut total_bytes: u64 = files.iter().map(|(_,_,s)| *s).sum();
    let mut total_entries: usize = files.len();
    let target_entries = max_entries.unwrap_or(usize::MAX);
    let target_bytes = max_bytes.unwrap_or(u64::MAX);
    let mut i = 0usize;
    while (total_entries > target_entries) || (total_bytes > target_bytes) {
        if i >= files.len() { break; }
        let (p, _m, sz) = &files[i];
        let _ = fs::remove_file(p);
        total_entries = total_entries.saturating_sub(1);
        total_bytes = total_bytes.saturating_sub(*sz);
        i += 1;
    }
}

// Project content fingerprint for cache invalidation
fn git_head_and_dirty(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", path.to_string_lossy().as_ref(), "rev-parse", "HEAD"]) 
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let mut head = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if head.is_empty() { return None; }
    // dirty state
    if let Ok(status_out) = Command::new("git")
        .args(["-C", path.to_string_lossy().as_ref(), "status", "--porcelain", "-uno"]) 
        .output() {
        if status_out.status.success() && !status_out.stdout.is_empty() {
            head.push_str("-dirty");
        }
    }
    Some(head)
}

fn fs_dir_fingerprint(path: &Path) -> String {
    // Shallow, fast, ignores common build dirs
    let mut total_files: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut max_mtime: u64 = 0;

    fn is_ignored(p: &Path) -> bool {
        let ignored = [".git", "target", "node_modules", "dist", "build", ".next", ".venv", "venv"];
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|name| ignored.iter().any(|ig| name.eq_ignore_ascii_case(ig)))
            .unwrap_or(false)
    }

    fn walk(dir: &Path, total_files: &mut u64, total_bytes: &mut u64, max_mtime: &mut u64) {
        if is_ignored(dir) { return; }
        if let Ok(rd) = fs::read_dir(dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                if is_ignored(&p) { continue; }
                if let Ok(meta) = ent.metadata() {
                    if meta.is_dir() {
                        walk(&p, total_files, total_bytes, max_mtime);
                    } else if meta.is_file() {
                        *total_files += 1;
                        *total_bytes = total_bytes.saturating_add(meta.len());
                        if let Ok(modified) = meta.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                // Convert to a decreasing value; use now - elapsed
                                let now = std::time::SystemTime::now();
                                if let Ok(nowdur) = now.duration_since(std::time::UNIX_EPOCH) {
                                    let mtime = nowdur.saturating_sub(elapsed).as_secs();
                                    if mtime > *max_mtime { *max_mtime = mtime; }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    walk(path, &mut total_files, &mut total_bytes, &mut max_mtime);
    let mut hasher = DefaultHasher::new();
    total_files.hash(&mut hasher);
    total_bytes.hash(&mut hasher);
    max_mtime.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn project_content_fingerprint(path: &Path) -> String {
    if let Some(head) = git_head_and_dirty(path) {
        return head;
    }
    fs_dir_fingerprint(path)
}

fn export_cache_key(path: &str, lv: &str, sections: &Option<Vec<String>>, top_n: Option<usize>, max_chars: Option<usize>) -> String {
    let mut elems = vec![path.to_string(), lv.to_string(), format!("top_n={}", top_n.unwrap_or(0)), format!("max={}", max_chars.unwrap_or(0))];
    if let Some(s) = sections {
        let mut s2 = s.clone();
        s2.sort();
        elems.push(format!("sections={}", s2.join("|")));
    }
    // include project fingerprint
    let fp = project_content_fingerprint(Path::new(path));
    elems.push(format!("fp={}", fp));
    let joined = elems.join("::");
    let mut h = DefaultHasher::new();
    joined.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn cache_get(key: &str, ttl_ms: u64) -> Option<(String, String)> {
    let dir = cache_dir();
    let p = dir.join(format!("{}.json", key));
    let meta = fs::metadata(&p).ok()?;
    let age = meta.modified().ok()?.elapsed().ok()?.as_millis() as u64;
    if age > ttl_ms { return None; }
    let mut f = fs::File::open(&p).ok()?;
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf).ok()?;
    let v: serde_json::Value = serde_json::from_str(&buf).ok()?;
    let etag = v.get("etag")?.as_str()?.to_string();
    let output = v.get("output")?.as_str()?.to_string();
    Some((etag, output))
}

fn cache_put(key: &str, etag: &str, output: &str) {
    let dir = cache_dir();
    let _ = fs::create_dir_all(&dir);
    let p = dir.join(format!("{}.json", key));
    let _ = fs::write(&p, serde_json::json!({"etag":etag,"output":output}).to_string());
    // LRU eviction
    cache_trim_lru(&dir, env_cache_max_entries(), env_cache_max_bytes());
}

fn presets_dir() -> PathBuf { std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("out").join("presets") }

fn write_preset(name: &str, json: serde_json::Value) {
    let dir = presets_dir();
    let _ = fs::create_dir_all(&dir);
    let p = dir.join(format!("{}.json", name));
    let _ = fs::write(p, serde_json::to_vec_pretty(&json).unwrap());
}

fn build_graph_for_path(project_path: &str) -> Result<archlens::types::CapsuleGraph, String> {
    use archlens::file_scanner::FileScanner;
    use archlens::parser_ast::ParserAST;
    use archlens::capsule_constructor::CapsuleConstructor;
    use archlens::capsule_graph_builder::CapsuleGraphBuilder;
    use archlens::validator_optimizer::ValidatorOptimizer;
    use archlens::types::{Capsule, Result as ArchResult};
    use std::path::Path;

    let scanner = FileScanner::new(
        vec!["**/*.rs".into(), "**/*.ts".into(), "**/*.js".into(), "**/*.py".into(), "**/*.java".into(), "**/*.go".into(), "**/*.cpp".into(), "**/*.c".into()],
        vec!["**/target/**".into(), "**/node_modules/**".into(), "**/.git/**".into(), "**/dist/**".into(), "**/build/**".into()],
        Some(8),
    ).map_err(|e| e.to_string())?;
    let files = scanner.scan_files(Path::new(project_path)).map_err(|e| e.to_string())?;

    let mut parser = ParserAST::new().map_err(|e| e.to_string())?;
    let constructor = CapsuleConstructor::new();
    let mut capsules: Vec<Capsule> = Vec::new();
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(&file.path) {
            if let Ok(nodes) = parser.parse_file(&file.path, &content, &file.file_type) {
                let mut caps = constructor.create_capsules(&nodes, &file.path.clone()).map_err(|e| e.to_string())?;
                capsules.append(&mut caps);
            }
        }
    }
    if capsules.is_empty() { return Err("No capsules".into()); }
    let mut builder = CapsuleGraphBuilder::new();
    let graph = builder.build_graph(&capsules).map_err(|e| e.to_string())?;
    let validator = ValidatorOptimizer::new();
    let graph = validator.validate_and_optimize(&graph).map_err(|e| e.to_string())?;
    Ok(graph)
}

fn trim_ai_summary_json(mut v: serde_json::Value, top_n: Option<usize>) -> serde_json::Value {
    let n = top_n.unwrap_or(0);
    if n == 0 { return v; }
    if let Some(arr) = v.get_mut("problems_validated").and_then(|x| x.as_array_mut()) { if arr.len() > n { arr.truncate(n); } }
    if let Some(arr) = v.get_mut("cycles_top").and_then(|x| x.as_array_mut()) { if arr.len() > n { arr.truncate(n); } }
    if let Some(arr) = v.get_mut("top_coupling").and_then(|x| x.as_array_mut()) { if arr.len() > n { arr.truncate(n); } }
    if let Some(arr) = v.get_mut("top_complexity_components").and_then(|x| x.as_array_mut()) { if arr.len() > n { arr.truncate(n); } }
    v
}

fn list_schema_resources() -> Vec<ResourceDescription> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let schemas = root.join("out").join("schemas");
    let presets = root.join("out").join("presets");
    let mut resources = Vec::new();
    for dir in [schemas, presets] {
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p.extension().and_then(|e| e.to_str()) == Some("json") || p.extension().and_then(|e| e.to_str()) == Some("schema.json") {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let abs = p.canonicalize().unwrap_or(p.clone());
                    let uri = format!("file://{}", abs.to_string_lossy());
                    let mime = if name.ends_with(".schema.json") { Some("application/schema+json".into()) } else { Some("application/json".into()) };
                    let description = if dir.ends_with("presets") { Some("AI preset (recommended tool args)".into()) } else { Some("JSON Schema".into()) };
                    resources.push(ResourceDescription { name, uri, mime, description });
                }
            }
        }
    }
    resources
}

fn read_resource_uri(uri: &str) -> Result<(String, String), String> {
    if let Some(path) = uri.strip_prefix("file://") {
        let p = PathBuf::from(path);
        let text = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let mime = if p.extension().and_then(|e| e.to_str()) == Some("json") { "application/json" } else if p.extension().and_then(|e| e.to_str()) == Some("json") { "application/json" } else if p.extension().and_then(|e| e.to_str()) == Some("schema.json") { "application/schema+json" } else { "text/plain" };
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
        },
        PromptDescription {
            name: "ai.health_check".into(),
            description: "Ask for a concise health check of the architecture with top risks and quick wins.".into(),
            messages: vec![ PromptMessage{ role: "system".into(), content: "Be concise, focus on risks and quick wins.".into() },
                            PromptMessage{ role: "user".into(), content: "Provide a compact health check for the project using the provided analysis.".into() } ]
        },
        PromptDescription {
            name: "ai.refactor.plan".into(),
            description: "Produce a prioritized refactoring plan given the problems.".into(),
            messages: vec![ PromptMessage{ role: "system".into(), content: "Provide a pragmatic, risk-aware refactoring plan.".into() },
                            PromptMessage{ role: "user".into(), content: "Suggest a prioritized refactoring plan for the issues found.".into() } ]
        },
        PromptDescription {
            name: "ai.next_steps".into(),
            description: "Given an analysis snippet, propose the next best MCP calls with arguments to drill down efficiently.".into(),
            messages: vec![ PromptMessage{ role: "system".into(), content: "Return a minimal set of next MCP calls with arguments to maximize signal, minimize tokens.".into() },
                            PromptMessage{ role: "user".into(), content: "Review this analysis and recommend next MCP calls (tools and args) to investigate critical risks first.".into() } ]
        }
    ]
}

fn get_prompt_by_name(name: &str) -> Option<PromptDescription> {
    list_prompts().into_iter().find(|p| p.name == name)
}

fn compute_recommendations(project_path: &str, json_opt: Option<&serde_json::Value>, focus_opt: Option<&str>) -> serde_json::Value {
    compute_recommendations_with_thresholds(project_path, json_opt, focus_opt, &reco_thresholds_from_env())
}

fn compute_recommendations_with_thresholds(project_path: &str, json_opt: Option<&serde_json::Value>, focus_opt: Option<&str>, th: &RecoThresholds) -> serde_json::Value {
    let mut recs: Vec<serde_json::Value> = Vec::new();
    let focus = focus_opt.unwrap_or("");
    let json = json_opt.cloned().unwrap_or(serde_json::json!({}));
    if json.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        recs.push(serde_json::json!({
            "tool":"export.ai_summary_json",
            "arguments": {"project_path": project_path, "top_n": 5, "max_output_chars": 20000, "use_cache": true},
            "why": "Start with minimal structured facts (summary/problems/cycles) to minimize tokens."
        }));
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary","sections":["summary","problems_validated","cycles"], "top_n": 5, "max_output_chars": 15000, "use_cache": true},
            "why": "Compact markdown view for human-readable summary if needed."
        }));
        return serde_json::json!({"status":"ok","recommendations": recs});
    }
    let cycles_count = json.get("cycles_top").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let top_coupling = json.get("top_coupling").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let top_coupling_len = top_coupling.len();
    let problems = json.get("problems_validated").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let high_sev = problems.iter().any(|p| p.get("severity").and_then(|s| s.get("H")).and_then(|h| h.as_u64()).unwrap_or(0) > 0);
    let high_sev_cats = problems.iter().filter(|p| p.get("severity").and_then(|s| s.get("H")).and_then(|h| h.as_u64()).unwrap_or(0) > 0).count();
    let summary = json.get("summary").cloned().unwrap_or(serde_json::json!({}));
    let complexity_avg = summary.get("complexity_avg").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let coupling_index = summary.get("coupling_index").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let cohesion_index = summary.get("cohesion_index").and_then(|x| x.as_f64()).unwrap_or(1.0);
    let components_total = summary.get("components").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    let layers = summary.get("layers").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let max_layer = layers.iter().filter_map(|l| l.get("count").and_then(|c| c.as_u64())).max().unwrap_or(0) as usize;

    if cycles_count > 0 {
        recs.push(serde_json::json!({
            "tool":"graph.build",
            "arguments": {"project_path": project_path, "detail_level":"summary","max_output_chars": 15000},
            "why": "Detected cycles; a mermaid graph helps visualize and locate problematic dependencies quickly."
        }));
    }
    if high_sev {
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary","sections":["problems_validated"], "top_n": 10, "max_output_chars": 20000, "use_cache": true},
            "why": "High-severity validator problems present; drill down into categories and top impacted components."
        }));
    }
    // Layer imbalance
    if components_total > 0 && (max_layer * 100 / components_total) as u8 >= th.layer_imbalance_pct {
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary","sections":["layers","problems_validated"], "max_output_chars": 14000, "use_cache": true},
            "why": "Layer imbalance detected; review layer distribution and associated problems."
        }));
    }
    if coupling_index > th.coupling_index || cohesion_index < th.cohesion_index || top_coupling_len > 0 {
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary","sections":["cycles","top_coupling"], "top_n": 10, "max_output_chars": 18000, "use_cache": true},
            "why": "Coupling issues indicated; review cycles and top hub components to target decoupling."
        }));
    }
    if complexity_avg > th.complexity_avg {
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary","sections":["top_complexity_components"], "top_n": 10, "max_output_chars": 16000, "use_cache": true},
            "why": "High average complexity; inspect top complex components for refactoring opportunities."
        }));
    }
    if high_sev_cats >= th.high_sev_cats {
        recs.push(serde_json::json!({
            "prompt":"ai.refactor.plan",
            "why": "Multiple high-severity categories detected; propose a pragmatic, risk-aware refactoring plan."
        }));
    }
    if !focus.is_empty() {
        let preset = if focus.contains("cycle") { "cycles_focus" } else if focus.contains("plan") { "refactor_plan" } else { "health_check" };
        recs.push(serde_json::json!({
            "tool":"export.ai_compact",
            "arguments": {"project_path": project_path, "detail_level":"summary"},
            "why": format!("User focus suggests preset '{}'.", preset)
        }));
    }
    if recs.is_empty() {
        recs.push(serde_json::json!({
            "tool":"export.ai_summary_json",
            "arguments": {"project_path": project_path, "top_n": 5, "max_output_chars": 20000, "use_cache": true},
            "why": "Fallback to structured summary to guide further steps."
        }));
    }
    serde_json::json!({"status":"ok","recommendations": recs})
}

#[cfg(test)]
mod tests {
    use super::compute_recommendations;
    use super::compute_recommendations_with_thresholds;
    use super::RecoThresholds;
    use super::export_cache_key;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn tools_from(rec: &serde_json::Value) -> Vec<String> {
        rec.get("recommendations")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| r.get("tool").and_then(|t| t.as_str()).map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn recommend_starts_with_summary_when_no_json() {
        let res = compute_recommendations(".", None, None);
        let tools = tools_from(&res);
        assert!(tools.iter().any(|t| t == "export.ai_summary_json"));
    }

    #[test]
    fn recommend_graph_build_when_cycles_present() {
        let mock = json!({
            "summary": {"complexity_avg": 5.0, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [{"path":["A","B","A"]}],
            "problems_validated": []
        });
        let res = compute_recommendations(".", Some(&mock), None);
        let tools = tools_from(&res);
        assert!(tools.iter().any(|t| t == "graph.build"));
    }

    #[test]
    fn recommend_problems_validated_when_high_severity() {
        let mock = json!({
            "summary": {"complexity_avg": 5.0, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [],
            "problems_validated": [{"category":"complexity","count":5,"severity":{"H":1,"M":0,"L":0},"top_components":["X"],"hint":"reduce complexity"}]
        });
        let res = compute_recommendations(".", Some(&mock), None);
        let tools = tools_from(&res);
        assert!(tools.iter().any(|t| t == "export.ai_compact"));
    }

    #[test]
    fn recommend_top_complexity_when_high_avg() {
        let mock = json!({
            "summary": {"complexity_avg": 12.0, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [],
            "problems_validated": []
        });
        let res = compute_recommendations(".", Some(&mock), None);
        let tools = tools_from(&res);
        assert!(tools.iter().any(|t| t == "export.ai_compact"));
    }

    #[test]
    fn recommend_top_complexity_when_high_avg_with_custom_threshold() {
        let mock = json!({
            "summary": {"complexity_avg": 6.5, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [],
            "problems_validated": []
        });
        let th = RecoThresholds { complexity_avg: 6.0, coupling_index: 0.7, cohesion_index: 0.3, layer_imbalance_pct: 60, high_sev_cats: 2 };
        let res = compute_recommendations_with_thresholds(".", Some(&mock), None, &th);
        let tools = tools_from(&res);
        assert!(tools.iter().any(|t| t == "export.ai_compact"));
    }

    #[test]
    fn recommend_layer_imbalance_triggers_layers_section() {
        let mock = json!({
            "summary": {"components": 10, "layers": [
                {"name":"Core","count":6}, {"name":"Infra","count":4}
            ], "complexity_avg": 3.0, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [],
            "problems_validated": []
        });
        let res = compute_recommendations(".", Some(&mock), None);
        let recs = res.get("recommendations").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let mut has_layers = false;
        for r in recs {
            if r.get("tool").and_then(|t| t.as_str()) == Some("export.ai_compact") {
                if let Some(args) = r.get("arguments").and_then(|a| a.as_object()) {
                    if let Some(sections) = args.get("sections").and_then(|s| s.as_array()) {
                        if sections.iter().any(|s| s.as_str() == Some("layers")) { has_layers = true; break; }
                    }
                }
            }
        }
        assert!(has_layers, "expected layers section recommendation when imbalance >= threshold");
    }

    #[test]
    fn recommend_prompt_refactor_when_many_high_severity_categories() {
        let mock = json!({
            "summary": {"components": 5, "layers": [], "complexity_avg": 5.0, "coupling_index": 0.2, "cohesion_index": 0.8},
            "cycles_top": [],
            "problems_validated": [
                {"category":"complexity","count":3,"severity":{"H":1,"M":0,"L":0}},
                {"category":"coupling","count":2,"severity":{"H":1,"M":0,"L":0}}
            ]
        });
        let res = compute_recommendations(".", Some(&mock), None);
        let recs = res.get("recommendations").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let has_prompt = recs.iter().any(|r| r.get("prompt").and_then(|p| p.as_str()) == Some("ai.refactor.plan"));
        assert!(has_prompt, "expected ai.refactor.plan prompt when multiple high-severity categories");
    }

    #[test]
    fn cache_key_changes_on_fs_change() {
        let dir = PathBuf::from("out/test_cache_tmp");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // initial file
        fs::write(dir.join("a.txt"), b"hello").unwrap();
        let p = dir.canonicalize().unwrap();
        let k1 = super::export_cache_key(p.to_string_lossy().as_ref(), "summary", &None, Some(5), Some(12345));
        // change FS: add file to ensure different fingerprint
        fs::write(dir.join("b.txt"), b"world!!! world!!!").unwrap();
        let k2 = super::export_cache_key(p.to_string_lossy().as_ref(), "summary", &None, Some(5), Some(12345));
        assert_ne!(k1, k2, "cache key must change when project content changes");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_lru_trims_to_max_entries() {
        let dir = PathBuf::from("out/test_cache_lru_entries");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Create 3 files
        for i in 0..3 {
            let p = dir.join(format!("{}.json", i));
            fs::write(&p, format!("{{\"etag\":\"e{}\",\"output\":\"{}\"}}", i, "x".repeat(10))).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        super::cache_trim_lru(&dir, Some(2), None);
        let count = fs::read_dir(&dir).unwrap().flatten().count();
        assert!(count <= 2, "LRU should trim to 2 entries or fewer");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_lru_trims_to_max_bytes() {
        let dir = PathBuf::from("out/test_cache_lru_bytes");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Create files ~30B each
        for i in 0..4 {
            let p = dir.join(format!("{}.json", i));
            fs::write(&p, format!("{{\"etag\":\"e{}\",\"output\":\"{}\"}}", i, "y".repeat(30))).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        // 4*~30B ≈ 120B; trim to <= 70B
        super::cache_trim_lru(&dir, None, Some(70));
        let mut total: u64 = 0;
        for ent in fs::read_dir(&dir).unwrap().flatten() { total += ent.metadata().unwrap().len(); }
        assert!(total <= 80, "LRU should trim total bytes to the target");
        let _ = fs::remove_dir_all(&dir);
    }
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
                    let abspath = ensure_absolute_path(args.project_path);
                    let lv = level(&args.detail_level).to_string();
                    let use_cache = args.use_cache.unwrap_or(true) && !args.force.unwrap_or(false);
                    let ttl = args.cache_ttl_ms.unwrap_or_else(env_cache_ttl_ms);
                    let key = export_cache_key(&abspath.to_string_lossy(), &lv, &args.sections, args.top_n, args.max_output_chars);

                    if use_cache {
                        if let Some((etag_cached, output_cached)) = cache_get(&key, ttl) {
                            if args.etag.as_deref() == Some(&etag_cached) {
                                return Ok(serde_json::json!({"status":"not_modified","etag": etag_cached}));
                            } else {
                                return Ok(serde_json::json!({"status":"ok","etag": etag_cached, "content":[{"type":"text","text": output_cached}]}));
                            }
                        }
                    }

                    let out = export::generate_ai_compact(abspath.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    let txt = format_export_markdown_with_controls(out, level(&args.detail_level), &args.sections, args.top_n, args.max_output_chars);
                    let etag = content_etag(&txt);
                    if args.use_cache.unwrap_or(true) { cache_put(&key, &etag, &txt); }
                    if args.etag.as_deref() == Some(&etag) {
                        Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                    } else {
                        Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                    }
                }
                "export.ai_summary_json" => {
                    let args: AISummaryArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let abspath = ensure_absolute_path(args.project_path);
                    let use_cache = args.use_cache.unwrap_or(true) && !args.force.unwrap_or(false);
                    let ttl = args.cache_ttl_ms.unwrap_or_else(env_cache_ttl_ms);
                    let key = export_cache_key(&abspath.to_string_lossy(), "json_summary", &Some(vec!["__json_summary__".into()]), args.top_n, args.max_output_chars);
                    if use_cache {
                        if let Some((etag_cached, output_cached)) = cache_get(&key, ttl) {
                            if args.etag.as_deref() == Some(&etag_cached) {
                                return Ok(serde_json::json!({"status":"not_modified","etag": etag_cached}));
                            } else {
                                return Ok(serde_json::json!({"status":"ok","etag": etag_cached, "content":[{"type":"text","text": output_cached}]}));
                            }
                        }
                    }
                    let graph = build_graph_for_path(abspath.to_string_lossy().as_ref())?;
                    let exporter = archlens::exporter::Exporter::new();
                    let mut json = exporter.export_to_ai_summary_json(&graph).map_err(|e| e.to_string())?;
                    json = trim_ai_summary_json(json, args.top_n);
                    let txt = serde_json::to_string_pretty(&json).unwrap_or("{}".into());
                    let etag = content_etag(&txt);
                    if args.use_cache.unwrap_or(true) { cache_put(&key, &etag, &txt); }
                    if args.etag.as_deref() == Some(&etag) {
                        Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                    } else {
                        let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                        Ok(serde_json::json!({"status":"ok","etag": etag, "json": serde_json::from_str::<serde_json::Value>(&txt).unwrap_or(json)}))
                    }
                }
                "structure.get" => {
                    let args: StructureArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let st = stats::get_project_structure(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                    let txt = format_structure_result(path.to_string_lossy().as_ref(), &st, level(&args.detail_level));
                    let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                    let etag = content_etag(&txt);
                    if args.etag.as_deref() == Some(&etag) {
                        Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                    } else {
                        Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                    }
                }
                "graph.build" => {
                    let args: DiagramArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    let mmd = cli::handlers::build_graph_mermaid(path.to_string_lossy().as_ref())
                        .or_else(|_| diagram::generate_mermaid_diagram(path.to_string_lossy().as_ref()))?;
                    let txt = format_diagram_text(mmd, path.to_string_lossy().as_ref(), level(&args.detail_level));
                    let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                    let etag = content_etag(&txt);
                    if args.etag.as_deref() == Some(&etag) {
                        Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                    } else {
                        Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                    }
                }
                "analyze.project" => {
                    let args: AnalyzeArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let path = ensure_absolute_path(args.project_path);
                    if args.deep.unwrap_or(false) {
                        let res = cli::handlers::run_deep_pipeline(path.to_string_lossy().as_ref())
                            .map_err(|e| e.to_string())?;
                        let lv = level(&args.detail_level);
                        let txt = clamp_text(&res, if lv=="full" { MAX_OUTPUT_CHARS } else if lv=="standard" { SUMMARY_LIMIT_CHARS * 2 } else { SUMMARY_LIMIT_CHARS });
                        let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                        let etag = content_etag(&txt);
                        if args.etag.as_deref() == Some(&etag) {
                            Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                        } else {
                            Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                        }
                    } else {
                        let s = stats::get_project_stats(path.to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
                        let lv = level(&args.detail_level);
                        let txt = format_analysis_result(path.to_string_lossy().as_ref(), &s, lv);
                        let txt = clamp_text_with_limit(&txt, args.max_output_chars);
                        let etag = content_etag(&txt);
                        if args.etag.as_deref() == Some(&etag) {
                            Ok(serde_json::json!({"status":"not_modified","etag": etag}))
                        } else {
                            Ok(serde_json::json!({"status":"ok","etag": etag, "content":[{"type":"text","text": txt}]}))
                        }
                    }
                }
                "arch.refresh" => {
                    Ok(serde_json::json!({"content":[{"type":"text","text": "ok"}]}))
                }
                "ai.recommend" => {
                    let args: AIRecommendArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;
                    let result = compute_recommendations(&args.project_path, args.json.as_ref(), args.focus.as_deref());
                    Ok(result)
                }
                _ => Err(format!("unknown tool: {}", name)),
            }
        }
        _ => Err("unknown method".into())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing (env-controlled)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .try_init();

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
    write_schema("ai_summary_args", schemars::schema_for!(AISummaryArgs));
    write_schema("resource_read_args", schemars::schema_for!(ResourceReadArgs));
    write_schema("ai_recommend_args", schemars::schema_for!(AIRecommendArgs));
    write_schema("prompt_get_args", schemars::schema_for!(PromptGetArgs));
    // Output models
    write_schema("model_project_stats", schemars::schema_for!(stats::ProjectStats));
    write_schema("model_project_structure", schemars::schema_for!(stats::ProjectStructure));
    // Presets (for AI agents)
    write_preset("health_check", serde_json::json!({
        "tool": "export.ai_compact",
        "arguments": {"detail_level":"summary","sections":["summary","problems_validated","cycles"], "top_n": 5, "max_output_chars": 15000}
    }));
    write_preset("cycles_focus", serde_json::json!({
        "tool": "export.ai_compact",
        "arguments": {"detail_level":"summary","sections":["cycles","top_coupling"], "top_n": 10, "max_output_chars": 20000}
    }));
    write_preset("refactor_plan", serde_json::json!({
        "tool": "export.ai_compact",
        "arguments": {"detail_level":"standard","sections":["summary","problems_validated","top_complexity_components"], "top_n": 10, "max_output_chars": 30000}
    }));

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
                                    let delay = env_test_delay_ms();
                                    let handle = tokio::task::spawn_blocking(move || {
                                        if let Some(ms) = delay { thread::sleep(Duration::from_millis(ms)); }
                                        handle_call(&method, pclone)
                                    });
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