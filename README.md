# qmcoxide (qmd Rust port)

**qmd** — Query Markup Documents — on-device hybrid search for your markdown notes, meeting transcripts, docs, and knowledge bases.

This is the Rust port (workspace `qmcoxide`, binary `qmd` for drop-in compatibility).

**Status**: Detailed prioritised task list (19 original + verify + 17 new gap-* from full verification pass) . Foundation (db, paths, config, chunk basic, store skeleton, indexing stub) complete with unit tests (db schema, paths docid/uri, chunk basic, store create+update count, config load) — green where `cargo test` can link (env link.exe shadow from scoop uutils often blocks full; use `cargo check --lib`). Higher layers (LLM, search/fusion, retrieval, full CLI, MCP, doctor/bench, exact chunk/config/indexing, tests harness, side-by-side) are stubs/TODOs only. **Full feature parity with qmd NOT achieved** (see docs/verification.md "Current Verification Results" + todo gap-P0-* for details). Gherkin specs exist but unexecutable. Continue one-by-one per plan: pick gap, impl+test-iterate till level green, MCP push, update todo. See plan (session/DESIGN), docs/requirements.md, specs/features/*.feature.

## Quick (future) Start
Once implemented:
```sh
cargo install qmcoxide  # provides `qmd`
# or cargo build --release && cp target/.../qmd ~/.cargo/bin/

qmd collection add ~/notes --name notes
qmd embed
qmd query "project timeline"
qmd mcp  # for agents
```

Full usage, SDK, MCP, architecture in the original https://github.com/tobi/qmd README (behavioral target) and our docs/.

## Key Artifacts in this port
- plan.md — full context, approach, phases, verification, surfaced specs.
- docs/requirements.md — explicit + NFR + implicit + tribal.
- specs/features/*.feature — Gherkin acceptance tests (path fidelity, collections, hybrid search, MCP, chunking, etc.).
- docs/score-fusion.md, chunking.md, etc.
- AGENTS.md — rules (never auto-index, etc.).

## Verification
Side-by-side with original `qmd` on fixtures + Gherkin + eval harness + cross-platform. See plan.md "Verification" section.

## License
MIT (same as original).

Upstream: https://github.com/tobi/qmd (the source of truth for behavior).

Repo: https://github.com/readall/qmcoxide (this Rust port).

Note: After git clone, run `cargo check` or `cargo build` to (re)generate Cargo.lock and fetch deps (lock may be environment-specific or updated).

## Windows Dev Note (this env)
`cargo check --bin qmd` succeeds. Full `cargo build/run` may hit MSVC `link.exe` shadowing if scoop/coreutils or similar puts a `link` in PATH before the real MSVC linker (common). Use check for validation until clean linker available. No new software was installed.

See plan.md for current status and next steps.
