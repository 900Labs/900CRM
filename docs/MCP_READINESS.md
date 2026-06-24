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
5. Any write-like request creates a proposed action for user review unless a future sprint explicitly implements and approves a stricter execution path.

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

Approving a proposed action only changes its decision state. It does not execute the requested action, mutate CRM records, or run MCP/client code.

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
- Direct execution of approved proposed actions.

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
- Keep write-like operations in `draft_only` proposed-action flow until a separate reviewed sprint implements execution semantics.
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
- [ ] Approved proposed actions still do not execute unless an execution sprint adds a reviewed execution path.
- [ ] Prompt-injection boundaries are documented and tested with CRM content treated as untrusted.
- [ ] Security documentation covers credential handling if tokens or secrets are introduced.
- [ ] Verification includes unit tests, integration tests, no raw SQL boundary regressions, no direct Svelte `invoke()` regressions, and a clean `git fsck`.
