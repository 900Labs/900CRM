use crm_core::services::SyncStatus;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let core = super::lock_core(&state)?;
    core.get_sync_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let core = super::lock_core(&state)?;
    core.trigger_sync().map_err(|e| e.to_string())
}
