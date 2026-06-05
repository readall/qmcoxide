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

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:7510c1e2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
