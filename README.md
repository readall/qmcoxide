# qmcoxide (qmd Rust port)

**qmd** — Query Markup Documents — on-device hybrid search for your markdown notes, meeting transcripts, docs, and knowledge bases.

This is the Rust port (workspace `qmcoxide`, binary `qmd` for drop-in compatibility).

**Status**: Early skeleton + specs. See `plan.md` (the approved implementation plan), `docs/requirements.md`, `specs/features/*.feature` (Gherkin), and `docs/*.md` for the full extracted requirements, NFRs, tribal knowledge, and verification process.

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