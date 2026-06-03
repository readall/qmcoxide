# DESIGN / Port Plan Summary for qmcoxide (Rust port of qmd)

Full detailed plan (with context, all surfaced reqs, Gherkin plan, verification, phases, risks, ambiguities) is maintained in the development session. Key excerpts and current status below. Update this as impl progresses. See also docs/requirements.md and specs/features/.

## Context & Motivation
See plan for analysis of https://github.com/tobi/qmd (v2.5.3) + originating Grok share "Porting qmd to Rust or Go".
Rich specs in README, full CHANGELOG (implicit reqs from fixes), CLAUDE.md (tribal: NEVER auto collection/embed/update), SYNTAX.md (formal query grammar), extensive tests, public SDK API in src/index.ts, complex launcher/bin for Bun/Node ABI/GPU.

Goal: faithful behavioral port in Rust for native packaging (single bin + model dl), eliminate JS runtime/launcher pain, better cross-plat (win CI), while preserving exact fidelity for users/agents/MCP (paths/docids, fusion math, chunk boundaries, query syntax, output shapes, etc.).

## Recommended Approach (from plan, decisions applied)
- **Binary name**: qmd (drop-in compat, per user decision after plan approval)
- **Crate**: qmcoxide (aligns workspace)
- **Cache/config dirs**: keep qmd subdirs (~/.cache/qmd , ~/.config/qmd) for compat (per decision)
- **Index**: clean break / new index only (per decision); still use qmd cache dir for co-existence during transition.
- **Project**: Cargo bin (qmd) + lib (qmcoxide for SDK). Layout: src/main.rs, src/lib.rs (pub mods: config, db, chunk, llm, store, mcp, cli, paths, maintenance, types), specs/features/*.feature , docs/* (requirements, SYNTAX, fusion, chunking, data-model, architecture, verification, DESIGN, convo placeholder)
- **XDG** via dirs crate.
- **Core tech** (to spike/validate): rusqlite (+bundled FTS5), sqlite-vec runtime load (chose for fidelity - exact vec0/SQL as original; tantivy alt for pure Rust packaging later), clap (full derive), serde_yaml, tree-sitter+grammars (optional feature for AST), hf-hub/ureq for models, llama.cpp binding (chose llama-cpp-2 for GGUF embed/rerank/chat + GPU parity: metal/cuda/vulkan + envs; see spike), thiserror/anyhow, indicatif/colored for UX, regex/sha2/dirs/glob.
- **MCP**: TBD crate (rmcp etc) or custom (json + axum for HTTP).
- **No Node/Bun/TSX/launcher complexity**.
- **Gherkin first** (cucumber or gherkin+assert_cmd/insta): cover every behavior + edges from tests/CHANGELOG (path fidelity critical, intent, structured queries, doctor, bench, output parity, etc.).
- **Fidelity**: side-by-side with original qmd on fixtures (same docs/ranks/paths/docids/contexts/scores within tol, exact outputs for formats/explain/full-path/lines, chunk boundaries, FTS quirks, migrations).
- **Other artifacts**: requirements.md (explicit/NFR/tribal/implicit), DESIGN, SYNTAX, fusion math, chunking algos, data model + migrations, verification checklist, AGENTS.md (rules), example config, CI (incl windows), LICENSE, etc.
- **Verification**: cargo test/clippy, Gherkin pass, parity runs (original vs Rust), eval, MCP inspector/HTTP daemon, resource (unload, GPU modes), packaging (cargo install --path, smoke), cross plat.
- **Phases** (see plan): 1 setup+artifacts (Gherkin, docs, DESIGN), 2-4 foundation (db/config/paths/chunk), LLM/vec spike+impl, search/fusion, retrieval/context, 5+ CLI full, MCP, maint/doctor, tests/harness, CI/pack, full verification, docs/migration.

**Current decisions (post-approval ask)**: qmd binary, qmd cache dirs (compat), clean break index.

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
- src/ modules as declared in lib.rs (stubs added for config/db/chunk/llm/store/mcp/cli/paths/maintenance/types).
- Cargo.toml has core deps (clap, rusqlite, serde_yaml, etc) + comments for spikes (tree-sitter feature, LLM, MCP, dev for gherkin/assert_cmd/insta).
- .github/workflows/ci.yml expanded for Rust (check/test + TODO clippy/smoke/windows).
- Artifacts pushed include all specs/Gherkin/docs + skeleton (plan/design in session + DESIGN.md here).

See full plan for risks (LLM parity numeric diffs ok if rank/quality match, packaging sqlite-vec), success (parity, Gherkin green, no auto index, usable cargo install qmcoxide providing qmd), phases (topo, parallel after foundation: CLI/MCP vs LLM).

Update DESIGN/requirements/Gherkin as we discover more during spikes/impl. Run verification checklist often (side-by-side original qmd vs this on fixtures covering special paths, intent, doctor, embed, MCP HTTP, outputs, chunk AST vs regex, etc).

(Extracted/condensed from approved plan.md at time of initial artifacts push.)