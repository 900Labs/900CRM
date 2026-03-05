//! 900CRM — library crate.
//!
//! This crate contains all application logic, separated from the thin binary
//! entry point in `main.rs`. The split allows unit-testing of command handlers
//! and storage logic without spinning up a full Tauri application.
//!
//! # Module Hierarchy
//!
//! ```text
//! ninehundredcrm_lib
//! ├── crm_engine/           — Business logic (validation, duplicate detection, pipeline)
//! │   ├── contacts.rs       — Contact validation, duplicate detection, merge
//! │   ├── deals.rs          — Stage definitions, win rate, pipeline metrics
//! │   ├── activities.rs     — Scheduling, overdue detection, stats
//! │   ├── pipeline.rs       — Pipeline configuration and full metrics
//! │   └── search.rs         — Unified cross-entity search
//! ├── commands/             — #[tauri::command] IPC boundary handlers
//! │   ├── contact_commands.rs
//! │   ├── deal_commands.rs
//! │   ├── activity_commands.rs
//! │   ├── dashboard_commands.rs
//! │   ├── custom_field_commands.rs
//! │   ├── report_commands.rs
//! │   ├── import_export.rs
//! │   └── settings_commands.rs
//! ├── storage/              — SQLite persistence (rusqlite)
//! │   ├── db.rs             — Database struct, connection setup, migrations
//! │   ├── contacts.rs       — Contact CRUD + FTS5 search
//! │   ├── deals.rs          — Deal CRUD + pipeline queries
//! │   ├── activities.rs     — Activity CRUD + scheduling
//! │   ├── custom_fields.rs  — Custom field definitions and values
//! │   ├── reporting.rs      — Pipeline conversion and activity funnel aggregates
//! │   ├── notes.rs          — Note CRUD
//! │   ├── tags.rs           — Tag CRUD + entity tagging
//! │   ├── settings.rs       — Key-value settings store
//! │   └── sync.rs           — Offline-first sync changelog
//! └── utils/                — Cross-cutting helpers
//!     ├── errors.rs         — CrmError enum, CrmResult type alias
//!     ├── uuid.rs           — UUID v4 generation
//!     ├── csv.rs            — CSV import/export helpers
//!     └── datetime.rs       — ISO 8601 timestamp utilities
//! ```
//!
//! # AppState
//!
//! The central state structure managed by Tauri and injected into every
//! command handler is defined here as [`AppState`]. It is constructed in
//! [`run`] and passed to `tauri::Builder::manage`.
//!
//! # Thread Safety
//!
//! All shared mutable state is wrapped in `std::sync::Mutex`:
//!
//! - `engine` — The [`crm_engine::CrmEngine`]; mutex serializes access.
//! - `db` — SQLite `rusqlite::Connection` is `!Send`; mutex ensures
//!   single-threaded use.
//!
//! # Locking Order
//!
//! To avoid deadlocks, **always** acquire locks in this order when multiple
//! locks are needed in the same code path:
//!
//! 1. `engine`
//! 2. `db`
//!
//! In practice most command handlers need only one lock and should release
//! it as soon as the operation completes.

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::Manager;

pub mod commands;
pub mod crm_engine;
pub mod storage;
pub mod utils;

// ─────────────────────────────────────────────────────────────────────────────
// AppState
// ─────────────────────────────────────────────────────────────────────────────

/// Shared application state managed by Tauri and injected into every command.
///
/// Use `state: tauri::State<'_, AppState>` as a parameter in any
/// `#[tauri::command]` to gain access to the database, engine, device ID,
/// and data directory.
///
/// All mutable fields are wrapped in [`Mutex`] to allow shared access from
/// the multi-threaded Tokio runtime Tauri uses for command dispatch.
///
/// # Locking Order
///
/// Always acquire locks in this order to prevent deadlocks:
///
/// 1. `engine` (if needed)
/// 2. `db`
///
/// # Example
///
/// ```rust,ignore
/// #[tauri::command]
/// async fn my_command(state: State<'_, AppState>) -> Result<String, String> {
///     let db = state.db.lock().map_err(|e| e.to_string())?;
///     // … use db.conn …
///     Ok("done".into())
/// }
/// ```
pub struct AppState {
    /// The CRM business logic engine.
    ///
    /// Locked when pipeline logic, validation, or metrics calculations
    /// require access to stage definitions or engine configuration.
    pub engine: Mutex<crm_engine::CrmEngine>,

    /// SQLite database connection.
    ///
    /// Locked for the duration of any database read or write operation.
    /// All storage operations use `db.conn` directly.
    pub db: Mutex<storage::Database>,

    /// UUID v4 string identifying this device.
    ///
    /// Generated once at startup and stored in the `settings` table under
    /// the key `"device_id"`. Used in the sync changelog to attribute
    /// changes to their origin.
    pub device_id: String,

    /// Absolute path to the application data directory.
    ///
    /// This is the directory returned by Tauri's `app_data_dir()` API,
    /// e.g. `~/.local/share/com.900labs.crm/`. The SQLite database file
    /// (`900crm.db`) lives here.
    pub data_dir: PathBuf,
}

