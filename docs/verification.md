# Verification Procedure (for the qmd Rust port)

See plan.md "Verification (end-to-end...)" for the full numbered checklist. This is the executable summary.

## Prerequisites
- Original `qmd` available (npm/bunx or global install of @tobilu/qmd) for side-by-side.
- Rust toolchain (cargo check/run works in this env; full link may need MSVC in PATH).
- Fixtures: copy of test/eval-docs or synthetic with MD, code, special filenames ("#42 [draft].md"), dotted versions, long docs.

## 1. Layout & skeleton
- cargo check --bin qmd succeeds
- cargo run -- --help shows stub
- Gherkin files exist and are valid (count scenarios)

## 2. Side-by-side parity (repeat for every flow)
Setup temp collection with original qmd:
qmd collection add /tmp/fixture --name fixture
qmd update
qmd embed

Capture:
qmd query "..." --json --explain -n 20 > original.json
qmd get "..." --json --full > original-get.json
... (search, vsearch, multi-get with globs/docids, --full-path, line ranges, context cases, special paths, etc.)

Run same with Rust qmd (once impl'd; same index or mirrored new one per compat decision).
Compare:
- Same docids / paths (exact, including special chars)
- Ranks match or within 1
- Scores correlate (same "highly relevant" etc.)
- Snippets, contexts, titles, line numbers match
- Bodies (when requested) match
- JSON shapes, empty cases ([]), error suggestions match
- MCP tool responses (if using inspector or test client) match

## 3. Gherkin
- All *.feature scenarios have steps implemented (cucumber or assert_cmd harness)
- `cargo test` (or specific) passes the Gherkin suite

## 4. Quality / eval
- Ported eval harness runs on fixture
- Hybrid recall/precision >= original on the eval set (or top-k overlap high)

## 5. Non-functional
- MCP stdio + HTTP daemon (start, query while warm, stop, status)
- GPU/CPU modes (QMD_FORCE_CPU=1 etc) - at least check no crash, doctor output
- Resource: models unload after close/idle (manual or test measure)
- Special chars, unicode, case, dotted FTS, large/small docs all covered in Gherkin/parity
- No auto index behavior in any test or example

## 6. Packaging / cross
- cargo build --release produces runnable qmd
- (When CI) ubuntu/macos/windows checks
- Smoke: run from PATH after "cargo install --path .", small collection cycle

## 7. Docs
- README quickstarts work (once impl)
- All examples in docs/ and plan run or are illustrative only
- AGENTS.md rules followed in this repo

Run this procedure after each major phase. Update this file with new cases from changelog/tests.

See also specs/features/ for the Gherkin that should be green.

## Current Verification Results (2026-04, post foundation + partial higher)
**Conclusion: NOT full feature parity achieved. Required/desired functionality incomplete vs specs.**

- **What is implemented (foundation green where testable)**: 
  - db.rs: schema init (tables + FTS5), open (ext load stub), basic test_schema passes (in-mem).
  - paths.rs: docid (sha first6 #xxxxxx), to_qmd_uri, basic display; unit tests for docid/uri/display (special chars in URI test limited).
  - config.rs: structs for CollectionConfig/AppConfig; load stubs return default; basic test.
  - chunk.rs: ChunkStrategy enum + chunk_document (basic heading/blank split + ~15% overlap stub); test_chunk_basic.
  - store.rs: create_store + open/init, list_collections (stub), update (glob scan + chunk count only, NO real DB inserts/FTS/vec/fp/deact); 2 tests (create+list, update_basic count>0).
  - lib.rs / main.rs / cli.rs (partial clap) / types: module skeleton + reexports (note: broken StorePlaceholder ref).
  - Cargo: dev-deps assert_cmd/predicates/tempfile/insta present but ZERO usage in code or tests. No gherkin runner.
  - `cargo check --lib` / units: blocked in this env by MSVC link.exe shadowing (scoop uutils-coreutils 'link' in PATH before real linker; hits build.rs of transitive like bindgen/windows for llama-cpp-2 hard dep). History notes "check suffices for stubs"; full test/CI parity not verifiable locally without PATH fix (e.g. explicit MSVC or deactivate uutils). See Windows note in README/AGENTS.
  - Gherkin: 10 feature files exist with concrete scenarios (path_fidelity, search_hybrid_query, retrieval_get_multi, mcp, doctor, bench, embed, chunking, collection_management, config_and_env) — none executable yet.

- **Major gaps identified (added to todo list as P0/P1/P2 gap-*)**:
  - P0: No real LLM (llama-cpp-2 declared but all new()/embed/rerank/expand are println+TODO; no HF dl, no GGUF load, no prompts, no fp, no env/GPU/unload). Blocks embed + any hybrid quality.
  - P0: No search impl at all (store has 0 lines for lex/vec/hybrid; no RRF, no fusion math from score-fusion.md, no structured intent/lex/vec/hyde, no --explain). search_hybrid_query.feature unbacked.
  - P0: Retrieval incomplete (get/multi only in CLI stub prints; no path/#docid/qmd:// handling, no ranges :from:count, no --full/--full-path, no suggestions, no bodies). retrieval + path_fidelity roundtrips fail.
  - P0: CLI surface partial (Commands has Collection stub, Context, Get basic, Other fallback; missing search/query/vsearch/embed/update/doctor/mcp/ls/status/bench + most flags like --json --explain --full-path --no-rerank etc). run() only println stubs. No formatters, no OSC8, no TTY.
  - P1: MCP stub only (no server, no tools, no daemon/HTTP/PID/health).
  - P1: Doctor/bench/maintenance empty or println (no checks, no probes, no metrics, no vacuum).
  - P1: Config/env stubs (no real serde_yaml load from XDG/index.yml or example, no per-col, no write-through, most QMD_* env ignored).
  - P1: Chunk not exact (no scoring table H1=100/fence=80, no 900tok window/decay/finalScore math per chunking.md, no code fence protect, no tree-sitter AST).
  - P1: Indexing fidelity low (update counts chunks but no insert to documents/FTS/vectors, no fp, no deactivate, no title, no ignore/gitignore full, no update-cmd).
  - P1: Contexts, paths full fidelity (parse/fuzzy/NFC/canonical/special char exact in all flows), schema full + real vec0 load + migrations missing.
  - P1: Types incomplete, lib reexports broken (QMDStore/ create_store not matching original SDK contract), store methods stubs.
  - P2: No integration tests (0 assert_cmd, no gherkin execution of *.feature), no side-by-side vs original qmd, no eval harness run, no parity assertions (ranks/docids/scores/outputs/contexts).
  - P2: Original convo not imported (placeholder only). CI TODOs (smoke, gherkin, release). README status overstated ("all done"). No-auto rule honored in docs but not yet in test harness. Resource (unload) unimpl. Packaging (cargo install qmd) untested due to build issues.
  - Other: llama hard dep (not optional) makes all builds pull native; vec ext not loadable in practice (TODO path); tree-sitter not in Cargo; some schema drift between db.rs vs data-model vs original.

- **Parity verdict**: Foundation (db/paths/config/chunk basic indexing skeleton) provides base per early plan phases 3-8. Higher (9-19 + verify) are stubs/partial — no drop-in use, no quality search, no MCP, no doctor/bench, CLI not usable for core flows. Full Gherkin cannot pass. Side-by-side impossible. See added gap-* todos for exact next work (topo deps: LLM+chunk+config+indexing before search/retrieval; CLI/MCP after store core).

- Next: Per user "Continue till all", pick highest pending gap (e.g. gap-P0-01-llm or gap-P0-02 after deps), impl+iterate tests till pass at level, push via MCP, update todo status. Re-run this verify after major clusters.

Update this file + DESIGN.md + README after each gap close + final re-verify.
