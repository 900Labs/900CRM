# Import And Export

Date: 2026-06-24

This document describes the current import/export behavior in 900CRM. It is
based on the desktop UI, Tauri command layer, and `crm-core` services as they
exist today.

## Current Scope

900CRM currently supports local CSV import/export and local JSON export for:

- contacts;
- deals;
- organizations.

Import and export use local files selected by the user. There is no cloud import
service, no remote export destination, and no automatic upload.

JSON support is export-only. JSON import is not implemented.

## Import Entry Point

The import/export modal is available from the desktop UI. The import tab asks
the user to choose an entity type and a CSV file.

For contacts, deals, and organizations, the current wizard flow is:

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

## CSV Parsing

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

## JSON Export

JSON export is available for contacts, deals, and organizations from the same
export modal as CSV. The user selects a local `.json` path in the save dialog.

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

Preflight returns warnings with the CSV row number, match type, CSV value,
existing record ID, existing display label, and a human-readable reason.

Preflight warnings do not block import automatically. The UI lets the user
continue despite warnings. The import then attempts to create each row and
reports created, skipped, and error counts.

## Row-Level Results

Imports return:

- `created`: number of rows successfully created;
- `skipped`: number of rows that failed during creation after parsing;
- `errors`: row-numbered error strings from failed create attempts.

Rows skipped during CSV parsing because the required field is blank are not
included in the import result as row-level errors.

Successful imported rows are written through normal `crm-core` create paths.
Those paths record sync changelog entries and audit evidence. The import
service also records an `import_row` audit entry for each successfully imported
contact, deal, or organization.

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

CSV import/export and JSON export are not a backup system.

- CSV export is useful for portability, spreadsheet use, and migration.
- JSON export is useful for portability and inspection of the supported flat
  entity fields.
- Local backup creates a SQLite snapshot plus metadata and is the safer way to
  preserve full application state before a destructive restore or risky import.
- Import does not create an automatic backup before writing rows.
- Restore validation applies to local backups, not CSV or JSON exports.

Before large imports, create a local backup from Settings. See
[Backup and Restore](BACKUP_RESTORE.md) for the validated backup workflow.

## Not Yet Implemented

The following are not implemented in the current import/export surface:

- JSON import.
- Contact or organization duplicate auto-merge during import.
- Deal duplicate auto-merge during import.
- Import rollback for a completed multi-row import.
- Automatic backup before import.
- Remote import/export endpoints.
- Cloud storage export destinations.
- Scheduled export.
- MCP/AI-driven import behavior.
- Sync-server upload or download as part of import/export.
- Custom field import/export.
- Notes, tags, activities, audit log, proposed actions, external clients, or
  permissions import/export.