// ─────────────────────────────────────────────────────────────────────────────
// run() — application entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Initializes all application components and starts the Tauri event loop.
///
/// Called from `main.rs`. Performs the following steps:
///
/// 1. Initializes `env_logger` for structured log output.
/// 2. Creates the Tauri application builder.
/// 3. In the `setup` hook:
///    a. Determines the app data directory.
///    b. Opens the SQLite database and runs migrations.
///    c. Resolves or generates the `device_id`.
///    d. Initializes the [`crm_engine::CrmEngine`].
///    e. Constructs and registers [`AppState`].
/// 4. Registers all `#[tauri::command]` handlers.
/// 5. Runs the Tauri event loop (blocks until the window closes).
///
/// # Panics
///
/// This function panics (with an informative message) if:
/// - The SQLite database cannot be opened or migrated.
/// - The app data directory cannot be determined.
///
/// These are fatal startup errors that cannot be recovered from.
pub fn run() {
    // ── Logging ───────────────────────────────────────────────────────────────
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(default_log_level),
    )
    .init();

    log::info!(
        "900CRM v{} starting ({})",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );

    // ── Tauri application ─────────────────────────────────────────────────────
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        // ── Setup hook ────────────────────────────────────────────────────────
        .setup(|app| {
            // Determine the application data directory.
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory — Tauri path API unavailable");

            log::info!("App data directory: {}", app_data_dir.display());

            // Open/create the SQLite database and run all migrations.
            let db = storage::Database::new(&app_data_dir)
                .expect("Failed to initialize SQLite database");

            log::info!("SQLite database initialized at {}/900crm.db", app_data_dir.display());

            // Resolve or generate the device_id.
            // On first launch a new UUID is created and persisted in settings.
            let device_id = {
                match storage::settings::get_setting(&db.conn, "device_id")
                    .ok()
                    .flatten()
                {
                    Some(s) if !s.value.is_empty() => {
                        log::info!("Loaded device_id from settings: {}", s.value);
                        s.value
                    }
                    _ => {
                        let new_id = utils::uuid::new_uuid();
                        let _ = storage::settings::set_setting(&db.conn, "device_id", &new_id);
                        log::info!("Generated new device_id: {}", new_id);
                        new_id
                    }
                }
            };

            // Initialize the CRM engine with default pipeline stages.
            let engine = crm_engine::CrmEngine::new();
            log::info!(
                "CrmEngine initialized with {} pipeline stages",
                engine.default_stages.len()
            );

            // Build AppState and register it with Tauri.
            let state = AppState {
                engine: Mutex::new(engine),
                db: Mutex::new(db),
                device_id,
                data_dir: app_data_dir,
            };

            app.manage(state);
            log::info!("AppState registered with Tauri");

            Ok(())
        })
        // ── Register ALL command handlers ─────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // ── Contact commands ──────────────────────────────────────────────
            commands::contact_commands::create_contact,
            commands::contact_commands::get_contact,
            commands::contact_commands::list_contacts,
            commands::contact_commands::update_contact,
            commands::contact_commands::delete_contact,
            commands::contact_commands::restore_contact,
            commands::contact_commands::search_contacts,
            commands::contact_commands::merge_contacts,
            // ── Deal commands ─────────────────────────────────────────────────
            commands::deal_commands::create_deal,
            commands::deal_commands::get_deal,
            commands::deal_commands::list_deals,
            commands::deal_commands::list_deals_by_stage,
            commands::deal_commands::update_deal,
            commands::deal_commands::move_deal_stage,
            commands::deal_commands::delete_deal,
            commands::deal_commands::get_pipeline_summary,
            // ── Activity commands ─────────────────────────────────────────────
            commands::activity_commands::create_activity,
            commands::activity_commands::get_activity,
            commands::activity_commands::list_activities,
            commands::activity_commands::list_activities_for_contact,
            commands::activity_commands::list_activities_for_deal,
            commands::activity_commands::list_upcoming_activities,
            commands::activity_commands::mark_activity_complete,
            commands::activity_commands::mark_activity_incomplete,
            commands::activity_commands::update_activity,
            commands::activity_commands::delete_activity,
            // ── Dashboard commands ────────────────────────────────────────────
            commands::dashboard_commands::get_dashboard_stats,
            // ── Custom field commands ────────────────────────────────────────
            commands::custom_field_commands::list_custom_field_defs,
            commands::custom_field_commands::create_custom_field_def,
            commands::custom_field_commands::update_custom_field_def,
            commands::custom_field_commands::delete_custom_field_def,
            commands::custom_field_commands::set_custom_field_value,
            commands::custom_field_commands::list_custom_field_values,
            // ── Reporting commands ───────────────────────────────────────────
            commands::report_commands::get_pipeline_conversion_report,
            commands::report_commands::get_activity_funnel_report,
            // ── Import / Export commands ──────────────────────────────────────
            commands::import_export::import_contacts_csv,
            commands::import_export::export_contacts_csv,
            commands::import_export::import_deals_csv,
            commands::import_export::export_deals_csv,
            // ── Settings commands ─────────────────────────────────────────────
            commands::settings_commands::get_settings,
            commands::settings_commands::get_setting,
            commands::settings_commands::update_setting,
            // ── Sync status commands ────────────────────────────────────────
            commands::sync_commands::get_sync_status,
            commands::sync_commands::trigger_sync,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running 900CRM application");
}
