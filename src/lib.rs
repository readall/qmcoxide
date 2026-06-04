//! qmcoxide lib (Rust port of qmd)
//! Public SDK surface (QMDStore equivalent, create_store, types, utilities).
//! See plan.md (or docs/DESIGN.md), docs/requirements.md, and specs/ for the full contract from original src/index.ts .

pub mod config;   // yaml, collections, contexts, models per-col, XDG
pub mod db;       // rusqlite compat, schema, FTS5, vec0 load, migrations for legacy
pub mod chunk;    // smart regex + AST (tree-sitter) chunking, exact scores/algos
pub mod llm;      // model mgmt (dl from HF same URIs), embed, rerank, expand prompts, sessions, GPU/env, fingerprints, inactivity unload
pub mod store;    // core: Store, indexing/reindex, search (lex/vec/hybrid), fusion RRF+blend, embed, get/multi, context, status, health, Maintenance
pub mod mcp;      // MCP server (stdio + HTTP daemon), tool impls matching contract (query/get/multi_get/status)
pub mod cli;      // CLI parser (clap), commands, formatters (json/csv/md/xml/files/explain/full-path), progress, hyperlinks, doctor etc.
pub mod paths;    // qmd:// handling, docid, path fidelity (verbatim, NFC, case)
pub mod syntax;   // formal SYNTAX EBNF parser for structured queries (intent/lex/vec/hyde/expand + lex ops) per SYNTAX.md
pub use syntax::{parse_query, Query, LexTerm};
pub mod maintenance; // vacuum, cleanup orphans, etc.

pub mod types;    // Shared types, errors, progress, results (DocumentResult, HybridQueryResult, etc.)

// Re-exports and types to be filled to match original public API parity (QMDStore, SearchOptions, etc.).
// For now placeholders so `cargo check` and early layout work. See plan for impl order.

pub use store::Store as QMDStore;  // TODO: real impl + create_store fn (parity with original QMDStore from src/index.ts)
pub use types::SearchResult;

// TODO: re-export key types, utils like extract_snippet, add_line_numbers, DEFAULT_*, Maintenance

