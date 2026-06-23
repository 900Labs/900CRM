# 900CRM Data Model

The build spec is the target data-model source of truth. This sprint starts the
foundation without performing a destructive rewrite of existing user data or UI
flows.

## Principles

- SQLite remains the local database.
- UUID text primary keys are the target for domain tables.
- Soft deletes use `deleted_at`.
- Timestamps use ISO 8601 text.
- `device_id` records mutation origin for offline-first workflows.
- Business services write audit rows and sync changelog rows for mutations.

## Implemented Foundation

Migration v2 creates these target foundation tables:

- `organizations`
- `tag_links`
- `audit_log`
- `external_clients`
- `external_client_permissions`
- `proposed_actions`

Migration v2 also extends existing v1 tables for forward compatibility:

- `contacts`: adds `organization_id`, `title`, `whatsapp`, address-line fields,
  `region`, `postal_code`, `source`, and `description`.
- `notes`: adds `body`.
- `tags`: adds `updated_at`, `deleted_at`, and `device_id`.
- `sync_changelog`: adds `operation` and `synced_at`.

The existing v1 tables remain active so the desktop app can still launch and
existing commands keep their frontend contract while the backend boundary moves
to `crm-core`.

Migration v3 adds the first non-destructive normalization bridge:

- Legacy `contacts` rows with `contact_type = 'organization'` are copied into
  `organizations` with the same `id` and `source = 'legacy_contact'`.
- Legacy person-contact links in `contacts.org_id` are mirrored into
  `contacts.organization_id` when the referenced organization exists.
- The legacy contact rows are not deleted or rewritten, so existing contact
  screens and imports remain compatible during the transition.

## Target Core Tables

The target schema separates people from organizations:

- `organizations`: company, NGO, school, clinic, or office-level records.
- `contacts`: person records with optional `organization_id`.
- `notes`: freeform notes linked by `entity_type` and `entity_id`.
- `tags` and `tag_links`: labels across contacts, organizations, deals,
  activities, and notes.
- `audit_log`: user-visible accountability log.
- `sync_changelog`: local mutation log for future sync, not a sync server.
- `external_clients`: optional future MCP/external clients, disabled by default.
- `external_client_permissions`: narrow tool-level permission rows.
- `proposed_actions`: draft/approval queue for future external-client writes.

## Current Service Boundary

Organization mutations now go through `crm-core` services and repositories:

- `create_organization`, `update_organization`, and `delete_organization` write
  `organizations`, `audit_log`, and `sync_changelog` in one service call.
- `link_contact_to_organization` updates the normalized `organization_id` and
  mirrors the legacy `org_id` column while contact UI migration is in progress.
- Desktop Tauri commands remain thin IPC wrappers over `crm-core`; they do not
  own SQL.

## Normalization Migration Readiness

`crm-core` now exposes a non-mutating normalization migration preflight for the
remaining contacts/organizations cleanup. The report counts:

- Active legacy organization contacts still stored in `contacts` with
  `contact_type = 'organization'`.
- Active contacts that still have `org_id` but no normalized
  `organization_id`.
- Active contacts whose `org_id` or `organization_id` points at a missing,
  deleted, or wrong-type organization record.
- Whether the backup/restore baseline is available on the local database before
  any destructive migration is considered.

The preflight is intentionally read-only. It does not rewrite contacts, remove
legacy columns, or authorize a destructive normalization.

## Backup and Restore Foundation

`crm-core` exposes local backup primitives for future high-risk migrations, and
the desktop app exposes them through Settings > Data Management:

- `create_local_backup` writes a full standalone SQLite snapshot plus
  `metadata.json`.
- Backup metadata records `created_at`, `app_version`, `schema_version`, and
  `device_id`.
- `validate_local_backup` checks metadata compatibility, database schema
  version, required core table presence, and SQLite `quick_check` integrity
  before any restore path can proceed.
- `restore_local_backup_to_app_data` remains conservative: replacing an app
  database requires an explicit confirmation flag from the caller and only runs
  after the active core connection has been closed.
- Desktop Tauri commands stay thin; backup business rules and validation live in
  `crm-core`.

This foundation is intended to protect users before any destructive
contact/organization normalization is considered.

## Deferred

- Final normalized `contacts` rebuild with only target columns.
- Removal of legacy organization-as-contact rows and `contacts.org_id`.
- `pipeline_stages`, `deal_contacts`, and `activity_links` normalization.
- Backup scheduling, backup encryption, and cloud/remote backup behavior.
- Sync server behavior.
