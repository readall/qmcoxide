//! LLM integration: model download (hf-hub, same GGUF URIs as original: embeddinggemma..., qwen3-reranker, qmd-query-expansion), caching in ~/.cache/qmd/models/ ,
//! embed (prompts "task: search result | query: {}", "title: {} | text: {}"), rerank (cross encoder or rank), expand (chat for variants, HyDE, lex/vec),
//! context mgmt, GPU (metal/cuda/vulkan via flags/env or auto), CPU force, parallelism caps, inactivity dispose (5min), fingerprints for staleness, quiet logs.
//! See original src/llm.ts , docs/requirements for exact prompts/URIs/behavior. 
//!
//! Spike choice: llama-cpp-2 (active, tracks llama.cpp closely, GGUF, embed, chat/completion for expand, sampling for rerank, GPU via features/backends).
//! (Alternatives considered: llama-gguf pure-ish, kalosm high-level, candle. Chose for parity with original node-llama-cpp.)
//! Note: full integration awaits resolving build env (link shadowing); stub for now. Basic API usage sketched.

use llama_cpp_2 as llama;  // the crate

pub struct LlamaCpp {
    // TODO: model: Option<llama::LlamaModel>, ctxs for embed/rerank/generate, etc.
    // GPU backend from QMD_LLAMA_GPU or auto, with QMD_FORCE_CPU override.
    _phantom: std::marker::PhantomData<()>,
}

impl LlamaCpp {
    pub fn new(embed_model: Option<String>, generate_model: Option<String>, rerank_model: Option<String>) -> Self {
        // TODO (spike):
        // - resolve models (env QMD_*_MODEL > config > DEFAULT_ hf:ggml-org/... )
        // - use hf-hub or ureq to dl to ~/.cache/qmd/models/ if not present (fingerprint? )
        // - load with llama::LlamaModel::load_from_file(..., with gpu params)
        // - for embed: use llama::LlamaContext or embed API
        // - rerank: may use logits or special ranking ctx if supported, or chat scoring
        // - expand: LlamaChatSession or completion with grammar/JSON for lex/vec/hyde
        // - set ggml log quiet, metal residency etc.
        // - inactivity timer for dispose
        println!("LLM spike: llama-cpp-2 selected; new() stub (real load in full impl)");
        Self { _phantom: std::marker::PhantomData }
    }

    // TODO: pub fn embed_batch(&self, prompts: &[String]) -> Vec<Vec<f32>> { ... }
    // pub fn rank(&self, query: &str, docs: &[String]) -> Vec<f32> { ... }
    // pub fn expand(&self, query: &str, intent: Option<&str>) -> Vec<crate::types::ExpandedQuery> { ... }
    // pub async fn dispose(&self) { ... }
}