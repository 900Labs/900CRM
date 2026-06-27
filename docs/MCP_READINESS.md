# MCP Readiness Baseline

Date: 2026-06-24
Last updated: 2026-06-26

This document records the current Model Context Protocol (MCP) readiness state
for 900CRM. It describes the optional local MCP package boundary that exists
today and the gates still required before any network MCP server, auth surface,
AI behavior, or broader write behavior is accepted.

## Current Status

- 900CRM core has no built-in AI agent.
- Normal desktop and `crm-core` operation do not require internet access, cloud services, or model providers.
- MCP is intended to be a separate optional package boundary, not a required desktop/core dependency.
- `crates/crm-mcp` contains an offline SDK-backed read-only tool catalog CLI,
  a disabled-by-default runtime guard/config/status model, a local one-shot
  JSON-RPC probe, and a disabled-by-default config-gated stdio loop for
  MCP requests. The stdio path can execute only the reviewed SDK read tools and
  `create_activity_draft` pending-action creation when config includes both a
  local app-data directory and reviewed external-client id. It is not a
  network MCP server.
- `crates/crm-sdk` now provides a narrow local facade over `crm-core` for
  reviewed reads and the reviewed activity-draft proposed-action flow; it is
  not an MCP runtime.
- The desktop app and `crm-core` do not start an MCP server, bind a localhost
  listener, expose prompt/resource surfaces, or manage MCP tokens/secrets.

## Architecture Boundary

Future MCP tools must call the read-only `crm-sdk` facade or explicit
`crm-core` service APIs. They must not read or write the SQLite database
directly, and they must not bypass the existing storage, validation, audit,
sync-log, and proposed-action boundaries.

The default `crm-mcp` catalog, one-shot probe, and runtime status remain
metadata only. They are useful for reviewing the initial tool/runtime boundary,
but default execution does not serve tools, execute SDK methods, accept client
requests, bind sockets, or enable an MCP runtime.

The current `crm-mcp` JSON-RPC handler is also metadata only. It handles one
already-provided JSON string at a time for deterministic local tests and
returns at most one JSON-RPC response. A separate stdio line loop can be
attempted only through an explicit config-backed CLI flag and only when the
config is enabled and loopback-valid. The stdio path is local process IO only;
it does not listen on TCP/HTTP/SSE, authenticate clients, query the database
directly, execute direct write tools, or start a network server. If config also
includes both `app_data_dir` and `external_client_id`, the stdio path can call
the reviewed `crm-sdk` facade for the initial read tools after `crm-core` read
permission evaluation succeeds, and can create a pending
`create_activity_draft` proposed action after `crm-core` draft permission
evaluation succeeds.

The intended call path is:

1. Optional MCP package receives a local client request.
2. MCP package validates the client, requested tool, and configured mode.
3. MCP package calls the `crm-sdk` facade for supported reads and the reviewed
   activity-draft proposed-action flow.
4. `crm-sdk` requires `crm-core` external-client read permission for read tools
   and draft permission for `create_activity_draft` before dispatching to
   existing `crm-core` services.
5. `crm-core` applies normal domain validation and records audit evidence.
6. Write-like external-client requests create proposed actions for user review
   unless a narrow reviewed execution path explicitly supports the action.
7. The current reviewed execution path is limited to core approval of
   `create_activity_draft` proposed actions.

## Active Readiness Surfaces

The following data and API surfaces exist today:

- External client records with `name`, `client_type`, `enabled`, and `permission_mode`.
- External client permission rows keyed by `(client_id, tool_name)`.
- Permission evaluation for the initial modes `disabled`, `read_only`, and `draft_only`.
- `crates/crm-sdk` read-only SDK facade with exported initial tool constants
  for `contacts.list`, `organizations.list`, `deals.list`,
  `activities.list`, and `search.global`.
- `crates/crm-sdk` exported draft tool constant and method for
  `create_activity_draft`. This creates only a pending proposed action after
  draft permission succeeds; it does not create an activity, approve a proposed
  action, reject a proposed action, execute a proposed action, or bypass
  Pending Actions review.
