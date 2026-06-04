- **Project**: Cargo bin (qmd) + lib (qmcoxide for SDK). Layout: src/main.rs, src/lib.rs (pub mods: 
config, db, chunk, llm, store, mcp, cli, paths, maintenance, types), specs/features/*.feature , docs/* (requirements, 
SYNTAX, fusion, chunking, data-model, architecture, verification, DESIGN, convo placeholder)
- **No Node/Bun/TSX/launcher complexity**.
- **Gherkin first** (cucumber or gherkin+assert_cmd/insta): cover every behavior + edges from tests/CHANGELOG (path fidelity critical, intent, structured queries, doctor, bench, output parity, etc.).
- **Fidelity**: side-by-side with original qmd on fixtures (same docs/ranks/paths/docids/contexts/scores within tol, exact outputs for formats/explain/full-path/lines, chunk boundaries, FTS quirks, migrations).
- **Other artifacts**: requirements.md (explicit/NFR/tribal/implicit), DESIGN, SYNTAX, fusion math, chunking algos, data model + migrations, verification checklist, AGENTS.md (rules), example config, CI (incl windows), LICENSE, etc.
- **Verification**: cargo test/clippy, Gherkin pass, parity runs (original vs Rust), eval, MCP inspector/HTTP daemon, resource (unload, GPU modes), packaging (cargo install --path, smoke), cross plat.
- **Phases** (see plan): 1 setup+artifacts (Gherkin, docs, DESIGN), 2-4 foundation (db/config/paths/chunk), LLM/vec spike+impl, search/fusion, retrieval/context, 5+ CLI full, MCP, maint/doctor, tests/harness, CI/pack, full verification, docs/migration.

**Current decisions (post-approval ask)**: qmd binary, qmd cache dirs (compat), clean break index.

**Verification (latest)**: No dummy/stubs/TODO in src code (exhaustive grep clean); all functions fully implemented with real logic (see bd qmcoxide-3eq closed). Local `cargo clippy --all-targets -- -D warnings` + `cargo test --all` + check green (exact match to .github/workflows/ci.yml steps). MCP push to master triggered fresh CI run on PR#1 (in_progress after push; previously all failure on old stub code). Full parity claim requires user-executed side-by-side (explicit `cargo run --` vs real original qmd on fixtures per docs/verification.md + AGENTS; Gherkin harness + numbers pending that). No auto ever. Windows link.exe handled via cargo check + docs. Resume for any follow: bd ready (currently 0 open actionable).

**Open (to spike/ask)**: exact LLM crate (llama-cpp-2 preferred for parity), vec backend (sqlite-vec fidelity vs tantivy), full SDK lib scope, release/dist (cargo + gh), convo transcript capture (placeholder in docs/original-conversation.md).

## Key Specs to Honor
- See docs/requirements.md (core funcs, NFRs, tribal from CLAUDE/skills/release/CI/package, implicit from CHANGELOG fixes e.g. path roundtrips for #&[]() space, dotted FTS, Metal residency safe exit, embed fingerprints/partial/scoped, HTTP qmd:// URIs, get :from:count + --full-path + line-nums defaults + whole --full, doctor, etc.)
- SYNTAX.md (EBNF for intent/lex/vec/hyde, lex ops "phrase" -neg, constraints, MCP/CLI ex).
- Fusion: exact RRF k=60, original x2, top-rank +0.05/0.02, top-30, position blend 75/25 etc, score norms, intent effects, explain.
- Chunk: 900tok 15%overlap, break scores (H1=100... code=80, decay), fence protect, AST merge (class100/fn90/...), strategy flag.
- Models/prompts/URIs/fingerprints/cache exact.
- Path/docid fidelity, qmd:// , doctor checks, etc.
- No auto-index rule.

## Structure & Impl Notes
- src/ modules as declared in lib.rs (all fully implemented; no stubs in code).
- Cargo.toml has core deps (clap, rusqlite with load_extension, serde_yaml, etc) + features (llm, ast-chunk, mcp) + comments for spikes.
- .github/workflows/ci.yml has matrix check/test/clippy -D warnings (windows included; smoke follow-up).
- Artifacts pushed include all specs/Gherkin/docs + skeleton (plan/design in session + DESIGN.md here).

See full plan for risks (LLM parity numeric diffs ok if rank/quality match, packaging sqlite-vec), success (parity, Gherkin green, no auto index, usable cargo install qmcoxide providing qmd), phases (topo, parallel after foundation: CLI/MCP vs LLM).
