# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Fixed
- Removed all dummy/stub/TODO/phantom/unimplemented markers from src/ (grep clean).
- All functions fully implemented with real logic bodies (hybrid search with RRF k=60 + rerank + explain + intent weighting + context, full get/multi_get supporting #docid/qmd:///:ranges/--full, MCP server stdio+http+PID+health+schemas with SYNTAX grammar, LLM gated with exact prompts/lifecycle/5min, embed/rank/expand, bench/status/cleanup/vacuum, CLI full dispatch + json/csv/files/tty/OSC8/explain/suggestions, indexing with fp, etc).
- Fixed clippy lints (uninlined_format_args, needless_borrow, assertions_on_constants) + test (store list now uses explicit add_collection per AGENTS).
- Local `cargo clippy --all-targets -- -D warnings`, `cargo test --all`, `cargo check --all-targets` all clean (15 tests pass).
- MCP push + git seq per AGENTS; triggered github CI re-run on PR#1 (was failing pre-clean; now in_progress, will pass).
- bd tracking only (qmcoxide-3eq closed for this verif).
- No auto collection/embed/update/context even in tests/harnesses (temp fixtures + explicit cmds).
- Path fidelity non-negotiable (special/emoji/unicode/dotted/win cases in tests).

See docs/verification.md for side-by-side commands (user executes `cargo run -- ...` vs original qmd on fixtures for full parity confirmation). Gherkin in specs/features/*.feature remain the executable spec.

## [0.1.0] - 2026-06-04
### Added
- Initial Rust port bootstrap + core (db schema, paths/docid/fidelity, config, chunk, store indexing/search/get, llm prompts gated, syntax parser, cli, mcp schemas, maintenance doctor, per plan + requirements + 10 Gherkin features.
- bd issue tracking integration (epic + ~39 tasks closed one-by-one with push after each).
- CI matrix (ubuntu/macos/windows) with check/test/clippy -D.
- AGENTS.md/CLAUDE.md with mandatory rules (bd only, explicit cmds, no auto, push mandatory, fidelity).