- `crates/crm-mcp` offline catalog generation for those initial SDK read tool
  constants. `cargo run -p crm-mcp -- --print-tool-catalog` and the
  `--list-tools` alias print deterministic JSON entries with `name`,
  `access_kind: "read"`, `requires_external_client_permission: true`,
  `sdk_backed: true`, and `runtime_enabled: false`.
- `crates/crm-mcp` runtime guard configuration and status metadata.
  `McpRuntimeConfig::default()` is disabled and localhost-only
  (`enabled: false`, `bind_host: "127.0.0.1"`, `bind_port: 0`,
  `app_data_dir: null`, `external_client_id: null`). Validation rejects enabled
  runtime configurations that name a non-loopback bind host, and rejects partial
  execution context where only one of `app_data_dir` or `external_client_id` is
  supplied.
  `cargo run -p crm-mcp -- --print-runtime-status` prints deterministic JSON
  with `serving: false`, `tool_execution_enabled: false`, and reason
  `runtime disabled` for the default config. This does not start a server,
  create a listener, execute tools, call the SDK, issue tokens, or bind a
  network socket.
- `crates/crm-mcp` can load optional JSON runtime config metadata from a file
  and print deterministic non-serving status with
  `cargo run -p crm-mcp -- --print-runtime-status-from-config <path>`.
  Missing optional config paths use the disabled default. Invalid JSON and
  enabled non-loopback hosts are rejected. If a config sets
  `enabled: true` on a loopback host without execution context, status reports
  `serving: false`, `tool_execution_enabled: false`, and reason
  `execution context missing`. If the config also includes both `app_data_dir`
  and `external_client_id`, status reports `tool_execution_enabled: true` and
  reason `reviewed stdio execution context available`. Loading config still
  does not start a network server, create a listener, issue tokens, perform
  authentication, or access the network.
- `crates/crm-mcp` can handle a single local JSON-RPC message with
  `cargo run -p crm-mcp -- --handle-jsonrpc-once '<json>'`. The supported
  metadata-only subset is `initialize`, `tools/list`, and no-response
  notifications such as `notifications/initialized`. `initialize` reports
  deterministic server metadata and tools-list capability only. `tools/list`
  maps the offline catalog to MCP-style tool metadata with read-only input
  schemas, read-only annotations, and explicit `runtimeEnabled: false` /
  `executionEnabled: false` readiness metadata. Malformed JSON returns the
  standard parse error with `id: null`, invalid request shapes return invalid
  request errors, and unsupported methods return method-not-found errors.
  `tools/call` is rejected and does not execute anything. This one-shot probe
  does not start a serving loop, transport, listener, SDK dispatch, database
  access, authentication, token/secret handling, or tool execution.
- `crates/crm-mcp` has a first local stdio transport-shaped boundary behind
  `cargo run -p crm-mcp -- --serve-stdio-from-config <path>`. The flag loads
  the existing runtime config metadata and rejects disabled configs before
  reading stdin or writing stdout. Enabled configs must still pass loopback
  validation. When both gates pass, newline-delimited JSON-RPC input is handled
  by the same JSON-RPC handler shape used by `--handle-jsonrpc-once`; request
  responses are emitted as newline-delimited JSON and notifications emit no
  line. Without execution context, `tools/call` is rejected. With execution
  context, `tools/call` can execute only `contacts.list`,
  `organizations.list`, `deals.list`, `activities.list`, `search.global`, and
  `create_activity_draft` through `crm-sdk::CrmSdk`. `create_activity_draft`
  creates a pending proposed action only when the external client has
  `draft_only` mode plus a matching `can_write = true` /
  `requires_confirmation = true` permission row. Unknown tools, write-like tool
  names outside this reviewed draft flow, malformed arguments, missing search
  query, missing permissions, disabled clients, and unsupported methods return
  JSON-RPC errors. This path is disabled by default, local-only stdio,
  SDK-routed, and does not add direct database access, token/secret handling,
  authentication, TCP/HTTP/SSE/socket listeners, sync-server behavior, schema
  changes, UI behavior, direct activity creation, or proposed-action
  approval/rejection/execution tools.
