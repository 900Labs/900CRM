# Import And Export

Date: 2026-06-25

This document describes the current import/export behavior in 900CRM. It is
based on the desktop UI, Tauri command layer, and `crm-core` services as they
exist today.

## Current Scope

900CRM currently supports local CSV import/export and local JSON import/export
for:

- contacts;
- deals;
- organizations.

Import and export use local files selected by the user. There is no cloud import
service, no remote export destination, and no automatic upload.

JSON import is intentionally limited to direct local `.json` files containing a
flat array of objects. Source object keys can match supported target fields
directly or be mapped in the import wizard before duplicate preflight.

## Import Entry Point

The import/export modal is available from the desktop UI. The import tab asks
the user to choose an entity type, import format, and local file.

For CSV contacts, deals, and organizations, the current wizard flow is:

1. Select entity type and CSV file.
2. Preview parsed headers and the first rows.
3. Map source CSV columns to supported CRM fields.
4. Run duplicate preflight checks.
5. Review duplicate warnings.
6. Confirm import.
7. Review the import summary.

Mapped imports require a desktop-selected file path so the Rust backend can read
the same CSV file. The browser file-input fallback can preview text, but mapped
backend import/preflight requires the desktop file picker path.

For JSON contacts, deals, and organizations, the current flow reuses the same
mapping and duplicate-gated confirmation concepts as CSV:

1. Select entity type and JSON format.
2. Select a local `.json` file.
3. Preview the parsed JSON rows in the browser.
4. Map source JSON fields to supported CRM fields.
5. Run duplicate preflight checks.
6. Review duplicate warnings if any are found.
7. Confirm import.
8. Review the import summary.

## CSV Import Parsing

Frontend preview parsing handles:

- quoted fields;
- escaped double quotes;
- commas and newlines inside quoted fields;
- Windows, Unix, and older Mac line endings;
- empty fields and trailing commas.

Backend CSV parsing uses the Rust `csv` crate with headers enabled, trimmed
fields, and flexible row widths.

Rows with a blank required field are skipped by the parser:

- contacts require `first_name`;
- deals require `title`;
- organizations require `name`.

## JSON Import Parsing

Direct JSON import accepts a top-level array of flat objects using the same
fields as the matching JSON export:

- contacts: `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
  `city`, `country`, and `notes`;
- deals: `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`;
- organizations: `name`, `email`, `phone`, `website`, `address_line1`,
  `address_line2`, `city`, `region`, `country`, `postal_code`, and
  `description`.

JSON rows are parsed into the same flat row structs used by CSV import and then
sent through the same `crm-core` create/import paths as CSV rows. When source
keys are nonstandard, the JSON mapping step maps object keys to those same flat
target fields before duplicate preflight or import. Rows with a blank mapped
required field are skipped before create attempts, matching CSV behavior.

JSON row numbers are reported with the same data-row offset as CSV imports: the
first JSON array item is row 2.

The desktop UI parses JSON files through a read-only preview command before
mapping, duplicate preflight, or import confirmation. The preview shows source
object keys and up to the first five JSON object rows. Preview parsing does not
create automatic backups, create or update CRM rows, or write audit/sync rows.
Invalid JSON, non-object rows, or an unsupported shape blocks the duplicate
preflight/import path and shows a preview error.

## Contact CSV

Supported contact import target fields:

| Field | Required | Notes |
|---|---|---|
| `first_name` | Yes | Blank rows are skipped. |
| `last_name` | No | Optional person field. |
| `org_name` | No | Legacy organization/company display field. |
| `email` | No | Validated by contact creation logic when present. |
| `phone` | No | Imported as text. |
| `address` | No | Legacy single-line address field. |
| `city` | No | Imported as text. |
| `country` | No | Imported as text. |
| `notes` | No | Imported into the legacy contact notes field. |

Mapped contact import can accept arbitrary source headers as long as each source
header is mapped to a supported target field or skipped. The UI suggests common
aliases such as `firstname`, `givenname`, `company`, `emailaddress`, and
`telephone`.

Contact export writes these same fields with a header row. It exports up to
100,000 active contacts sorted by `first_name` ascending through the contact
listing path.

## Organization CSV

Supported organization import target fields:

| Field | Required | Notes |
|---|---|---|
| `name` | Yes | Blank rows are skipped. |
| `email` | No | Validated by organization creation logic when present. |
| `phone` | No | Imported as text. |
| `website` | No | Imported as text. |
| `address_line1` | No | Optional address line. |
| `address_line2` | No | Optional address line. |
| `city` | No | Imported as text. |
| `region` | No | State, province, or region. |
| `country` | No | Imported as text. |
| `postal_code` | No | Imported as text. |
| `description` | No | Freeform organization description. |

Mapped organization import can accept arbitrary source headers as long as each
source header is mapped to a supported target field or skipped. The UI suggests
common aliases such as `company`, `organisation`, `url`, `state`, `province`,
`zipcode`, and `notes`.

Organization export writes these same fields with a header row. It exports
active organizations sorted by name through the organization listing path.

## Deal CSV

Supported deal import fields:

| Field | Required | Notes |
|---|---|---|
| `title` | Yes | Blank rows are skipped. |
| `value` | No | Parsed as a decimal number; invalid or missing values become `0.0`. |
| `currency` | No | Defaults to `USD` if omitted. |
| `stage` | No | Defaults to `Lead` if omitted. |
| `expected_close` | No | Stored as provided by the create path. |
| `notes` | No | Imported as deal notes. |

Mapped deal import can accept arbitrary source headers as long as each source
header is mapped to a supported target field or skipped. The UI suggests common
aliases such as `opportunity`, `amount`, `pipeline stage`, `close date`, and
`memo`.

Deal import does not map imported deals to contacts or organizations. Imported
deals pass `None` for `contact_id` and `organization_id`.

Deal export writes `title`, `value`, `currency`, `stage`, `expected_close`, and
`notes` with a header row. Values are formatted with two decimal places.

## JSON Import And Export

JSON import and export are available for contacts, deals, and organizations from
the same import/export modal as CSV. Import uses an open dialog for `.json`
files. Export uses a save dialog for `.json` files.

JSON exports are pretty-printed arrays of objects. They use the same flat fields
as the matching CSV export:

- contacts: `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
  `city`, `country`, and `notes`;
