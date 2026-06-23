# 900CRM Privacy

900CRM is designed as an offline-first, local-first CRM.

## Commitments

- No cloud account is required.
- No internet connection is required for core CRM use.
- No telemetry or analytics are sent by the core app.
- CRM data is stored locally in SQLite.
- Optional integrations must be disabled by default.
- External AI/MCP integrations must be separate from the core app.

## Local Storage

The desktop app stores data in the Tauri app data directory for
`com.900labs.crm`. The database file is named `900crm.db`.

The core database uses:

- Local SQLite.
- WAL mode.
- Foreign keys enabled.
- Soft deletes for CRM records.
- Audit log foundations for accountable changes.

## Local Backups

`crm-core` can create a local full-database backup as a standalone SQLite
snapshot plus `metadata.json`. The metadata records backup creation time, app
version, schema version, and device ID so future restore flows can validate
compatibility before replacing any local database.

Backups are not uploaded or transmitted by the core app. They contain the same
CRM data as the local database and should be protected with the same care.
Restore remains explicitly confirmed by the caller and the Settings UI; backup
validation alone does not replace user data. Validation includes metadata,
schema/readiness checks, and SQLite `quick_check` integrity validation before a
restore can proceed.

## AI and MCP Boundary

The core app contains no built-in AI agent. It does not send CRM data to an
external model.

Future MCP integrations must be opt-in, local by default, permissioned, and
audited. Future draft/write actions must use `proposed_actions` rather than
mutating CRM records directly.

## Deferred Privacy Work

- App lock and local encryption are not implemented in this sprint.
- Backup encryption and backup scheduling are not implemented in this sprint.
- Remote MCP binding is out of scope.
- Sync server behavior is out of scope.
- External-client approval UI is out of scope.
