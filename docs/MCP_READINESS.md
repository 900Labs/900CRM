# MCP Readiness

900CRM is MCP-ready, not MCP-enabled, in this sprint.

## Current State

- `crates/crm-mcp` exists as a placeholder only.
- The core application does not import MCP libraries.
- The desktop application does not start or require an MCP server.
- MCP settings are represented as local settings placeholders:
  `mcp_status`, `mcp_enabled`, and `mcp_permission_mode`.
- External-client tables exist in SQLite but are disabled by default.

## Required Future Boundary

Future MCP code must call `crm-core` services:

```text
MCP tool/resource/prompt -> crm-core service -> storage repository -> SQLite
```

Future MCP code must not call SQLite directly.

## Permission Model

The target permission modes are:

- `disabled`
- `read_only`
- `draft_only`
- `write_with_confirmation`
- `write_allowed`

Initial implementation should support only:

- `disabled`
- `read_only`
- `draft_only`

All future write-like MCP actions should create `proposed_actions` rows first.
Desktop approval/execution behavior is deferred to a later sprint.

## Explicitly Forbidden Tools

Do not expose tools like:

- `run_sql`
- `query_database`
- `execute_sql`
- `read_file`
- `write_file`
- `execute_command`
- `shell`
- `terminal`

Only narrow business-level tools are allowed in future MCP work.

## Audit Requirements

Future MCP activity must write audit rows for:

- Client identity.
- Tool/resource/prompt accessed.
- Input summary.
- Entity touched.
- Whether a proposed action was created.
- Whether a user approved, rejected, or executed the action.

The current sprint provides the tables and service stub for proposed actions,
but no MCP request handling.
