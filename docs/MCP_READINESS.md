# MCP Readiness Baseline

Date: 2026-06-24

This document records the current Model Context Protocol (MCP) readiness state for 900CRM. It is a baseline for future implementation work, not a description of active MCP behavior.

## Current Status

- 900CRM core has no built-in AI agent.
- Normal desktop and `crm-core` operation do not require internet access, cloud services, or model providers.
- MCP is intended to be a separate optional package boundary, not a required desktop/core dependency.
- `crates/crm-mcp` currently contains only a placeholder binary and is not an implemented MCP server.
- The desktop app and `crm-core` do not start an MCP server, bind a localhost listener, expose prompts/resources/tools, or manage MCP tokens/secrets.

## Architecture Boundary

Future MCP tools must call `crm-core` service APIs. They must not read or write the SQLite database directly, and they must not bypass the existing storage, validation, audit, sync-log, and proposed-action boundaries.

The intended call path is:

1. Optional MCP package receives a local client request.
2. MCP package validates the client, requested tool, and configured mode.
3. MCP package calls explicit `crm-core` services.
4. `crm-core` applies normal domain validation and records audit evidence.
5. Write-like external-client requests create proposed actions for user review unless a narrow reviewed execution path explicitly supports the action.
6. The current reviewed execution path is limited to core approval of `create_activity_draft` proposed actions; it does not add an MCP runtime.

## Active Readiness Surfaces

The following data and API surfaces exist today:

- External client records with `name`, `client_type`, `enabled`, and `permission_mode`.
- External client permission rows keyed by `(client_id, tool_name)`.
- Permission evaluation for the initial modes `disabled`, `read_only`, and `draft_only`.
- Proposed action rows with pending, approved, rejected, and other schema-reserved lifecycle states.
- Audit log entries for accountable changes and proposed-action decisions.
- Pending Actions UI for reviewing proposed actions.
- Audit Log UI for inspecting recorded activity.
- Proposed action approve/reject APIs and UI controls.
- Core approval execution for pending proposed actions where `tool_name` or
  `action_type` is `create_activity_draft`.

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

No UI currently exposes permission grant review or activation controls. Existing Settings surfaces can list and create disabled external client placeholders, but they do not enable clients, issue credentials, or grant tool permissions.

## Current Non-Goals

The current codebase intentionally does not include:

- MCP server startup.
- A localhost MCP listener.
- MCP authentication tokens, client secrets, or credential storage.
- Prompt, resource, or tool implementations.
- Model-provider integrations.
- Internet or cloud requirements.
- Raw SQL access for MCP clients.
- File-system or shell tools.
- Permission-grant UI.
- General direct execution of approved proposed actions beyond the supported
  `create_activity_draft` core path.

## Required Security Gates For Future MCP Work

Future MCP implementation work must include these gates before acceptance:

- Default to localhost-only binding, with no remote network exposure by default.
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
- [ ] No MCP server starts unless the user explicitly enables it.
- [ ] The default listener is localhost-only.
- [ ] No cloud, internet, or model-provider dependency is required for core CRM use.
- [ ] Every MCP tool calls `crm-core` services instead of direct SQL.
- [ ] No raw SQL, shell, process, or arbitrary file tools are exposed.
- [ ] External client enablement and per-tool permission grants have explicit review UI.
- [ ] Only `disabled`, `read_only`, and `draft_only` are active unless a future sprint implements broader modes with tests and docs.
- [ ] Read access is audited with enough context to identify client, tool, entity scope, and result status.
- [ ] Draft proposed actions are audited and visible in Pending Actions.
- [ ] Approved proposed actions execute only when a reviewed, supported core execution path exists; unsupported actions remain pending with explicit errors.
- [ ] Prompt-injection boundaries are documented and tested with CRM content treated as untrusted.
- [ ] Security documentation covers credential handling if tokens or secrets are introduced.
- [ ] Verification includes unit tests, integration tests, no raw SQL boundary regressions, no direct Svelte `invoke()` regressions, and a clean `git fsck`.
