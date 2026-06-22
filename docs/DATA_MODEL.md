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

## Deferred

- Final normalized `contacts` rebuild with only target columns.
- Removal of legacy organization-as-contact rows and `contacts.org_id`.
- `pipeline_stages`, `deal_contacts`, and `activity_links` normalization.
- JSON export and backup behavior beyond existing CSV flows.
- Sync server behavior.