- SDK read methods for contacts, organizations, deals, activities, and global
  search. Each method calls
  `CrmCore::evaluate_external_client_tool_read_permission(client_id,
  tool_name)` before returning data from existing `crm-core` services.
- Settings UI local activation review/edit controls for existing external-client
  records using only `disabled`, `read_only`, and `draft_only`.
- Proposed action rows with pending, approved, rejected, and other schema-reserved lifecycle states.
- Audit log entries for accountable changes and proposed-action decisions.
- Audit log entries for explicit external-client read/draft permission
  evaluations and draft permission checks performed before external proposed
  actions are created, including denied attempts where the attempted
  client/tool can be identified.
- Pending Actions UI for reviewing proposed actions.
- Audit Log UI for inspecting recorded activity.
- Proposed action approve/reject APIs and UI controls.
- Core approval execution for pending proposed actions where `tool_name` or
  `action_type` is `create_activity_draft`.
- Settings UI permission review/edit controls for existing external-client
  per-tool permission rows.

Approving a supported `create_activity_draft` proposed action creates an activity
through the normal `crm-core` service/storage path and marks the proposed action
`executed`. The permission-keyed `tool_name` must be `create_activity_draft`;
`action_type` may be either `create_activity_draft` or the compatible legacy
category value `create_activity`. Mismatched tool/action identities remain
pending and return an explicit invalid-input error when approval is attempted.
Rejection remains decision-only. Approval still does not run MCP/client code.

The supported draft input JSON shape is:

```json
{
  "title": "Call Amina",
  "activity_type": "call",
  "description": "Confirm next steps",
  "due_at": "2026-06-25T09:00:00Z",
  "linked_entities": [
    { "entity_type": "contact", "entity_id": "contact-id" },
    { "entity_type": "organization", "entity_id": "organization-id" },
    { "entity_type": "deal", "entity_id": "deal-id" }
  ]
}
```

`title` is required. `activity_type` defaults to `task` when omitted.
`description`, `due_at`, and `linked_entities` are optional. `due_at` maps to
the existing activity `due_date` field. Linked entity types are limited to
`contact`, `organization`, and `deal`; the current core path supports at most one
linked contact and one linked deal because those preserve the existing legacy
activity mirror fields, while organization links use the first-class activity
link table.

## Permission Modes

The allowed initial modes are:

- `disabled`: client access is denied.
- `read_only`: read access can be allowed for explicitly granted tools.
- `draft_only`: read access can be allowed for explicitly granted tools, and draft proposed-action creation can be allowed only when the matching permission row has `can_write = true` and `requires_confirmation = true`.

The schema reserves future values such as `write_with_confirmation` and `write_allowed`, but they are inactive in the current implementation. Current evaluation logic treats those future modes as unsupported.

The Settings UI can list and create disabled external client placeholders,
update local activation state for those clients, and review or upsert local
per-tool permission rows. This is a readiness surface only: local activation
does not issue credentials, start a server/listener, enable sync server
behavior, or run MCP/client code. Disabled clients still evaluate as disabled
even when permission rows exist.

Explicit `crm-core` permission evaluations now write local `audit_log` evidence
with the attempted `client_id`, `tool_name`, access kind (`read` or `draft`),
mode when available, allowed flag, decision reason, and result status. Draft
permission checks inside `create_external_proposed_action_stub` use the
`mcp_client` actor, include available proposed-action entity scope, and record
denied checks before returning the existing denial or not-found error. These
audit-only evaluation entries do not create `sync_changelog` rows.

Activation updates must keep stored state consistent:

- disabled clients store `enabled = false` and `permission_mode = 'disabled'`;
- enabled clients store `enabled = true` with `permission_mode = 'read_only'`
  or `permission_mode = 'draft_only'`;
- future modes such as `write_with_confirmation` and `write_allowed` are
  rejected by the activation update path until a future sprint explicitly
  supports them.

## Current Non-Goals

The current codebase intentionally does not include:

- MCP server startup.
- A localhost MCP listener.
- MCP authentication tokens, client secrets, or credential storage.
- Prompt or resource implementations.
- MCP runtime bindings beyond the reviewed SDK read tools and the reviewed
  `create_activity_draft` pending-action tool.
