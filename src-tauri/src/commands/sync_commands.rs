//! Tauri IPC commands for lightweight sync status.
//!
//! These commands expose local sync readiness and changelog state. They do not
//! perform network replication yet.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::storage::{settings, sync};
use crate::utils::datetime::now_iso8601;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: String,
    pub last_sync_at: Option<String>,
    pub error_message: Option<String>,
    pub pending_changes: u32,
}

fn parse_bool(value: Option<&str>) -> bool {
    matches!(value, Some("true") | Some("1"))
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let pending_changes = sync::get_all_pending_changes(&db.conn)
        .map_err(|e| e.to_string())?
        .len() as u32;

    let last_sync_at = sync::get_latest_change_timestamp(&db.conn).map_err(|e| e.to_string())?;

    let sync_enabled = settings::get_setting(&db.conn, "sync_enabled")
        .map_err(|e| e.to_string())?
        .map(|s| s.value)
        .unwrap_or_else(|| "false".to_string());

    let sync_url = settings::get_setting(&db.conn, "sync_url")
        .map_err(|e| e.to_string())?
        .map(|s| s.value)
        .unwrap_or_default();

    if !parse_bool(Some(sync_enabled.as_str())) {
        return Ok(SyncStatus {
            state: "idle".to_string(),
            last_sync_at,
            error_message: None,
            pending_changes,
        });
    }

    if sync_url.trim().is_empty() {
        return Ok(SyncStatus {
            state: "error".to_string(),
            last_sync_at,
            error_message: Some("Sync URL is not configured.".to_string()),
            pending_changes,
        });
    }

    Ok(SyncStatus {
        state: "idle".to_string(),
        last_sync_at,
        error_message: None,
        pending_changes,
    })
}

#[tauri::command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let pending_changes = sync::get_all_pending_changes(&db.conn)
        .map_err(|e| e.to_string())?
        .len() as u32;

    let sync_enabled = settings::get_setting(&db.conn, "sync_enabled")
        .map_err(|e| e.to_string())?
        .map(|s| s.value)
        .unwrap_or_else(|| "false".to_string());

    let sync_url = settings::get_setting(&db.conn, "sync_url")
        .map_err(|e| e.to_string())?
        .map(|s| s.value)
        .unwrap_or_default();

    if !parse_bool(Some(sync_enabled.as_str())) {
        return Ok(SyncStatus {
            state: "idle".to_string(),
            last_sync_at: None,
            error_message: None,
            pending_changes,
        });
    }

    if sync_url.trim().is_empty() {
        return Ok(SyncStatus {
            state: "error".to_string(),
            last_sync_at: None,
            error_message: Some("Sync URL is not configured.".to_string()),
            pending_changes,
        });
    }

    // Placeholder for full replication engine: report successful local trigger.
    Ok(SyncStatus {
        state: "success".to_string(),
        last_sync_at: Some(now_iso8601()),
        error_message: None,
        pending_changes,
    })
}
