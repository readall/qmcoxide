Feature: Retrieval (get and multi-get)
  # get by path/docid, multi by glob/csv, line ranges, full-path, bodies, max-bytes, errors with suggestions.
  # From README, 2.5.x changelog ( :from:count, --full-path, line-nums default, docid in outputs), cli.test + sdk.test

  Scenario: Get by path, by #docid, with line range suffix and flags
    Given indexed doc "docs/api.md" with known content and #abc123 docid
    When "qmd get docs/api.md:10:5"
    Then returns lines 10-14 (or equiv)
    When "qmd get #abc123 --full-path"
    Then header uses real FS path (./ or abs)

  Scenario: multi-get by glob, csv list (mix paths + docids), --max-bytes
    When multi-get "docs/*.md,#abc123,notes/2024*.md" --max-bytes 20480
    Then returns the docs (or errors for missing), respects byte cap, includes docid + qmd:// or full-path

  Scenario: Not found returns suggestions (similar files)
    When get a close-but-wrong path or docid
    Then error with similarFiles list (from index)
