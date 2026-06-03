//! MCP server: stdio (default) and HTTP transport (--http --port, --daemon with PID file in cache, stop, health /health, models warm 5min for embed ctx).
//! Tools: query (supports q or searches[], intent, collections, limit, candidateLimit, rerank, explain, chunkStrategy), get (pathOrDocid with line opts, full, fullPath), multi_get, status.
//! Must match original shapes, qmd:// URIs, docid, line nums etc for compat with Claude etc.
//! See original src/mcp/server.ts , mcp.test.ts , README MCP section, requirements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryToolInput {
    // q or searches, intent, collections, ...
}

pub async fn run_mcp_server(http: bool, port: u16, daemon: bool) {
    // TODO: stdio jsonrpc or axum for /mcp , tool handlers calling store, quiet GGML/LLAMA env before load, residency for mac if needed.
    // daemon: write pid, etc.
    println!("MCP server stub (stdio/HTTP per plan)");
}