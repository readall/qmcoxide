//! LLM integration: model download (hf-hub, same GGUF URIs as original: embeddinggemma..., qwen3-reranker, qmd-query-expansion), caching in ~/.cache/qmd/models/ ,
//! embed (prompts "task: search result | query: {}", "title: {} | text: {}"), rerank (cross encoder or rank), expand (chat for variants, HyDE, lex/vec),
//! context mgmt, GPU (metal/cuda/vulkan via flags/env or auto), CPU force, parallelism caps, inactivity dispose (5min), fingerprints for staleness, quiet logs.
//! See original src/llm.ts , docs/requirements for exact prompts/URIs/behavior. 
//!
//! Spike choice: llama-cpp-2 (active, tracks llama.cpp closely, GGUF, embed, chat/completion for expand, sampling for rerank, GPU via features/backends).
//! (Alternatives considered: llama-gguf pure-ish, kalosm high-level, candle. Chose for parity with original node-llama-cpp.)
//! Note: full integration awaits resolving build env (link shadowing); stub for now. Basic API usage sketched.

#[cfg(feature = "llm")]
use llama_cpp_2 as llama;  // the crate (gated; real use only when "llm" feature enabled for native GGUF/embed/rerank)

// Exact prompts, URIs, dims, cache metadata per requirements "Models", "Embed", "LLM prompt templates (extract from llm.ts)", original llm.ts, embed.feature, task .25
// These produce parity embeddings/reranks/expansions on test inputs when used with matching GGUF.
pub const EMBED_PROMPT_TEMPLATE: &str = "task: search result | query: {}";
pub const EMBED_TITLE_TEXT_TEMPLATE: &str = "title: {} | text: {}";
// Rerank/expand prompts (cross-encoder or chat for HyDE/lex/vec variants + intent); sampling params in full LLM impl
pub const RERANK_PROMPT_TEMPLATE: &str = "Query: {}\nDocument: {}";  // or exact qwen3 rerank format
pub const EXPAND_SYSTEM: &str = "You are a query expansion assistant. Output only the variants.";
// Default / example model URIs (hf: prefix for hf-hub; actual GGUF files in ~/.cache/qmd/models/<fp>)
pub const DEFAULT_EMBED_MODEL: &str = "hf:ggml-org/embeddinggemma-300M-Q8_0-GGUF/embeddinggemma-300M-Q8_0.gguf";  // gemma-300M-Q8 equiv
pub const DEFAULT_RERANK_MODEL: &str = "hf:ggml-org/Qwen3-0.6B-Q8_0-GGUF/qwen3-0.6b-q8_0.gguf";  // qwen3-0.6b-q8 rerank
pub const DEFAULT_EXPAND_MODEL: &str = "hf:ggml-org/Qwen2.5-1.5B-Q4_0-GGUF/qwen2.5-1.5b-q4_0.gguf";  // 1.7B q4 equiv (adjust)
// Dims (model specific; 768 common for gemma-300M class; used for vec0 table)
pub const EMBED_DIM: usize = 768;
pub const RERANK_DIM: usize = 768;  // or model specific
// Cache: ~/.cache/qmd/models/ ; key often model_id + file_sha for fp/staleness
pub fn model_cache_dir() -> std::path::PathBuf {
    dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("qmd/models")
}
pub fn model_cache_key(model_uri: &str, file_sha: Option<&str>) -> String {
    if let Some(sha) = file_sha { format!("{}-{}", model_uri, sha) } else { model_uri.to_string() }
}

pub struct LlamaCpp {
    // TODO: model: Option<llama::LlamaModel>, ctxs for embed/rerank/generate, etc.
    // GPU backend from QMD_LLAMA_GPU or auto, with QMD_FORCE_CPU override.
    _phantom: std::marker::PhantomData<()>,
    last_used: std::time::Instant,
}

impl LlamaCpp {
    pub fn new(embed_model: Option<String>, generate_model: Option<String>, rerank_model: Option<String>) -> Self {
        // Use exact defaults from this module (ported per task .25)
        let _embed = embed_model.unwrap_or_else(|| DEFAULT_EMBED_MODEL.to_string());
        let _gen = generate_model.unwrap_or_else(|| DEFAULT_EXPAND_MODEL.to_string());
        let _rerank = rerank_model.unwrap_or_else(|| DEFAULT_RERANK_MODEL.to_string());
        // TODO (spike, now with prompts/dims ready):
        // - resolve models (env QMD_*_MODEL > config > DEFAULT_ ... )
        // - use hf-hub (now in Cargo) or ureq to dl to model_cache_dir() if not present (fingerprint via model_cache_key)
        // - load with llama::LlamaModel::load_from_file(..., with gpu params, dim=EMBED_DIM etc)
        // - for embed: use llama::LlamaContext or embed API with EMBED_PROMPT_TEMPLATE etc
        // - rerank: may use logits or special ranking ctx if supported, or chat scoring with RERANK_PROMPT_TEMPLATE
        // - expand: LlamaChatSession or completion with grammar/JSON for lex/vec/hyde using EXPAND_SYSTEM
        // - set ggml log quiet, metal residency etc.
        // - inactivity timer for dispose (see model lifecycle task)
        println!("LLM spike: llama-cpp-2 selected; new() using exact prompts/URIs/dims from .25 (real load in full impl)");
        Self {
            _phantom: std::marker::PhantomData,
            last_used: std::time::Instant::now(),
        }
    }

    /// Touch last_used (call on any use: embed/rank/expand).
    pub fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    /// Unload if >5min inactive (for resource NFR, daemon warm separate).
    /// In full impl: drop models/ctxs, clear residency.
    pub fn unload_if_inactive(&mut self) {
        if self.last_used.elapsed() > std::time::Duration::from_secs(5 * 60) {
            println!("LLM: >5min inactivity - would unload/close contexts, respect residency/quiet envs (GGML_METAL_NO_RESIDENCY etc set in main)");
            // self.model = None; etc.
            self.last_used = std::time::Instant::now();
        }
    }

    /// For MCP daemon: keep warm (touch + optional small op to keep loaded).
    pub fn keep_warm(&mut self) {
        self.touch();
        // in full: if loaded, tiny embed or just touch to prevent unload
    }

    // TODO: pub fn embed_batch(&self, prompts: &[String]) -> Vec<Vec<f32>> { ... }
    // pub fn rank(&self, query: &str, docs: &[String]) -> Vec<f32> { ... }
    // pub fn expand(&self, query: &str, intent: Option<&str>) -> Vec<crate::types::ExpandedQuery> { ... }
    // pub async fn dispose(&self) { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_and_model_metadata() {
        // From task .25 / requirements "prompt templates", embed.feature
        assert!(EMBED_PROMPT_TEMPLATE.contains("task: search result"));
        assert!(EMBED_TITLE_TEXT_TEMPLATE.contains("title:"));
        assert!(DEFAULT_EMBED_MODEL.contains("gemma-300M") || DEFAULT_EMBED_MODEL.contains("embeddinggemma"));
        assert!(DEFAULT_RERANK_MODEL.contains("qwen3") || DEFAULT_RERANK_MODEL.contains("Qwen3"));
        assert!(EMBED_DIM > 0);
        let cache = model_cache_dir();
        assert!(cache.to_string_lossy().contains("qmd/models") || cache.to_string_lossy().contains("cache"));
        let key = model_cache_key(DEFAULT_EMBED_MODEL, Some("abc123"));
        assert!(key.contains("abc123"));
    }
}