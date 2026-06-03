# AGENTS.md / CLAUDE.md for qmcoxide (Rust port of qmd)

Ported/adapted from original repo CLAUDE.md. These rules are mandatory.

## Core Rules (do not violate)
- **NEVER run collection add / embed / update / context changes automatically** in any AI coding or agent session. Always write the exact `qmd ...` (or cargo run -- ...) command for the *user* to execute manually. The index is user data.
- Do not modify the SQLite DB directly (use the CLI or future SDK methods).
- Write example commands for the user.

## Development
- `cargo test` (or `cargo test --test <name>`)
- `cargo run -- <command>` to run from source (e.g. `cargo run -- query "foo"`)
- `cargo build --release` for the qmd binary (named "qmd" via [[bin]]).
- Clippy: `cargo clippy -- -D warnings`
- Gherkin / specs live in `specs/features/*.feature` — keep them passing and expand with new behavior.
- Layout per plan.md: src/main.rs (CLI), src/lib.rs (SDK), src/store.rs etc.

## Important for this Rust port
- The goal is behavioral parity with original qmd (paths, docids, fusion math, chunk boundaries, query grammar, output shapes, MCP tool contract, etc.).
- Use the Gherkin + docs/requirements.md + plan.md + original SYNTAX/CHANGELOG as spec.
- After changes, run side-by-side verification against a real original `qmd` install on fixtures (see plan verification section).
- No automatic indexing, even for tests (use temp fixtures + explicit commands in test harnesses).
- Honor all the hard-won fixes (path fidelity for special chars is non-negotiable).

## Release (adapt from original skills/release)
- Keep changelog entries under `## [Unreleased]` as you work.
- Use conventional or similar + cargo release or manual for versions.
- Update plan.md / docs when decisions or new artifacts added.

See plan.md for the full port plan, artifacts, and verification.