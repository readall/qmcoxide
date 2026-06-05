Feature: Hybrid search (query) with expansion, intent, structured sub-queries, RRF + rerank + blend
  # Core of the value prop. Exact fusion from README diagram + score section + SYNTAX + intent.test.ts + structured-search.test.ts + rrf-trace.test.ts

  Scenario: Plain query auto-expands and uses hybrid + rerank (best quality)
    Given indexed collection with relevant docs
    When I run "qmd query \"authentication flow\""
    Then results use expansion (lex + vec + hyde variants), RRF (k=60, original x2), top-30 rerank, position blend
    And top results are highly relevant; scores in 0-1 range with interpretation

  Scenario: Structured query with lex/vec/hyde and intent
    When I provide a multi-line query:
      """
      intent: web performance and Core Web Vitals
      lex: performance
      vec: how to improve page load times
      """
    Then expansion prompt included intent, reranker used it, chunk/snippet weighting applied, strong-signal bypass disabled
    And results are disambiguated toward web perf (not fitness or team health)

  Scenario: --explain shows full traces (RRF contributions, bonuses, reranker, final blend)
    When run with --explain --json
    Then each result has explain object with per-list ranks, bonuses, rerank score, blended score etc.

  Scenario: --no-rerank and candidate limit affect pipeline
    When --no-rerank or -C 10
    Then rerank step skipped or fewer candidates; latency lower, quality may differ

  # Add min-score, --all, collection filter, multi-collection, chunk-strategy, empty results -> [] for json, etc.
