Feature: Chunking (regex + AST)
  # Exact algorithm from README "Smart Chunking" + ast.ts + tests/ast*.test.ts

  Scenario: MD heading boundaries preferred
    Given a long MD doc with # H1, ## H2 near the 900 tok mark
    When embedded or chunked with default (regex)
    Then cuts at highest scoring heading in the window, not arbitrary token or line

  Scenario: Code fences kept intact
    Given MD with large ```rust ... ``` block spanning chunk size
    Then the code block is not split (or kept whole if possible); breaks inside ignored

  Scenario: AST for code files (when --chunk-strategy auto)
    Given a .rs file with fn main, struct Foo, impl
    When chunk-strategy=auto
    Then chunks align at fn/struct/impl boundaries (merged with regex scores)

  Scenario: Fallback and flag on query/embed
    When no tree-sitter grammars or strategy=regex
    Then pure regex used for all (incl code)
    And flag is accepted on qmd embed and qmd query (for consistent chunk selection in search)

  # Add overlap 15%, pos tracking for snippets, etc.
