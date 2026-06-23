# Transitional Architecture Debt

This file tracks foundation debt that is intentionally left in place until the
next focused cleanup sprint. It is not a product roadmap.

## TCH-001: Move Remaining SQL Out Of `crm_engine`

Status: resolved in `codex/crm-engine-storage-boundary-cleanup`

The target architecture says SQL belongs only in `crates/crm-core/src/storage`.
The remaining direct database queries have been moved out of these legacy
business-logic modules:

- `crates/crm-core/src/crm_engine/activities.rs`
- `crates/crm-core/src/crm_engine/contacts.rs`
- `crates/crm-core/src/crm_engine/pipeline.rs`
- `crates/crm-core/src/crm_engine/search.rs`

Repository ownership after cleanup:

- Duplicate/contact discovery queries live in `storage::contacts`.
- Activity stats queries live in `storage::activities`.
- Pipeline age and summary queries live in `storage::deals`.
- Global search queries live in `storage::search`.
- `crm_engine` remains responsible for validation, scoring, result formatting,
  and cross-repository orchestration.

Focused Rust tests now cover the moved query paths for activity stats, duplicate
detection, pipeline age filtering, and unified search composition.

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

Status: open, backup prerequisite narrowed

Organizations are now first-class rows in `organizations`, and migration v3
bridges legacy `contact_type = 'organization'` rows into that table without
deleting or rewriting the original contacts. Contact linking now writes
`contacts.organization_id` and mirrors `contacts.org_id`.

The backup foundation now provides `crm-core` methods for full local SQLite
backup creation, metadata and SQLite integrity validation, desktop command/API
surfaces, Settings UI controls, and confirmed-only restore. That reduces the
data-preservation risk before future destructive normalization, but it does not
by itself authorize a destructive contacts-table rewrite.

The normalization preflight now reports the remaining migration hazards without
mutating data:

- Active legacy organization contacts still stored in `contacts`.
- Active contacts with `org_id` but no normalized `organization_id`.
- Active contacts whose legacy or normalized organization link points at a
  missing, deleted, or wrong-type organization record.
- Whether the local backup/restore baseline is present before destructive work
  is even considered.

Remaining debt:

- The old contacts table still contains organization-specific columns and
  legacy organization contact rows.
- Existing contact create/edit flows still preserve the free-text
  `org_name` behavior for compatibility.
- Some backend contact paths still carry both `org_id` and `organization_id`
  until a later non-destructive cleanup decides how to preserve local data.

Cleanup direction:

1. Require a fresh local backup and successful backup validation.
2. Run the normalization preflight and resolve invalid organization links first.
3. Mirror any remaining valid `org_id` links into `organization_id` without
   deleting legacy data.
4. Move remaining organization-as-contact UX into normalized organization flows
   only after data-preservation and rollback decisions are explicit.
5. Only after the preflight reports no blockers, plan a separate destructive
   migration for legacy organization contact rows and legacy columns.

Do not destructively normalize the `contacts` table until users have a clear
migration/rollback path and the destructive migration has its own focused
sprint.
