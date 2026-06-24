# Backup and Restore

900CRM stores CRM data locally in SQLite. The backup surface creates a portable local backup directory and restores it only after validation plus explicit user confirmation.

## Backup contents

A local backup directory contains:

- `900crm.db` - a standalone SQLite snapshot of the CRM database.
- `metadata.json` - backup format, app version, schema version, device id, creation time, and database file name.

Backup creation refuses to overwrite an existing `900crm.db` or `metadata.json` in the selected folder. Use an empty folder for each new backup.

## Create a backup

1. Open `Settings`.
2. In `Data Management`, choose a backup folder.
3. Select `Create Backup`.
4. Move the completed backup folder to durable storage, such as an encrypted external drive, if the machine is at risk.

Backups are local files. 900CRM does not upload them, sync them, send them to
any 900 Labs service, or encrypt the backup folder.

The Settings Data Management backup panel warns before backup actions that
local backup folders are unencrypted local files. Store backup folders
containing sensitive data in a trusted or encrypted location.

## Validate a backup

Validation is a read-only safety check. It verifies:

- backup metadata is readable JSON;
- backup format version is supported;
- app version is compatible;
- backup schema version matches the database schema;
- required core tables are present;
- SQLite `PRAGMA quick_check` passes on a temporary copy of the backup database.

Validate a backup before attempting restore, especially after copying it across disks or devices.

## Restore a backup

Restore is destructive because it replaces the current local app database.

The desktop UI allows restore only after the selected folder validates. The restore handler validates again, then asks for explicit confirmation before passing `confirm_destructive_restore: true` to the Tauri command.

Before restoring:

- create a fresh backup of the current database if you may need to roll back;
- confirm the selected folder is the intended backup;
- close other tools that may be inspecting the app data database.

During restore, the active `CrmCore` instance is closed, the validated backup database is copied into the app data directory, stale SQLite sidecars are removed, and `CrmCore` is reopened.

## Developer API surface

Frontend wrapper:

- `apps/desktop/src/lib/api/backup.ts`

Tauri commands:

- `create_local_backup`
- `validate_local_backup`
- `restore_local_backup_to_app_data`

Core service:

- `crates/crm-core/src/services/backup.rs`

The backup surface is intentionally local-only. It does not add MCP behavior, AI behavior, remote sync behavior, or database migrations.
