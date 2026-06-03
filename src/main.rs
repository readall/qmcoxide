//! qmd (Rust port / qmcoxide)
//! On-device hybrid search for markdown notes, docs, transcripts, and code.
//! See plan.md (session) / docs/DESIGN.md , docs/requirements.md + specs/features/*.feature for full specs.

use std::env;

mod cli; // clap dispatch, formatters etc.
use clap::Parser; // for Cli::parse() on the derived type from mod cli

fn main() {
    // Seed quiet envs for mcp/llm (like original bin/qmd launcher): GGML/LLAMA log, metal residency on darwin unless QMD_METAL_KEEP_RESIDENCY=1
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        std::env::set_var("LLAMA_LOG_LEVEL", std::env::var("LLAMA_LOG_LEVEL").unwrap_or_else(|_| "error".into()));
        std::env::set_var("GGML_LOG_LEVEL", std::env::var("GGML_LOG_LEVEL").unwrap_or_else(|_| "error".into()));
        std::env::set_var("GGML_BACKEND_SILENT", "1");
    }
    if std::env::consts::OS == "macos" && std::env::var("QMD_METAL_KEEP_RESIDENCY").ok().as_deref() != Some("1") {
        std::env::set_var("GGML_METAL_NO_RESIDENCY", "1");
    }

    let cli = cli::Cli::parse();
    cli::run(cli);
}