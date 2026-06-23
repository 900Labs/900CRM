# Backup and Restore Baseline

This document is the release baseline for the local backup/restore stack. It
describes the implemented behavior only; scheduled backups, cloud backup,
encryption, sync, MCP, and AI workflows are out of scope.

## Current Surface

Local backup/restore is available from Settings > Data Management > Local
Backup. The UI calls thin frontend wrappers in `apps/desktop/src/lib/api/backup.ts`,
which invoke these Tauri commands:

- `create_local_backup`
- `validate_local_backup`
- `restore_local_backup_to_app_data`

The Tauri command handlers are adapters over `crm-core`; they do not own SQL or
backup business rules.

## Backup Creation

`create_local_backup` writes a full local database backup into the folder chosen
by the user. The backup directory contains:

- `900crm.db`: standalone SQLite database snapshot.
- `metadata.json`: backup metadata with `created_at`, `app_version`,
  `schema_version`, `device_id`, backup format version, and database filename.

Backups stay on the local filesystem. The app does not upload or transmit backup
files.

## Validation and Integrity Checks

`validate_local_backup` rejects backups before restore if any required
compatibility or integrity check fails. Validation includes:

- Backup metadata shape and backup format compatibility.
- Application/schema compatibility checks.
- Required readiness table checks.
- SQLite integrity validation using `PRAGMA quick_check` against a temporary
  copied database, not the source backup file.

The temporary-copy validation path avoids mutating the backup and keeps
validation safe for WAL/FTS-backed database files.

## Restore Safety

Restore is destructive because it replaces the current local app database. The
stack has two confirmation layers:

- The Settings UI first validates a selected backup and then shows an explicit
  restore confirmation panel.
- The backend restore command requires `confirm_destructive_restore = true`.
  Calling restore without that flag must fail.

The desktop command closes the active `CrmCore`, performs the confirmed restore
through `crm-core`, and then reopens `CrmCore` against the restored app data.
Runtime smoke tests must use disposable app data; do not test confirmed restore
against real user data.

## Tauri Capability Requirements

The desktop app uses Tauri v2 capabilities in
`apps/desktop/src-tauri/capabilities/default.json`. Current backup and
import/export flows require:

- `core:default` for the local app window and core Tauri APIs.
- `dialog:allow-open` for backup folder selection and import file selection.
- `dialog:allow-save` for CSV export save dialogs.
- `fs:allow-read-text-file` plus the configured file scope for CSV import.

Do not add shell, notification, broad filesystem, MCP, AI, or sync permissions
unless current code starts using those surfaces and the permission is reviewed in
the same sprint.

## macOS GUI Automation Limitation

The real Tauri runtime has been smoke-tested with disposable app data and the
Settings route rendered successfully. On this machine, macOS automation did not
reliably drive the native folder picker: `System Events` returned `-10827`, and
lower-level click injection did not consistently land in the WebView.

Because of that OS automation limitation, release verification should combine:

- Browser/frontend tests for API wrapper behavior.
- Rust tests for backup validation and restore safety.
- Real Tauri launch logs proving capability parsing and no IPC permission
  denial.
- Manual GUI smoke for folder picker selection when macOS automation cannot
  drive native dialogs.

## Verification Checklist

Run these checks from the repository root:

```bash
npm run check
npm run test
npm run build
cargo check --workspace
cargo test --workspace
git fsck --full --no-progress
git status --short --branch
```

For runtime smoke, use disposable app data and a disposable backup directory.
Verify:

- Settings > Data Management renders in the Tauri desktop app.
- Create Backup opens a folder picker and writes `900crm.db` plus
  `metadata.json`.
- Validate Backup succeeds on that backup.
- Restore validation shows an explicit confirmation step.
- Cancel restore does not call the destructive restore path.
- Confirmed restore is tested only against disposable app data.
- Tauri logs show no capability or IPC permission-denied errors.
