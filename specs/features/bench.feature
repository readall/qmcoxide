Feature: qmd bench for search quality evaluation
  # From 2.1.0 changelog, src/bench/ , test/eval*.ts , fixture jsonl
  # qmd bench <fixture.json> [--json] [--collection]
  # Measures precision@k, recall, MRR, F1 across backends (bm25, vector, hybrid, full pipeline)
  # Ships with eval-docs fixture.

  Scenario: Run bench on fixture
    Given eval fixture with queries + expected
    When "qmd bench test/eval-deep-research.jsonl --json"
    Then reports per query + summary metrics for different pipelines
    # TODO: harness port for Rust