- deals: `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`;
- organizations: `name`, `email`, `phone`, `website`, `address_line1`,
  `address_line2`, `city`, `region`, `country`, `postal_code`, and
  `description`.

JSON export uses the same active-row listing boundaries as CSV export:

- contacts export up to 100,000 active contacts sorted by `first_name`
  ascending through the contact listing path;
- deals export active deals through the deal listing path;
- organizations export active organizations through the organization listing
  path.

JSON export does not include record IDs, timestamps, deleted rows, device IDs,
relationship rows, custom fields, separate note records, tags, activities, audit
log entries, proposed actions, external clients, permissions, settings, or
backup metadata.

JSON import has the same entity scope. It does not import record IDs, timestamps,
deleted rows, device IDs, relationship rows, custom fields, separate note
records, tags, activities, audit log entries, proposed actions, external
clients, permissions, settings, or backup metadata.

JSON import preview is read-only and browser-visible in the import/export modal.
It shows source object keys before the mapping step. Matching supported JSON
keys are auto-mapped, and nonstandard source keys can be mapped to the
supported flat import fields before duplicate preflight and confirmed import.

## Duplicate Preflight

Duplicate preflight is currently implemented for contacts, deals, and
organizations.

Contact preflight checks active contacts for:

- case-insensitive exact email matches;
- exact phone matches after trimming.

Organization preflight checks active organizations for:

- case-insensitive exact name matches;
- case-insensitive exact email matches;
- exact phone matches after trimming.

Deal preflight checks active deals for:

- case-insensitive exact title matches after trimming the imported title.

Preflight returns warnings with the source row number, match type, source value,
existing record ID, existing display label, and a human-readable reason. CSV
imports use CSV data row numbers. JSON imports use the same offset as JSON
import errors: the first array item is row 2.

Duplicate preflight is read-only. It does not create contacts, deals, or
organizations, and it does not write audit rows or sync changelog rows.

Preflight warnings do not block import automatically. The UI lets the user
continue despite warnings. The import then attempts to create each row and
reports created, skipped, and error counts.

## Duplicate Auto-Merge

Contact, deal, and organization imports include an explicit opt-in checkbox to
merge duplicate import rows into matching existing records. The option is
available for CSV, mapped CSV, JSON, and mapped JSON imports. It is off by
default.

Duplicate preflight still runs before confirmation when auto-merge is enabled.
The confirmation copy states that duplicate warnings will be merged into
matching existing records where safe.

When enabled, duplicate matching uses the same active-record fields as
preflight:

- contacts match by case-insensitive email or trimmed phone;
- deals match active deals by exact title after trimming, case-insensitively;
- organizations match by case-insensitive name, case-insensitive email, or
  trimmed phone.

If all matching rules identify one existing active record, the import row fills
blank fields on that record without overwriting existing non-empty values.
Existing IDs and active rows are preserved, and updates route through the
existing `crm-core` update services so sync and audit behavior stays aligned
with normal edits. Non-duplicate rows continue to create normally.

Deal auto-merge is intentionally conservative. Existing deal titles are never
overwritten. Imported `expected_close` and `notes` values fill only blank
existing fields. Imported `value` fills only when the existing value is `0.0`
and the imported value parses to a nonzero number. This value rule exists
because the flat CSV/JSON row shape stores deal value as text and cannot
distinguish a blank value from an explicit zero once it reaches the import row.
Imported `currency` and `stage` do not overwrite existing deal values, including
the default `USD` and `Lead` values.

If one import row matches multiple existing records, that row is skipped with a
row-numbered error rather than merged unsafely.

