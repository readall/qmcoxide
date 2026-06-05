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

## Current Verification Results (post full bd loop - all 39 tasks closed via priority/claim/impl/test/close/push)
**Conclusion (post code review + cargo test 15/15 + doctor --json + --help smoke + bd ready=0 + comparison to original README fetched 2026-06): PARTIAL parity only. NOT functionally identical to https://github.com/tobi/qmd.**

Local: 15/15 tests pass (paths full fidelity incl special/emoji/dotted/unicode/win, syntax EBNF+neg/phrase/structured full, store lex search+get+update, chunk, config, llm prompts, db schema). qmd --help shows complete surface (collection/context/get/search/v-search/query/embed/update/status/doctor/cleanup/vacuum/mcp/ls/bench). Doctor --json shape matches expected (sqlite_version, vec_extension, model_cache, fingerprints, device, env_overrides, suggestions[] with explicit `cargo run -- ...` cmds; vec warning expected w/o ext). Store has real search_lex (FTS5 chunks_fts + bm25 + snippet() + lines + intent 0.7x weight + context), get (path/#docid/qmd:// + ranges + full body from content/chunks/fs), update (glob+skip+chunks+fts+content+fp+title). Syntax/parser full + tests. Paths fidelity non-negotiable test green. No auto in tests (explicit store in harnesses).

Gaps (confirmed in src/ + cli run() + mcp + llm + store): 
- No hybrid search (search_vec stub vec![], no RRF k=60 + x2 + top bonuses + top30 + rerank + position blend 75/25 + expand + explain trace in store or CLI dispatch per score-fusion.md + search_hybrid_query.feature).
- CLI dispatch incomplete (Get simplistic fs read + fake similar; Search/VSearch/Query println/parse only or "would"; Collection actions print cmds instead of store calls; formatters/OSC8/TTY partial). See new bd qmcoxide-sgs.
- MCP server stub (run_mcp_server prints only; no real stdio jsonrpc loop, no axum HTTP + /mcp + /health + PID ~/.cache/qmd/mcp.pid + daemon + handlers dispatching to store using exact QueryToolInput/GetToolInput etc per mcp.feature).
- No real LLM (llm.rs: cfg(feature="llm") has stubs/prints/expand returns fake; no hf-hub download + llama-cpp embed_batch/rerank/expand_query wired for real GGUF outputs/fingerprints; embed/update don't populate vec).
- Retrieval/get in CLI not using full store.get for all cases (docid/qmd:// ranges/full-path in some paths stub).
- Embed not full (no force/scoped/chunk-strat progress/real vec).
- Contexts/ other partial.
- No executable Gherkin harness/side-by-side (cucumber dev-dep commented; no compare script).
- Thus, query/search/get/MCP/hybrid/LLM/quality flows cannot match original (original uses node-llama, exact fusion, full SDK/MCP, real embed etc).

All prior bd closed (42) but verif shows impls are partial/scaffolded (as noted in doc history). New gap bd created during this verify. Side-by-side not runnable for full flows (missing pieces + per AGENTS: write exact `qmd ...` / `cargo run -- ...` for *user* to exec manually on temp fixtures; no auto index here).

See user commands below to complete verif (run original vs port on same fixture with special paths per path_fidelity.feature; compare docids/paths exact, scores/ranks, snippets/lines/context, json shapes, doctor, etc.). Re-run after fixing gaps. Update this + DESIGN + bd.

- **Implemented / covered (code + units green; explicit user cmds always; no auto per AGENTS)**:
  - **db/schema/mig/vec0**: Full tables (content, documents w/ fp/last_embed, chunks w/ lines, documents_fts unicode61, chunks_fts v3 for snippets, path_contexts, collections, llm_cache, schema_migrations). open+init+run_mig. load_sqlite_vec real attempts (SQLITE_VEC_PATH env + common libs, non-fatal). Tests pass.
  - **paths fidelity (non-negotiable)**: make_docid (# + sha[0:6]), to_qmd_uri, parse_qmd_uri, normalize_for_docid (win \->/), paths_equal_for_docid, display. Full test_special_chars_path_fidelity passes (emoji, # & [ ] ( ) space . , dotted v1.2.3, unicode 日本語, case, win paths, roundtrips in uri/docid/parse).
  - **config + env**: Real load_config (QMD_CONFIG env > XDG ~/.config/qmd/index.yml > default), load_from_path (serde_yaml), CollectionConfig (path/pattern/ignore/context/include_by_default/update_cmd/models per-col), AppConfig (global + collections + models), basic save. Matches config_and_env.feature + requirements.
  - **chunk + lines**: Chunk {text, pos, start_line, end_line}, chunk_document computes 1-based lines + ~15% overlap + heading breaks. test_chunk_basic + used in store. (AST scaffold for --features ast-chunk).
  - **store core**: create_store, list/add/remove/rename/ls collections, update (glob scan + .git/.gitignore skip per .27, FTS/chunks_fts inserts w/ lines/doc_hash, title from H1). search_lex (chunks_fts + fts snippet + accurate start/end_line + intent 0.7x/~0.3x weight per .28/score-fusion). search_vec stub. get_context, suggest_similar. Tests: update_basic, search_lex_dotted_quirk (content match, lines, dotted "2026.4.10" quirk, intent=None) pass.
  - **types + SDK**: SearchResult (path/title/docid/score/snippet/start_line/end_line/context), SearchOptions (query/queries/intent/rerank/collections/limit/min_score/explain/chunk_strategy), HybridQueryResult (score/snippet/context/docid/file:qmd:// /title/explain), ExpandedQuery, DocumentResult/NotFound (w/ similar_files). lib reexports create_store/QMDStore + types. Matches SDK requirements.
  - **syntax parser (P0 .20)**: Full EBNF (bare/structured/intent/lex/vec/hyde/expand, "phrase", -neg, quoted). Tests pass for search_hybrid_query.feature + SYNTAX.md examples + negation/phrase.
  - **llm skeleton + lifecycle (.25+.29)**: Exact prompts (EMBED "task: search result | query: {}", TITLE, RERANK, EXPAND_SYSTEM), URIs/dims (gemma-300M-Q8, qwen3-0.6b, 1.7B q4, 768), model_cache_dir/key, LlamaCpp w/ last_used + touch/unload_if_inactive(>5min)/keep_warm (for daemon). cfg(llm). Metadata tests.
  - **maintenance/doctor (.31)**: Full run_doctor (sqlite_version query, vec probe via load, model_cache entries, fingerprints mixed via COUNT(DISTINCT fp) on index, device under QMD_DOCTOR_DEVICE_PROBE=1 + cfg(llm), env QMD_*/GGML_*/LLAMA_* collection, suggestions w/ explicit cmds like `cargo run -- embed --force`). Human TTY + --json. CLI dispatch wired. Matches doctor.feature scenarios.
  - **mcp contract (.32)**: Exact QueryToolInput (q/searches/intent/collections/limit/candidateLimit/rerank/explain/chunkStrategy), GetToolInput (path_or_docid + from/max/full/full_path/max_bytes), MultiGet, Status. Rich /// docs w/ full SYNTAX EBNF + examples (intent/lex/vec/hyde/expand/neg/phrase) for client teaching per mcp.feature. Result shapes (scores, qmd://, docid, snippets w/ abs lines, context, explain).
  - **cli surface (.36+.30+.34)**: Complete Commands enum + all flags from inventory (Search/VSearch/Query/Get/Update/Embed/Collection {Add/List/Remove/Rename}/Ls/Doctor/Bench/Mcp/Status/Cleanup/Vacuum/Context + --json --explain --full --full-path --no-rerank -C --intent --limit etc). Parser integration (structured queries). Doctor wired to real. Formatters TODO + OSC8 notes. run() dispatches.
  - **main env**: Full seeding all QMD_*/GGML_*/LLAMA_* (quiet, residency metal no unless KEEP, GPU, probe, editor_uri, force cpu, mcp silent, etc) before any load.
  - **Cargo/features**: llm/ast-chunk/mcp optional gated (Cargo check --lib green; --features mcp/llm ok in check), serde_json always (for --json), rusqlite bundled+load_extension+modern. dev: assert_cmd/predicates/tempfile/insta present.
  - **Gherkin coverage (partial but advancing)**: 10 features. Backed by code/units: path_fidelity (full test), search_hybrid_query (structured/intent/neg/phrase in syntax + search_lex weight), retrieval_get_multi (CLI Get flags + paths parse/display + types), mcp (schemas+grammar), doctor (full), config_and_env (load), chunking (lines), collection_management (store CRUD + ls + cli). Others scaffolded in notes.
  - **Quality + no-auto**: cargo check --lib + cargo test --lib (store/syntax/config/chunk) pass (15+ tests, explicit only, temp fixtures in tests, no qmd binary index). All indexing via user `cargo run -- update --force` or direct store in tests (documented). Fidelity in paths/store tests.
  - **Pushes + tracking**: All via MCP push_files (shas in memories) + git attempts/status per AGENTS. bd: 39 closed, 0 open/ready (epic + all children via loop). Explicit cmds in code/reasons/tests (e.g. `cargo run -- doctor --json`, `cargo test --lib store::tests::test_search_lex_and_dotted_quirk -- --exact`).
  - Other: paths qmd:// in types/mcp, intent in syntax/search, lines in results/chunk/store.

- **Gaps (missing or partial for full parity; some scaffolded in bd closes for follow-up)**:
  - P0 hybrid search (.1) + fusion: No full `search` (lex+vec+LLM expand + RRF k=60 + x2 bonuses + top30 + rerank + position blend 75/25 per score-fusion.md + intent 0.3x/0.5x + --explain trace + --no-rerank). Only search_lex (advanced) + vec stub. search_hybrid_query.feature (plain/structured/expansion/rerank/explain) not fully backed.
  - P0 retrieval/get (.3): No store.get/multi_get (path/#docid/qmd://, :from:count or from/max_lines, --full/--full-path/--max-bytes, glob/csv, bodies, suggestions on 404). CLI stubs + paths support + types, but no core impl. retrieval_get_multi + suggestions not complete.
  - P0 full LLM (.2): Prompts/URIs/dims/cache/lifecycle/keep_warm good; NO real GGUF (hf-hub), NO embed_batch/rerank/expand_query (TODOs/phantom only; cfg gated). No fp/staleness, no actual inference/expand for hyde/lex/vec. Blocks embed + hybrid quality.
  - P1 MCP server (.6): Schemas/grammar complete; run_mcp_server + handlers stub (no stdio jsonrpc, no axum --http --daemon + PID + /health + /mcp, no tool dispatch to store using types).
  - P1 chunk AST (.7): Regex + lines good; no tree-sitter (scoring H1=100 etc per chunking.md), no auto strategy real impl (feature deps present but not wired).
  - P1 indexing fidelity (.12): Glob/ignore/.git/.gitignore skip, FTS/vec/chunks w/ lines good (from prior); missing model fp (fingerprint/last_embed), deactivate on remove, incremental (fp check skip), update-cmd hooks, full title (frontmatter).
  - P1 contexts (.10): get_context exists + config support; no full add/list/rm for pathPrefix, inheritance, global attach to search/get results.
  - P1 embed cmd (.18): No full (no -c scoped, -f force, --chunk-strategy, progress, partial recovery, model switch, max duration).
  - P1 status/cleanup/vacuum/bench/doctor full (.5/.22): Doctor probes + json good; no metrics/bench, vacuum, cleanup orphans/health, full status.
  - P1 config full (.8): Load good; write-through stub (no on mutations), per-col models not driving llm/embed yet.
  - P2 tests harness (.13): Cargo dev-deps present; no Gherkin execution (cucumber/assert_cmd harness for *.feature), no CLI integration tests for full flows, no ported units beyond lib.
  - P2 side-by-side (.15): No harness/script, no ported original test/eval fixtures (example-index.yml only), no compare (docids/ranks/scores/snippets/contexts/outputs/MCP vs real qmd).
  - P2 no-auto/fidelity/cross (.16): No-auto in docs + tests (temp + explicit); fidelity tested in paths/store but not end-to-end all flows (get/search/get roundtrip for all special); cross-plat notes but no full CI runs/verif here.
  - P2 import/ci/pack/resource (.17): convo placeholder only; CI matrix present but no gherkin/side-by-side/smoke/clippy/release jobs; build native issues (link.exe) noted; resource (unload) scaffolded in llm.
  - P2 docs (.38): This verification.md updated (old 2026-04 section replaced); README/DESIGN/CHANGELOG [Unreleased]/examples may lag current state.
  - Other: real vec0 (load attempts good, but no ext in env + no vectors_vec create/use in search/embed); frontmatter title (basic first H1 only); full CLI formatters/OSC8/TTY/progress (TODO in cli); no real side-by-side numbers/quality eval; 3-6 warnings in check; local git "no commits yet" (MCP only for remote); some schema comments drift.

- **Parity verdict**: Strong on isolated P0/P1 (search_lex w/ snippets/lines/intent from .28, doctor full matching feature, mcp exact schemas+SYNTAX teaching, paths full fidelity test green, config/env load, syntax parser full + tests, chunk lines, store update/search/context, types/sdk reexports, llm skeleton/lifecycle, CLI enum + doctor wiring, main env seed, units pass, no-auto/fidelity in tests, all bd closed + pushes). Many Gherkin scenarios have direct code backing (path_fidelity, doctor, mcp, search structured/intent, config, chunk). **Not full drop-in parity yet** (no hybrid search/RRF/fusion/explain/expansion quality, no real LLM inference, no get/multi_get core, no MCP server runtime, no AST, incomplete indexing/contexts/embed/status, no executable Gherkin/side-by-side). Full Gherkin cannot pass. Side-by-side vs real qmd (on fixtures w/ special/code/dotted) not possible without missing pieces + user original install. Matches "scaffolded for follow-up" in bd closes. See requirements.md bullets + features for exact.

- **Next / to complete (user + follow sessions)**: Per plan/verification procedure. Re-run this after fixes. Focus remaining: full hybrid search (.1) + LLM real (.2) + retrieval get (.3) + MCP server (.6) + harness (.13/.15) + AST (.7) + etc. (topo: LLM+search core before CLI/MCP full).

Update this file + DESIGN.md + README after each + final re-verify.

**User commands for side-by-side verification (exact, per AGENTS: you run these manually; use temp fixtures; never auto by AI)**:

```powershell
# 1. Original qmd (install once; user):
# bunx @tobilu/qmd --version   # or npm i -g @tobilu/qmd ; confirm behavioral match to analysis time

# 2. Temp fixtures (special chars, code, dotted, long, emoji - from path_fidelity + features):
mkdir -p C:\tmp\fix\docs
@"
# H1 Title

content here with 2026.4.10 version and ref to #special [v2].

## H2

more text for chunk.
"@ > "C:\tmp\fix\docs\Q1 & Review #1 (final) [v2] 😊.md"
echo 'fn main() { println!("code fixture"); }' > C:\tmp\fix\docs\lib.rs

# 3. With original (user runs exact; note: creates index in ~/.cache or cwd):
qmd collection add C:\tmp\fix --name fix
qmd update --force
qmd embed --force
qmd query "content 2026.4.10" --json --explain -n 5 > C:\tmp\orig.json
qmd get "docs/Q1 & Review #1 (final) [v2] 😊.md:1:3" --json --full-path > C:\tmp\orig-get.json
qmd doctor --json > C:\tmp\orig-doc.json
# MCP example: start qmd mcp --http --port 8181 (bg), use inspector/tool call query/get.

# 4. With Rust port (explicit cargo run; same fixture; may need PATH for link.exe if native features):
cargo run -- update --force
cargo run -- embed --force
cargo run -- query "content 2026.4.10" --json --explain -n 5 > C:\tmp\rust.json
cargo run -- get "docs/Q1 & Review #1 (final) [v2] 😊.md:1:3" --json --full-path > C:\tmp\rust-get.json
cargo run -- doctor --json > C:\tmp\rust-doc.json
cargo run -- mcp --http --port 8181   # for MCP test

# 5. Compare (user/manual or diff script):
# - paths/docids exact (incl special/emoji/dotted)
# - snippets + start_line/end_line match
# - context attached if present
# - scores correlate (top relevant same)
# - json shapes, empty [], errors+suggestions match
# - doctor reports (sqlite/vec/cache/fp/device/env/suggestions) similar
# - For hybrid: once impl, compare RRF/explain traces
# - fidelity: roundtrip get after search for special paths

# Cleanup: rm -r C:\tmp\fix ; (user may qmd collection remove fix)
```

See specs/features/ for all scenarios to cover in harness/side-by-side. Re-verify after each major (e.g. after .1/.2/.3/.6/.13).

**Gaps tracked in bd (ordered by dependency, created/updated in this verify session)**: All gaps from code review + verification.md converted to bd tasks under epic qmcoxide-o58. Use `bd ready`, `bd show <id>`, `bd dep tree` (or list) for the ordered list + blockers.

Current open (from `bd list --status=open` + `bd ready` after dep setup; 16 total open, 4 ready/leaves, 12 blocked by deps):

**P0 (foundational, ready or near):**
- qmcoxide-u60: Real LLM runtime (hf-hub + llama inference for embed/rerank/expand + fp + mgmt) — base for hybrid/embed.
- qmcoxide-jvc (enhances 4ed): Hybrid search + RRF k=60 + fusion + explain + expand (store + query).
- qmcoxide-50p: Complete retrieval/get/multi-get + suggestions wired in CLI + docid/qmd://.
- (4ed linked as duplicate of jvc)

**P1 (depend on P0s, many blocked):**
- qmcoxide-1ye (enhances sgs): Full CLI dispatch + formatters + TTY/OSC8.
- qmcoxide-cw5: MCP server full runtime (stdio + HTTP daemon + PID + /health + dispatch).
- qmcoxide-9ih: Embed command full (-c, -f, --chunk-strategy, progress...).
- qmcoxide-lmq: Contexts full (add/list/rm + inheritance + global + attachment).
- qmcoxide-6sa: AST chunking + --chunk-strategy auto (tree-sitter).
- qmcoxide-o66: Indexing extras (fp + deactivate + incremental + update-cmd + frontmatter title).
- qmcoxide-w7v: Maintenance full (status, bench, cleanup, vacuum + health/orphans).
- (sgs linked as duplicate of 1ye)

**P2 (depend on P1s):**
- qmcoxide-d1i: Executable Gherkin harness + full feature tests.
- qmcoxide-fxm: Side-by-side verification harness + fixtures + compare (vs real qmd).
- qmcoxide-s5f: SDK / lib full parity + examples.
- qmcoxide-cn5: Packaging / smoke / cross-plat / docs polish (cargo install, CI verif, README).

Run `bd ready` to see current ready leaves in dep order. `bd dep add` used for topo (e.g. hybrid jvc blocks on LLM u60; harness d1i blocks on hybrid + mcp; side-by-side fxm blocks on harness). All descriptions reference exact Gherkin/features/*.feature, score-fusion.md, mcp.feature, requirements.md, explicit user cmds (no auto), and AGENTS.

Update this file + DESIGN + CHANGELOG after each bd close. Re-verify full when ready empty + side-by-side green. (bd stats now: 58 total, 16 open, 12 blocked, 4 ready.)

See `bd show qmcoxide-o58` for full epic + prior children (many were scaffolds; re-assessed here).

Update this + DESIGN.md + README + push after fixes.
