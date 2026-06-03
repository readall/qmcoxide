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