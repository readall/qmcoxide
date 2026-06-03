Feature: Path fidelity and docid stability
  # Critical from Unreleased + many prior fixes + path-fidelity.test.ts + store-paths.test.ts
  # Filenames/paths with special chars, unicode, case, dots must round-trip exactly
  # through index -> search -> get -> ls -> multi-get. Docids stable 6-char hex of content hash.

  Background:
    Given a fresh index at a temp location
    And a collection "notes" at a temp dir

  Scenario: Special chars in filename round-trip (hash, spaces, brackets, parens, ampersand)
    Given a file "Meeting #42 (Q4) [draft] & notes.md" in the collection with content "Q4 planning notes"
    When I run qmd update
    Then qmd ls notes shows the exact path "notes/Meeting #42 (Q4) [draft] & notes.md"
    And qmd search "Q4" --json returns file "notes/Meeting #42 (Q4) [draft] & notes.md"
    And qmd get "notes/Meeting #42 (Q4) [draft] & notes.md" succeeds and body matches
    And qmd get "#<docid>" (from search) succeeds

  Scenario: Dotted version strings match in FTS5 (e.g. 2026.4.10)
    Given a document containing "Release 2026.4.10 notes"
    When I run qmd search "2026.4.10"
    Then it matches the document (FTS splits and ANDs tokens, does not strip dots to 2026410)

  Scenario: Case preservation and case-sensitive FS
    Given files "MEMORY.md" and "memory.md" in collection
    When indexed on Linux (case-sensitive)
    Then both are distinct, searchable by original case, and get by exact path works

  Scenario: Unicode / emoji filenames
    Given a file "🐘.md" with content "elephant note"
    Then search and get by "notes/🐘.md" or docid works without crash or mangling

  Scenario: Full-path output uses ./ relative under PWD or absolute realpath
    When search with --full-path under a subdir of collection root
    Then paths are ./relative or absolute, and get --full-path round-trips to FS

  # Add more from path-fidelity.test.ts, store-paths.test.ts, recent fixes for # & [] () etc.
