Feature: Embed command and vector generation
  # embed -f , -c scoped, --chunk-strategy, progress (byte + chunk count), fingerprints, model override QMD_EMBED_MODEL, max duration env, parallel, recovery from partial, complete chunk coverage required.
  # From changelog many fixes, README, requirements.

  Scenario: Embed all or scoped, force re-embed
    When "qmd embed"
    Then generates for pending, uses default model, stores with fingerprint
    When "qmd embed -c notes -f"
    Then only notes collection, clears only its vectors, re-embeds

  Scenario: Chunk strategy and model switch
    When "qmd embed --chunk-strategy auto"
    Then uses AST for code files
    Given QMD_EMBED_MODEL set to Qwen
    When embed -f
    Then uses new model, requires re-embed note, vectors not cross compat

  Scenario: Partial embed recovery, max duration
    # interrupted embed leaves incomplete; doctor/status honest; retries; QMD_EMBED_MAX_DURATION_MS
