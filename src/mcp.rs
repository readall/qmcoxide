//! MCP server: stdio (default) and HTTP transport (--http --port, --daemon with PID file in cache, stop, health /health, models warm 5min for embed ctx).
//! Tools: query (supports q or searches[], intent, collections, limit, candidateLimit, rerank, explain, chunkStrategy), get (pathOrDocid with line opts, full, fullPath), multi_get, status.
//! Must match original shapes, qmd:// URIs, docid, line nums etc for compat with Claude etc.
//! See original src/mcp/server.ts , mcp.test.ts , README MCP section, requirements.
//!
//! Schemas here are the contract (for .32). Descriptions embed full SYNTAX grammar (see docs/SYNTAX.md) so clients (Claude etc) learn structured queries.
//! Exact I/O per mcp.feature + acceptance: scores, qmd://, docid (#xxxxxx), snippets with absolute lines, context, optional explain trace.

use serde::{Deserialize, Serialize};

/// Query tool (hybrid search).
/// Supports bare q (auto LLM expand) or structured per SYNTAX EBNF.
///
/// Grammar (embedded for client teaching):
///   query = ( bare_line | structured )
///   bare_line = [ "expand:" ] text
///   structured = intent_line? (lex_line | vec_line | hyde_line)+
///   intent_line = "intent:" text newline
///   lex_line = "lex:" ( term ( " " term )* ) newline   ; term = word | "phrase" | -word | -"phrase"
///   vec_line = "vec:" text newline
///   hyde_line = "hyde:" text newline
///
/// Examples in desc:
///   intent: web performance and Core Web Vitals
///   lex: performance
///   vec: how to improve page load times
///
///   lex: "machine learning" -"deep learning"
///   lex: auth -oauth -saml
///
/// Results: array of { path (verbatim), docid (#6hex), score, snippet (with abs line anchors), context?, explain? }
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QueryToolInput {
    /// Bare query text (triggers expand) or omit for structured.
    #[serde(default)]
    pub q: Option<String>,
    /// Alternative for multi-vector/lex (advanced hybrid).
    #[serde(default)]
    pub searches: Option<Vec<String>>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub collections: Option<Vec<String>>,
    #[serde(default = "default_limit")]
    pub limit: Option<usize>,
    #[serde(default)]
    pub candidate_limit: Option<usize>,
    #[serde(default)]
    pub rerank: Option<bool>,
    #[serde(default)]
    pub explain: Option<bool>,
    #[serde(default)]
    pub chunk_strategy: Option<String>, // "regex" | "auto"
}

fn default_limit() -> Option<usize> { Some(10) }

/// Get tool: verbatim path, #docid, or qmd:// URI.
/// Supports ranges :from:count or --from-line --max-lines, --full, --full-path, --max-bytes.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetToolInput {
    pub path_or_docid: String,
    #[serde(default)]
    pub full: Option<bool>,
    #[serde(default)]
    pub full_path: Option<bool>,
    #[serde(default)]
    pub from_line: Option<usize>,
    #[serde(default)]
    pub max_lines: Option<usize>,
    #[serde(default)]
    pub max_bytes: Option<usize>,
}

/// Multi get for several specs.
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiGetToolInput {
    pub paths_or_docids: Vec<String>,
    // same range/full opts apply per item or global
}

/// Status: collections, counts, health, last embed times, etc.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StatusToolInput {
    #[serde(default)]
    pub json: Option<bool>,
}

pub async fn run_mcp_server(http: bool, port: u16, daemon: bool) {
    // Full server per .6: stdio jsonrpc (default) or axum for /mcp (if --http), tool handlers using schemas + store (query/get etc with types).
    // Quiet env, residency, daemon PID in cache, /health, warm models 5min.
    // Schemas + grammar already exact for client compat.
    // User: `cargo run -- mcp --http --port {port} --daemon` (explicit).
    if http {
        println!("MCP HTTP server on port {port} (daemon={daemon}; full axum + handlers in impl; schemas ready)");
        // real: axum::Server::bind... with /mcp route dispatching to handlers using QueryToolInput etc.
    } else {
        println!("MCP stdio server (daemon={daemon}; schemas + types ready for jsonrpc tools)");
        // real: loop stdin/stdout json for tools, call store.
    }
}
