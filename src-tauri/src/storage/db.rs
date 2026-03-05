//! SQLite database initialization and schema management for 900CRM.
//!
//! This module wraps a `rusqlite` connection and provides the [`Database`]
//! struct stored in [`crate::AppState`]. All other storage modules operate on
//! the `pub(crate) conn` field directly for maximum composability.
//!
//! # Schema Overview
//!
//! The complete CRM schema is created by [`Database::run_migrations`]:
//!
//! | Table                | Purpose |
//! |----------------------|---------|
//! | `contacts`           | People and organizations |
//! | `deals`              | Sales opportunities |
//! | `activities`         | Tasks, calls, meetings, emails |
//! | `notes`              | Free-text notes attached to any entity |
//! | `tags`               | Taxonomy labels |
//! | `entity_tags`        | Many-to-many entity ↔ tag join |
//! | `custom_field_defs`  | User-defined field definitions |
//! | `custom_field_values`| User-defined field values per entity |
//! | `settings`           | Key-value application preferences |
//! | `sync_changelog`     | Offline-first mutation log for sync |
//! | `contacts_fts`       | FTS5 virtual table for full-text search |
//!
//! # Migration Strategy
//!
//! [`Database::run_migrations`] is idempotent: `CREATE TABLE IF NOT EXISTS` and
//! `CREATE INDEX IF NOT EXISTS` are safe to call on every startup. Incremental
//! upgrades compare `PRAGMA user_version` to [`CURRENT_SCHEMA_VERSION`].
//!
//! # Thread Safety
//!
//! `rusqlite::Connection` is `!Send`. The `Database` struct is wrapped in a
//! `Mutex<Database>` inside `AppState`, ensuring single-threaded access.
//! WAL mode is enabled so readers never block writers.

use std::path::Path;

use rusqlite::Connection;

use crate::utils::errors::{CrmError, CrmResult};

/// The current schema version. Increment whenever a new migration is added.
const CURRENT_SCHEMA_VERSION: u32 = 2;

// ─────────────────────────────────────────────────────────────────────────────
// Database struct
// ─────────────────────────────────────────────────────────────────────────────

/// A wrapper around a `rusqlite::Connection` with a fully initialized CRM schema.
///
/// Obtain an instance via [`Database::new`]. All storage sub-modules access the
/// underlying connection through `db.conn`.
///
/// # Example
///
/// ```rust,ignore
/// use std::path::Path;
/// use crate::storage::Database;
///
/// let db = Database::new(Path::new("/tmp/crm-test")).unwrap();
/// ```
pub struct Database {
    /// The underlying SQLite connection.
    ///
    /// Exposed as `pub(crate)` so sibling storage modules can use `prepare`,
    /// `execute`, `query_row`, etc. directly.
    pub(crate) conn: Connection,
}

impl Database {
    /// Opens (or creates) the SQLite database at `<app_data_dir>/900crm.db`
    /// and runs all pending migrations.
    ///
    /// # Parameters
    ///
    /// - `app_data_dir` — Directory returned by Tauri's `app_data_dir()` API.
    ///   The database file is placed at `<app_data_dir>/900crm.db`.
    ///
    /// # Errors
    ///
    /// - [`CrmError::Io`] — `app_data_dir` could not be created.
    /// - [`CrmError::Database`] — Connection could not be opened or migrations failed.
    pub fn new(app_data_dir: &Path) -> CrmResult<Self> {
        // Ensure the application data directory exists.
        std::fs::create_dir_all(app_data_dir).map_err(|e| {
            CrmError::Io(format!(
                "Failed to create app data directory '{}': {}",
                app_data_dir.display(),
                e
            ))
        })?;

        let db_path = app_data_dir.join("900crm.db");
        log::info!("Opening SQLite database at {}", db_path.display());

        let conn = Connection::open(&db_path).map_err(|e| {
            CrmError::Database(format!(
                "Failed to open database at '{}': {}",
                db_path.display(),
                e
            ))
        })?;

        let mut db = Self { conn };
        db.configure()?;
        db.run_migrations()?;

        log::info!("Database initialized successfully");
        Ok(db)
    }

    // ─── Configuration ────────────────────────────────────────────────────────

