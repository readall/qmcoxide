Feature: qmd doctor diagnostics
  # From 2.5.0 changelog, cli/qmd.ts doctor impl, skills/qmd/SKILL.md, llm.rs doctor usage
  # Checks: sqlite/sqlite-vec versions, embedding fingerprint freshness/mixed, legacy adoption, content-hash sampling, device/GPU (with QMD_DOCTOR_DEVICE_PROBE), model cache, common env overrides (QMD_*, GGML_*), warnings for CPU etc.

  Scenario: Basic doctor run reports key health
    When I run "qmd doctor"
    Then output includes SQLite version, vec extension, model cache status, embedding fingerprint info, GPU/CPU accel note
    And suggests next steps like "qmd embed" or "unset QMD_DOCTOR_DEVICE_PROBE"

  Scenario: Doctor with device probe
    When QMD_DOCTOR_DEVICE_PROBE=1 qmd doctor
    Then performs native llama device probe (safe), reports backends (metal/vulkan/cuda), warns on no GPU

  Scenario: Doctor detects mixed fingerprints or stale vectors
    Given index with old and new embedding models/fingerprints
    When qmd doctor
    Then warns about mixed, suggests re-embed -f or adoption steps

  # TODO: more from code: repro metal etc, but core checks.
