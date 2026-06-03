Feature: MCP server (stdio and HTTP) and tools
  # From README MCP section, mcp/server.ts, mcp.test.ts, SYNTAX examples for structured queries

  Scenario: stdio MCP exposes query/get/multi_get/status with correct shapes
    Given MCP client connected to "qmd mcp"
    When client calls query tool with q or searches + intent + collections
    Then result matches contract (scores, qmd:// file URIs, docid, snippets with abs lines, context, optional explain)
    # Similar for get (with :suffix support), multi_get, status

  Scenario: HTTP transport + daemon
    When "qmd mcp --http --port 8181 --daemon"
    Then PID file written, status shows "MCP: running (PID ...)", models stay loaded
    And POST /mcp works, GET /health returns uptime
    When "qmd mcp stop"
    Then daemon exits cleanly

  Scenario: MCP query accepts structured and teaches syntax
    # Tool description must include full grammar/examples per original