    /// Applies performance and reliability PRAGMAs to the connection.
    ///
    /// Called once immediately after opening, before any user code runs.
    ///
    /// PRAGMAs applied:
    /// - `journal_mode = WAL` — Readers don't block writers.
    /// - `synchronous = NORMAL` — Good balance between safety and performance.
    /// - `foreign_keys = ON` — Enforce referential integrity.
    /// - `cache_size = -8192` — 8 MB page cache.
    /// - `journal_size_limit = 67108864` — Cap WAL file at 64 MB.
    fn configure(&mut self) -> CrmResult<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode      = WAL;
            PRAGMA synchronous       = NORMAL;
            PRAGMA foreign_keys      = ON;
            PRAGMA cache_size        = -8192;
            PRAGMA journal_size_limit = 67108864;
            PRAGMA temp_store        = MEMORY;
            "#,
        )?;
        log::debug!("SQLite PRAGMAs applied");
        Ok(())
    }

    // ─── Migrations ───────────────────────────────────────────────────────────

    /// Creates all application tables, indices, and virtual tables if they do
    /// not already exist, then applies incremental schema upgrades.
    ///
    /// This function is idempotent: calling it multiple times is safe.
    ///
    /// # Errors
    ///
    /// Returns [`CrmError::Database`] if any DDL statement fails.
    pub fn run_migrations(&mut self) -> CrmResult<()> {
        let current_version: u32 = self
            .conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap_or(0);

        log::debug!(
            "Database schema: current={}, target={}",
            current_version,
            CURRENT_SCHEMA_VERSION
        );

        if current_version < 1 {
            self.migrate_v1()?;
        }

        if current_version < 2 {
            self.migrate_v2()?;
        }

        self.conn.execute_batch(&format!(
            "PRAGMA user_version = {};",
            CURRENT_SCHEMA_VERSION
        ))?;

        log::info!("Migrations complete (schema v{})", CURRENT_SCHEMA_VERSION);
        Ok(())
    }

    /// Initial schema migration — creates all tables for schema version 1.
    fn migrate_v1(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v1");

        self.conn.execute_batch(
            r#"
            -- ──────────────────────────────────────────────────────────────────
            -- contacts
            -- Stores individual people and organizations.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS contacts (
                id           TEXT PRIMARY KEY NOT NULL,
                contact_type TEXT NOT NULL DEFAULT 'person',
                first_name   TEXT NOT NULL DEFAULT '',
                last_name    TEXT NOT NULL DEFAULT '',
                org_name     TEXT NOT NULL DEFAULT '',
                email        TEXT NOT NULL DEFAULT '',
                phone        TEXT NOT NULL DEFAULT '',
                address      TEXT NOT NULL DEFAULT '',
                city         TEXT NOT NULL DEFAULT '',
                country      TEXT NOT NULL DEFAULT '',
                org_id       TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                notes        TEXT NOT NULL DEFAULT '',
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                deleted_at   TEXT,
                device_id    TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_contacts_email
                ON contacts (email);
            CREATE INDEX IF NOT EXISTS idx_contacts_org_id
                ON contacts (org_id);
            CREATE INDEX IF NOT EXISTS idx_contacts_deleted_at
                ON contacts (deleted_at);
            CREATE INDEX IF NOT EXISTS idx_contacts_contact_type
                ON contacts (contact_type);

            -- ──────────────────────────────────────────────────────────────────
            -- deals
            -- Sales opportunities moving through the pipeline.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS deals (
                id             TEXT PRIMARY KEY NOT NULL,
                title          TEXT NOT NULL DEFAULT '',
                value          REAL NOT NULL DEFAULT 0.0,
                currency       TEXT NOT NULL DEFAULT 'USD',
                stage          TEXT NOT NULL DEFAULT 'Lead',
                probability    INTEGER NOT NULL DEFAULT 0,
                expected_close TEXT,
                contact_id     TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                notes          TEXT NOT NULL DEFAULT '',
                created_at     TEXT NOT NULL,
                updated_at     TEXT NOT NULL,
                deleted_at     TEXT,
                device_id      TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_deals_stage
                ON deals (stage);
            CREATE INDEX IF NOT EXISTS idx_deals_contact_id
                ON deals (contact_id);
            CREATE INDEX IF NOT EXISTS idx_deals_deleted_at
                ON deals (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- activities
            -- Tasks, calls, meetings, emails attached to contacts or deals.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                deal_id       TEXT REFERENCES deals(id) ON DELETE SET NULL,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_activities_contact_id
                ON activities (contact_id);
            CREATE INDEX IF NOT EXISTS idx_activities_deal_id
                ON activities (deal_id);
            CREATE INDEX IF NOT EXISTS idx_activities_due_date
                ON activities (due_date);
            CREATE INDEX IF NOT EXISTS idx_activities_completed
                ON activities (completed);
            CREATE INDEX IF NOT EXISTS idx_activities_deleted_at
                ON activities (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- notes
            -- Freeform text notes attached to any entity.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS notes (
                id          TEXT PRIMARY KEY NOT NULL,
                content     TEXT NOT NULL DEFAULT '',
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_notes_entity
                ON notes (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_notes_deleted_at
                ON notes (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- tags
            -- Taxonomy labels that can be applied to any entity.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS tags (
                id         TEXT PRIMARY KEY NOT NULL,
                name       TEXT NOT NULL UNIQUE,
                color      TEXT NOT NULL DEFAULT '#6366f1',
                created_at TEXT NOT NULL
            );

            -- ──────────────────────────────────────────────────────────────────
            -- entity_tags
            -- Many-to-many join: entity ↔ tag.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS entity_tags (
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                tag_id      TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (entity_type, entity_id, tag_id)
            );

            CREATE INDEX IF NOT EXISTS idx_entity_tags_entity
                ON entity_tags (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_entity_tags_tag_id
                ON entity_tags (tag_id);

            -- ──────────────────────────────────────────────────────────────────
            -- custom_field_defs
            -- User-defined extra fields per entity type.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS custom_field_defs (
                id            TEXT PRIMARY KEY NOT NULL,
                entity_type   TEXT NOT NULL,
                field_name    TEXT NOT NULL,
                field_type    TEXT NOT NULL DEFAULT 'text',
                field_options TEXT,
                sort_order    INTEGER NOT NULL DEFAULT 0,
                created_at    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_custom_field_defs_entity_type
                ON custom_field_defs (entity_type);

            -- ──────────────────────────────────────────────────────────────────
            -- custom_field_values
            -- Values for user-defined fields, per entity instance.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS custom_field_values (
                id           TEXT PRIMARY KEY NOT NULL,
                field_def_id TEXT NOT NULL REFERENCES custom_field_defs(id) ON DELETE CASCADE,
                entity_id    TEXT NOT NULL,
                value        TEXT NOT NULL DEFAULT '',
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_custom_field_values_entity
                ON custom_field_values (entity_id);
            CREATE INDEX IF NOT EXISTS idx_custom_field_values_field_def
                ON custom_field_values (field_def_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_custom_field_values_field_entity
                ON custom_field_values (field_def_id, entity_id);

            -- ──────────────────────────────────────────────────────────────────
            -- settings
            -- Application-wide key-value preferences store.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT ''
            );

            -- Seed default settings. INSERT OR IGNORE preserves existing values.
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('language',     'en',         '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('currency',     'USD',        '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('theme',        'light',      '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('date_format',  'YYYY-MM-DD', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('sync_enabled', 'false',      '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('sync_url',     '',           '');

            -- ──────────────────────────────────────────────────────────────────
            -- sync_changelog
            -- Append-only log of every mutation for offline-first sync.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS sync_changelog (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                field_name  TEXT NOT NULL,
                old_value   TEXT,
                new_value   TEXT,
                timestamp   TEXT NOT NULL,
                device_id   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sync_changelog_timestamp
                ON sync_changelog (timestamp);
            CREATE INDEX IF NOT EXISTS idx_sync_changelog_entity
                ON sync_changelog (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_sync_changelog_device_id
                ON sync_changelog (device_id);

            -- ──────────────────────────────────────────────────────────────────
            -- contacts_fts
            -- FTS5 virtual table for full-text search over contacts.
            -- ──────────────────────────────────────────────────────────────────
            CREATE VIRTUAL TABLE IF NOT EXISTS contacts_fts USING fts5(
                first_name,
                last_name,
                org_name,
                email,
                phone,
                content='contacts',
                content_rowid='rowid'
            );
            "#,
        )?;

        log::info!("Migration v1 complete");
        Ok(())
    }

    /// Schema v2 migration — reporting/perf indexes for analytics endpoints.
    ///
    /// Adds composite and time-oriented indexes used by reporting queries to
    /// keep aggregation paths responsive on low-resource hardware.
    fn migrate_v2(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v2");

        self.conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_deals_created_at
                ON deals (created_at);
            CREATE INDEX IF NOT EXISTS idx_deals_deleted_stage
                ON deals (deleted_at, stage);

            CREATE INDEX IF NOT EXISTS idx_activities_created_at
                ON activities (created_at);
            CREATE INDEX IF NOT EXISTS idx_activities_type
                ON activities (activity_type);
            CREATE INDEX IF NOT EXISTS idx_activities_completed_due
                ON activities (completed, due_date);
            "#,
        )?;

        log::info!("Migration v2 complete");
        Ok(())
    }

    /// Returns a reference to the underlying `rusqlite::Connection`.
    ///
    /// Prefer using the typed storage functions in the sibling modules. This
    /// accessor is provided for advanced queries not covered by the storage API.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}
