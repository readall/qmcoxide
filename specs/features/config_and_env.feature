Feature: Config (YAML) and Environment variables
  # index.yml at XDG ~/.config/qmd/index.yml (or equiv), global_context, collections with path/pattern/ignore/context (tree), models: per col, includeByDefault, update-cmd.
  # Env: XDG_*, QMD_EMBED_MODEL etc, QMD_LLAMA_GPU, QMD_FORCE_CPU, QMD_EMBED_PARALLELISM, QMD_*_CONTEXT_SIZE, QMD_EDITOR_URI, QMD_STATUS_DEVICE_PROBE, QMD_DOCTOR_*, GGML_*/LLAMA_* quiet/residency.
  # SDK modes: configPath, inline config, DB-only. Write through for mutations.

  Scenario: Load from YAML, inline, DB only
    Given qmd.yml with collections and global_context
    When create store with configPath
    Then collections and contexts loaded, synced to DB
    # Similar for inline, and reopen DB-only

  Scenario: Per collection models and env override
    Given yaml with models: { embed: "hf:..." }
    Or QMD_EMBED_MODEL=...
    Then embed/search use it, fingerprint includes, re-embed note on switch

  Scenario: Editor URI for hyperlinks
    Given QMD_EDITOR_URI="vscode://file/{path}:{line}:{col}" or in config
    When search on TTY
    Then OSC8 links use template (path URI encoded etc)
