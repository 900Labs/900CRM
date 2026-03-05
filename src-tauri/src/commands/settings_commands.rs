//! Tauri IPC commands for application settings management.
//!
//! # Commands
//!
//! | Command           | Description |
//! |-------------------|-------------|
//! | `get_settings`    | Returns all settings as a key-value map |
//! | `get_setting`     | Returns a single setting value by key |
//! | `update_setting`  | Sets a setting key to a new value |

use std::collections::HashMap;

use tauri::State;

use crate::storage::settings::{self, Setting};
use crate::storage::sync;
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// get_settings
// ─────────────────────────────────────────────────────────────────────────────

/// Returns all application settings as a flat key-value map.
///
/// # Returns
///
/// A `HashMap<String, String>` where keys are setting names and values are
/// their current string values.
///
/// # Errors
///
/// Returns a `String` error on database failure.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_settings");

    let all = settings::get_all_settings(&db.conn).map_err(|e| e.to_string())?;
    let map: HashMap<String, String> = all.into_iter().map(|s| (s.key, s.value)).collect();
    Ok(map)
}

// ─────────────────────────────────────────────────────────────────────────────
// get_setting
// ─────────────────────────────────────────────────────────────────────────────

/// Returns a single setting by key.
///
/// # Parameters
///
/// - `key` — Setting key (e.g. `"theme"`, `"language"`).
///
/// # Returns
///
/// The [`Setting`] struct, or `null` (serialized as `None`) if the key
/// does not exist.
///
/// # Errors
///
/// Returns a `String` error on database failure.
#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<Setting>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_setting key={}", key);
    settings::get_setting(&db.conn, &key).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// update_setting
// ─────────────────────────────────────────────────────────────────────────────

/// Sets a setting to a new value.
///
/// If the key already exists, the value is updated. If the key is new, it
/// is inserted. Returns the updated [`Setting`].
///
/// # Parameters
///
/// - `key` — Setting key.
/// - `value` — New setting value.
///
/// # Errors
///
/// Returns a `String` error on database failure.
#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<Setting, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();
    log::info!("Command: update_setting key={} value={}", key, value);
    let setting = settings::set_setting(&db.conn, &key, &value).map_err(|e| e.to_string())?;
    sync::record_change(&db.conn, "setting", &key, &key, None, Some(&value), &device_id)
        .map_err(|e| e.to_string())?;
    Ok(setting)
}