- MCP network-server serving behind the runtime guard/config/status metadata.
- MCP network-server serving behind the JSON config-file metadata.
- MCP serving behind the one-shot JSON-RPC metadata probe.
- TCP, HTTP, SSE, socket, or other network transport loops for JSON-RPC handling.
- Direct write-like, unreviewed, or non-SDK tool execution through `tools/call`.
- Model-provider integrations.
- Internet or cloud requirements.
- Raw SQL access for MCP clients.
- File-system or shell tools.
- Token/secret UI, listener UI, or MCP runtime UI.
- SDK direct write methods or SDK proposed-action decision/execution methods.
- General direct execution of approved proposed actions beyond the supported
  `create_activity_draft` core path.

## Required Security Gates For Future MCP Work

Future MCP implementation work must include these gates before acceptance:

- Default to localhost-only binding, with no remote network exposure by default.
- Re-validate localhost-only binding at the point a future listener is
  implemented; the current guard validates configuration only and does not bind.
- Require explicit user enablement before any MCP server process or listener starts.
- Require explicit user review before granting a client access to any tool.
- Keep tool access behind external client records and per-tool permission rows.
- Route all data access through `crm-core` services, not raw SQL.
- Do not expose raw SQL, arbitrary file, process, or shell execution tools.
- Treat CRM content as untrusted input at the prompt-injection boundary.
- Audit every client access attempt and every allowed read, draft, decision, and future execution event.
- Keep write-like external-client operations in `draft_only` proposed-action flow unless they are handled by a reviewed, narrow core execution path such as `create_activity_draft`.
- Store no tokens or secrets unless a dedicated security design covers creation, storage, rotation, revocation, and auditability.

## Future MCP Acceptance Checklist

A future MCP implementation should not be accepted until all of the following are true:

- [ ] `crates/crm-mcp` is implemented as an optional package and is not required for normal desktop/core operation.
- [x] `crates/crm-mcp` has an offline SDK-backed read-only catalog for the
      initial reviewed tool names, with all entries marked
      `runtime_enabled: false`.
- [ ] No MCP server starts unless the user explicitly enables it.
- [x] `crates/crm-mcp` has a disabled-by-default runtime guard/config/status
      model with localhost-only default configuration and loopback validation
      when enabled.
- [x] `crates/crm-mcp` has a metadata-only one-shot JSON-RPC handler for
      `initialize`, `tools/list`, and notifications, with `tools/call`
      explicitly rejected and no serving loop or transport.
- [x] `crates/crm-mcp` has a disabled-by-default config-gated local stdio loop
      that can execute only reviewed SDK read tools and the reviewed
      `create_activity_draft` pending-action tool when local execution context
      is present.
- [ ] The future implemented listener is localhost-only.
- [ ] No cloud, internet, or model-provider dependency is required for core CRM use.
- [x] Current MCP read tools call `crm-core` services through `crm-sdk` instead
      of direct SQL.
- [x] MCP read tools reuse the reviewed `crm-sdk` tool constants and
      permission-gated read methods where the SDK supports the requested tool.
- [ ] No raw SQL, shell, process, or arbitrary file tools are exposed.
- [ ] External client enablement and per-tool permission grants have explicit review UI.
- [ ] Only `disabled`, `read_only`, and `draft_only` are active unless a future sprint implements broader modes with tests and docs.
- [ ] MCP runtime read access is audited with enough context to identify client,
      tool, entity scope, and result status. Current read-only `tools/call`
      dispatch records `crm-core` external-client permission evaluation audit
      evidence for attempted client, tool, allowed flag, reason, and status, but
      result-scope audit detail remains future work.
- [x] Draft proposed actions are audited and visible in Pending Actions for the
      reviewed `create_activity_draft` MCP/SDK path.
- [ ] Approved proposed actions execute only when a reviewed, supported core execution path exists; unsupported actions remain pending with explicit errors.
- [ ] Prompt-injection boundaries are documented and tested with CRM content treated as untrusted.
- [ ] Security documentation covers credential handling if tokens or secrets are introduced.
- [ ] Verification includes unit tests, integration tests, no raw SQL boundary regressions, no direct Svelte `invoke()` regressions, and a clean `git fsck`.
