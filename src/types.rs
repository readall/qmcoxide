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

// TODO: many more: HybridQueryResult, SearchOptions {query?, queries?, intent?, rerank?, collections?, limit?, minScore?, explain?, chunkStrategy?, ...}, etc.