# Architecture (Rust port)

See original README "Architecture" diagram (the big ASCII pipeline) and "How It Works".

Rust version aims for identical logical pipeline and data flow:
- Collections + contexts (yaml + DB sync)
- Indexer (glob scan -> documents + FTS + chunks)
- Embedder (chunker -> prompts -> LLM embed -> vectors_vec)
- Querier (parse query or expand -> lex/vec/hyde -> FTS + vec in parallel -> RRF + bonuses -> rerank -> blend -> results with context/snippet)
- Retriever (by path/docid/glob, body slices)
- MCP layer on top of store
- CLI on top of store + formatters

Key crates (TBD after spikes): rusqlite (+ vec), tree-sitter*, LLM binding (llama-cpp-*), clap, serde_yaml, etc.

Differences from TS: no launcher, native single bin (models separate dl), direct FFI or safe wrappers for sqlite-vec / llama.

See plan.md for detailed port order and fidelity requirements.
