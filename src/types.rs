//! Types: DocumentResult, DocumentNotFound (with similarFiles), SearchResult, HybridQueryResult (with score, snippet, context, docid, file:qmd:// , explain?), ExpandedQuery, Update/Embed Progress/Result, IndexStatus, SearchOptions, etc.
//! Match original from src/index.ts and store for SDK/CLI/MCP parity.
//! Also errors.

#[derive(Debug)]
pub struct DocumentResult {
    pub docid: String,
    pub title: String,
    pub context: Option<String>,
    // path, body optional, etc.
}

#[derive(Debug)]
pub struct DocumentNotFound {
    pub error: String,
    pub similar_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub docid: String, // e.g. #a1b2c3
    pub score: f64,
    pub snippet: String,
    pub start_line: usize,
    pub end_line: usize,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub queries: Option<Vec<String>>,
    pub intent: Option<String>,
    pub rerank: Option<bool>,
    pub collections: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
    pub explain: Option<bool>,
    pub chunk_strategy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HybridQueryResult {
    pub score: f64,
    pub snippet: String,
    pub context: Option<String>,
    pub docid: String,
    pub file: String, // qmd://col/path
    pub title: Option<String>,
    pub explain: Option<String>, // trace
}

pub type ExpandedQuery = String; // or struct with variants

// TODO: more (Progress, IndexStatus, Document, etc). See requirements SDK.
