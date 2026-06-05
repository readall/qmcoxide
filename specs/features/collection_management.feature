Feature: Collection management
  # From CLAUDE.md, README, collections-config.test.ts, collections.ts, store

  Scenario: Add collection with name and custom mask
    Given no collections
    When I run "qmd collection add /tmp/my-notes --name notes --mask '**/*.md'"
    Then "qmd collection list" shows name "notes", path, glob_pattern '**/*.md', doc_count 0
    And the collection is included by default

  Scenario: Remove and rename collection
    Given collection "oldname"
    When I run "qmd collection rename oldname newname"
    Then list shows "newname"
    When I run "qmd collection remove newname"
    Then it is gone

  Scenario: ls shows collection tree and subpaths
    Given collection "journals" with files under 2024/ and 2025/
    When I run "qmd ls journals/2024"
    Then it lists files under that prefix using qmd:// or display paths

  # Add update-cmd, include/exclude, multi -c etc from changelog 1.1.0+