## Row-Level Results

Imports return:

- `created`: number of rows successfully created;
- `merged`: number of duplicate rows folded into existing records;
- `skipped`: number of rows that failed during creation or duplicate
  auto-merge handling after parsing;
- `errors`: row-numbered error strings from failed create attempts or skipped
  ambiguous duplicate auto-merge rows.

Rows skipped during CSV or JSON parsing because the required field is blank are
not included in the import result as row-level errors.

Successful imported rows are written through normal `crm-core` create paths.
Those paths record sync changelog entries and audit evidence. The import
service also records an `import_row` audit entry for each successfully imported
contact, deal, or organization. Successful duplicate auto-merge rows are
updated through normal update services and also receive an `import_row_merge`
audit entry.

Desktop CSV, mapped CSV, JSON, and mapped JSON imports for contacts, deals, and
organizations first create an automatic local backup through
`CrmCore::create_local_backup`. The backup is created immediately before import
rows are written, and backup failure stops the import. The import summary shown
in the UI includes the created backup path.

When the import summary includes an automatic pre-import backup path, the UI can
restore that backup directly from the summary. The restore action validates the
backup folder first, asks for explicit destructive confirmation, then calls the
same local restore path used by Settings. If validation or restore fails, the
summary remains visible with the failure message.

## Row-Level Import Rollback

Completed desktop imports for contacts, deals, and organizations return a
self-contained row-level rollback plan when the import creates rows or changes
fields through duplicate auto-merge. The current Import/Export summary can use
that plan to request rollback of the just-completed import.

Rollback is available for CSV, mapped CSV, JSON, and mapped JSON imports. It is
limited to the same flat contact, deal, and organization fields supported by
import/export:

- created rows are soft-deleted through the existing `crm-core` delete service
  for the matching entity;
- duplicate auto-merge rows restore only the fields that the import merge
  changed, through the existing `crm-core` update service for the matching
  entity.

Rollback is conflict-safe. Before applying each row action, `crm-core` compares
the current active row to the post-import state recorded in the rollback plan.
If the row was edited, deleted, or otherwise no longer matches the expected
post-import state, that row is skipped and reported as a row-level error. Later
rollback rows continue to run.

The rollback result reports:

- `rolled_back`: number of row actions applied;
- `skipped`: number of row actions skipped, including conflicts and rows that
  were already deleted;
- `errors`: row-level error objects with entity type, entity ID, row number,
  code, and message.

The rollback plan is intentionally summary-scoped. It is not persisted across an
app restart and is not available after the current import summary is dismissed.
The UI disables the row-level rollback button after a successful rollback
command response. Reusing the same plan is still safe: already-rolled-back
created rows are reported as skipped, and already-restored merge rows no longer
match the post-import expected state.

Row-level rollback is not full database restore. It does not roll back
relationships, custom fields, notes-as-records, tags, activities, audit logs,
sync changelog rows, proposed actions, external clients, permissions, settings,
backup metadata, or schema changes.

## Export Behavior

Exports write local CSV or JSON files selected by the user through the save
dialog.

Exported files are plain text. They may contain personal, customer, sales, and
business data. 900CRM does not encrypt exported files, upload them, or apply
access controls after they are written.

The Settings Data Management export action displays this disclosure before the
user opens the export flow. Store exported files containing sensitive data in a
trusted or encrypted location.

## Relationship To Backups

CSV import/export and JSON import/export are not a backup system.

- CSV export is useful for portability, spreadsheet use, and migration.
- JSON import/export is useful for portability and inspection of the supported
  flat entity fields.
- Local backup creates a SQLite snapshot plus metadata and is the safer way to
  preserve full application state before a destructive restore or risky import.
- Supported desktop imports automatically create a local pre-import backup
  before writing rows.
- Automatic pre-import backups are stored under the platform app data directory
  at `pre-import-backups/<timestamp-and-sequence>/`.
- Current import summaries can run row-level rollback for created rows and
  duplicate auto-merge field changes from the just-completed import.
- Import summaries can validate and restore their automatic pre-import backup
  after explicit destructive confirmation.
- Duplicate auto-merge writes use the same automatic pre-import backup guard as
  normal import creates.
- Restore validation applies to local backups, not CSV or JSON exports.

Before large imports, confirm the automatic backup path in the import summary
and move or copy that backup to durable encrypted storage if needed. Row-level
rollback is useful for undoing flat rows from the current summary, while
restoring the pre-import backup replaces the current local database and remains
the full-state escape hatch. See [Backup and Restore](BACKUP_RESTORE.md) for
the validated backup workflow.

## Not Yet Implemented

The following are not implemented in the current import/export surface:

- Remote import/export endpoints.
- Cloud storage export destinations.
- Scheduled export.
- MCP/AI-driven import behavior.
- Sync-server upload or download as part of import/export.
- Custom field import/export.
- Notes, tags, activities, audit log, proposed actions, external clients, or
  permissions import/export.
