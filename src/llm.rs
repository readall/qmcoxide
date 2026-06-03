//! LLM integration: model download (hf-hub, same GGUF URIs as original: embeddinggemma..., qwen3-reranker, qmd-query-expansion), caching in ~/.cache/qmd/models/ ,
//! embed (prompts "task: search result | query: {}", "title: {} | text: {}"), rerank (cross encoder or rank), expand (chat for variants, HyDE, lex/vec),
//! context mgmt, GPU (metal/cuda/vulkan via flags/env or auto), CPU force, parallelism caps, inactivity dispose (5min), fingerprints for staleness, quiet logs.
//! See original src/llm.ts , docs/requirements for exact prompts/URIs/behavior. Spike crate (llama-cpp-2 etc) here.

pub struct LlamaCpp {
    // TODO: holders for embed/rerank/generate contexts/sessions
}

impl LlamaCpp {
    pub fn new(embed_model: Option<String>, ...) -> Self {
        // TODO: resolve model (env > config > default), download if needed, load with gpu flags, QMD_FORCE_CPU etc.
        Self {}
    }

    // TODO: embed_batch(chunks) -> vecs
    // rank (query, docs) -> scores
    // expand (query, intent) -> Vec<ExpandedQuery {type: Lex/Vec/Hyde, query}>
    // dispose on idle
}