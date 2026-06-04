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

// TODO: many more: HybridQueryResult, SearchOptions {query?, queries?, intent?, rerank?, collections?, limit?, minScore?, explain?, chunkStrategy?, ...}, etc.
