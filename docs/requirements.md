# QMD (Rust Port) - Requirements, NFRs, Specs, Tribal Knowledge

This document + the Gherkin in `specs/features/*.feature` + SYNTAX.md + plan.md constitute the living specification for the Rust port (qmcoxide crate, qmd binary).

Sourced from exhaustive read-only analysis of https://github.com/tobi/qmd (README, full CHANGELOG, CLAUDE.md, SYNTAX.md, package.json, src/index.ts public API, test suite structure, CI, bin launcher, etc.) + the originating share conversation (title only retrievable; full body capture pending user paste).

See plan.md for approach, verification, phases, open questions (some decided: binary "qmd", cache "qmd" subdirs, index clean-break).

## Explicit Functional Requirements (core)
- On-device only hybrid search (BM25 FTS5 + vector + LLM rerank + expansion) for local FS collections of .md (and code/other via glob).
- Collections: add/list/remove/rename/ls with name, path, glob mask, ignore[], includeByDefault, optional update-cmd shell hook.
- Context: add/list/rm for collection+pathPrefix (qmd:// virtual), hierarchical inheritance, global context, attached to results.
- Indexing (update): FS scan (glob), title extraction (H1/frontmatter), content hash -> docid (first 6 hex), store verbatim paths, FTS, deactivate removed.
- Embed: smart chunk (~900 tok 15% overlap, MD break scoring table + code fence protect + optional AST tree-sitter for ts/js/py/go/rs), format prompts, batch embed, store vectors + fingerprints (model+params), force/scoped.
- Search modes: search (lex/BM25 only), vsearch (vec only), query (full hybrid+expand+rerank, recommended).
- Structured queries (per SYNTAX.md EBNF): intent:, lex: (with "phrase" -negate), vec:, hyde: (hypothetical doc). Bare/expand: for auto LLM variants. First gets x2 weight.
- Intent: disambiguates everywhere (expansion prompt, rerank, chunk 0.5x, snippet 0.3x, disables strong BM25 bypass).
- Fusion (exact): per README "Score Normalization & Fusion" + diagram: RRF k=60 + original x2 + top-rank bonuses + top30 + rerank (yes/no+logprob) + position blend (1-3:75/25, 4-10:60/40, 11+:40/60).
- Retrieval: get by path (fuzzy suggestions) or #docid, multi-get (glob or csv list of paths/docids), line ranges (fromLine/max or :from:count suffix), --full (whole doc), --full-path (real FS), max-bytes cap.
- Output formats parity: default (color, OSC8 hyperlinks via QMD_EDITOR_URI template, TTY-aware), --json, --csv, --md, --xml, --files (for agents), --explain (traces), --all --min-score, --format.
- MCP: stdio (default) + HTTP (--http --port, --daemon + PID + stop + status), tools query/get/multi_get/status exactly as documented (shapes, qmd:// URIs, structured support).
- SDK (lib): create_store equivalent, QMDStore interface with search (query or queries[] + intent etc), searchLex/Vec, expand, get/getDocumentBody, multiGet, collection/context mgmt, update/embed, status/health, close. 3 modes (yaml/inline/db-only), write-through.
- Models: same 3 GGUF (embed gemma-300M-Q8, rerank qwen3-0.6b-q8, expand 1.7B q4), same HF URIs, prompts, cache ~/.cache/qmd/models/, env overrides, fingerprints.
- Env + config parity (XDG, QMD_*, GGML/LLAMA quiet, GPU force, parallelism, max duration, editor_uri, per-col models in yaml).
- CLI parity: all subcommands + flags from CLAUDE + README + 2.5+ (doctor, line ranges, full-path, format, etc). See inventory in plan.md.
- Maintenance: status, doctor (diagnostics, GPU safe, fingerprints), cleanup, vacuum, orphan handling, migrations (paths, fingerprints, legacy).
- Chunk strategy: regex default, auto (AST) for code; flag on embed/query.
- Error/edge: DocumentNotFound + similar suggestions, empty -> [] for json, skip unreadable/empty, graceful CPU fallback, etc.

## Non-Functional Requirements (NFRs)
- Local/privacy (models dl only on first use from HF; no other net).
- Resource: lazy + 5min inactivity unload for LLM contexts, bounded context sizes, parallelism caps (Win CUDA serial default), truncate long, residency opt-out for CLI on mac.
- Fidelity: exact path roundtrip (special chars #&[]() space . emoji, case, unicode NFC where needed, Linux case-sens), docid stability, FTS quirks (dotted versions, CJK, hyphens etc), score/rank quality (hybrid wins, buckets), chunk boundaries match original algos.
- Cross-platform: Linux/macOS/Windows (original CI limited to *nix but win support existed via optional sqlite-vec; Rust should include win in matrix). GPU (metal/cuda/vulkan) or CPU via same env/flag surface.
- Index robustness + auto-migrate legacy on update for known fixes (paths, fingerprints, case, etc.).
- SDK isolation: explicit dbPath, no globals/side effects.
- MCP long-lived: models warm, HTTP for shared, daemon, health, concurrent ok.
- UX/Agentic: structured outputs, --files, explain, hyperlinks (TTY), editor jump, progress, human times, empty formats correct.
- Quality gate: eval harness (port fixtures/harness) + side-by-side vs original qmd on same corpus (ranks/docs/contexts/paths exact or within tol; quality >= ).
- No auto-index in AI flows (honor CLAUDE "Do NOT run automatically").
- Packaging: simple cargo install / build --release produces usable qmd (models dl on use); no complex launcher (Rust eliminates ABI pain).
- CI: at least ubuntu/macos (match original) + windows; cargo test/clippy/smoke.

## Tribal Knowledge & Process Rules (must honor in Rust port + docs)
- From CLAUDE.md: Use Rust equiv of "bun" dev flow (cargo test etc). Never auto-run collection/embed/update in agent sessions; always emit example commands for user to run. Never "compile to single exe" if it would break natives (but Rust is native). Release: keep [Unreleased] entries live; adapt release process.
- From skills/release: full changelog + hook + cumulative notes discipline.
- From history/launcher: many workarounds (Metal residency, quiet GGML before import, lockfile-driven runner choice, node_modules gate for source mode, pnpm global pitfalls) — in Rust most disappear, but replicate env seeding for quiet/GPU/residency where binding needs it.
- From tests/CI: comprehensive parity on launcher detection (eliminated), bin smoke, container, grammar packaging (tree-sitter), etc.
- From package: ship equivalent of skills/, scripts/ (or Rust ports), CHANGELOG, LICENSE.

## Implicit Requirements (from every "Fixed" in CHANGELOG + test assertions)
(See plan.md "Implicit" section for the long list: path roundtrips, dotted FTS, Metal destructor safe exit, launcher heuristics, embed fingerprints/migrations/partial recovery/scoped, HTTP URI shape, get line range + docid + full-path + line-nums defaults, --full whole-doc, doctor separation, lazy LLM, Windows CUDA serial, etc.)

Every one of these is now a required behavior for the port (captured in Gherkin + tests).

## Query Syntax (formal)
See docs/SYNTAX.md (exact EBNF, lex operators, intent rules, MCP/CLI examples). Parser + semantics must match 100% for agent compat.

## Architecture / Data Model / Fusion / Chunking
See plan.md appendix + docs/ (to be expanded: architecture.md, data-model.md with full CREATEs from store.ts, score-fusion.md with exact math, chunking.md with score tables + algos + AST nodes).

## Verification
Full checklist in plan.md (side-by-side original qmd vs Rust on fixtures for all flows + Gherkin pass + eval + MCP + cross-plat + no auto-index + resource + packaging smoke).

## Open / To Capture
- Full original share conversation transcript (paste here or into docs/conversation.md).
- Exact current CREATE TABLE statements + migration logic (extract during impl from store.ts + migrate-schema.ts + test setups).
- LLM prompt templates (extract from llm.ts).
- Any additional NFRs from the share link once imported.

This + Gherkin + plan.md + original SYNTAX/CHANGELOG/README are the spec. Update as port progresses.