# Transitional Architecture Debt

This file tracks foundation debt that is intentionally left in place until the
next focused cleanup sprint. It is not a product roadmap.

## TCH-001: Move Remaining SQL Out Of `crm_engine`

Status: open

The target architecture says SQL belongs only in `crates/crm-core/src/storage`.
Most Tauri command SQL has been moved behind `crm-core` services and storage
repositories, but these transitional modules still contain direct SQL:

- `crates/crm-core/src/crm_engine/activities.rs`
- `crates/crm-core/src/crm_engine/contacts.rs`
- `crates/crm-core/src/crm_engine/pipeline.rs`
- `crates/crm-core/src/crm_engine/search.rs`

Cleanup direction:

- Move duplicate/contact discovery SQL into `storage::contacts`.
- Move activity stats SQL into `storage::activities` or `storage::dashboard`.
- Move pipeline conversion SQL into `storage::deals` or a pipeline repository.
- Move global search SQL into dedicated search repository functions.
- Keep `crm_engine` focused on pure business rules and calculations.

This sprint does not perform the migration because the tooling/lockfile fix,
service split, and readiness tests are the reliability priority.

## TCH-002: Frontend Tooling Depends On Materialized Local Files

Status: monitor

The Svelte/Vite/Vitest runtime hang was caused by local APFS/iCloud dataless
files in `apps/desktop/src` and `scripts`. The package imports resolved and
`npm ci` succeeded, but `svelte-check`, Vitest, `rg`, and `sed` blocked while
macOS fetched source file contents on first read.

Current mitigation:

- Keep timeout wrappers around frontend checks as hang protection only.
- Use `npm ci` from the repo root as the clean install path.
- If the hang returns on this machine, materialize the workspace files before
  rerunning checks, for example by reading `apps/desktop/src` and `scripts` once.

Follow-up:

- Avoid committing cloud-provider duplicate files such as `* 2.rs`.
- Keep generated/cache directories ignored so scanners do not walk old local
  build artifacts.

## TCH-003: Organization Bridge Still Keeps Legacy Contact Shape

Status: open

Organizations are now first-class rows in `organizations`, and migration v3
bridges legacy `contact_type = 'organization'` rows into that table without
deleting or rewriting the original contacts. Contact linking now writes
`contacts.organization_id` and mirrors `contacts.org_id`.

Remaining debt:

- The old contacts table still contains organization-specific columns and
  legacy organization contact rows.
- Existing contact create/edit flows still preserve the free-text
  `org_name` behavior for compatibility.
- Some backend contact paths still carry both `org_id` and `organization_id`
  until a later non-destructive cleanup decides how to preserve local data.

Cleanup direction:

- Keep organization CRUD in `storage::organizations` and `services::organizations`.
- Move remaining organization-as-contact UX into normalized organization flows
  only after export/backup and data-preservation decisions are explicit.
- Do not destructively normalize the `contacts` table until users have a clear
  migration/rollback path.
