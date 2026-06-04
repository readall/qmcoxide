//! qmd (Rust port / qmcoxide)
//! On-device hybrid search for markdown notes, docs, transcripts, and code.
//! See plan.md (session) / docs/DESIGN.md , docs/requirements.md + specs/features/*.feature for full specs.

use clap::Parser; // for qmcoxide::cli::Cli::parse() (trait for the derive in lib)


fn main() {
    // Full env seeding for all QMD_*/GGML_*/LLAMA_* (quiet, residency, GPU, parallelism, max duration, editor_uri, doctor/status probe, force cpu, metal keep etc) per requirements "Env + config parity", AGENTS, config_and_env.feature, llm/mcp tasks.
    // Do before any llama/mcp load (like original bin/qmd launcher).
    // QMD_FORCE_CPU, QMD_LLAMA_GPU, QMD_EMBED_MODEL, QMD_EDITOR_URI, QMD_EMBED_PARALLELISM, QMD_*_CONTEXT_SIZE, QMD_STATUS_DEVICE_PROBE, QMD_DOCTOR_*, GGML_LOG_LEVEL, LLAMA_LOG_LEVEL, GGML_BACKEND_SILENT, GGML_METAL_NO_RESIDENCY, QMD_METAL_KEEP_RESIDENCY etc.
    for (k, v) in std::env::vars() {
        if k.starts_with("QMD_") || k.starts_with("GGML_") || k.starts_with("LLAMA_") {
            // pass through or normalize; specific below
            let _ = v;
        }
    }
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        std::env::set_var("LLAMA_LOG_LEVEL", std::env::var("LLAMA_LOG_LEVEL").unwrap_or_else(|_| "error".into()));
        std::env::set_var("GGML_LOG_LEVEL", std::env::var("GGML_LOG_LEVEL").unwrap_or_else(|_| "error".into()));
        std::env::set_var("GGML_BACKEND_SILENT", "1");
    }
    if std::env::consts::OS == "macos" && std::env::var("QMD_METAL_KEEP_RESIDENCY").ok().as_deref() != Some("1") {
        std::env::set_var("GGML_METAL_NO_RESIDENCY", "1");
    }
    // Example: QMD_FORCE_CPU=1 -> set GGML_CUDA=0 or similar if needed in LLM
    if std::env::var("QMD_FORCE_CPU").ok().as_deref() == Some("1") {
        std::env::set_var("GGML_CUDA", "0");
        std::env::set_var("GGML_VULKAN", "0");
        // etc for other
    }

    let cli = qmcoxide::cli::Cli::parse();
    qmcoxide::cli::run(cli);
}
