use crm_core::services::ImportResult;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn import_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_deals_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_deals_csv(&file_path).map_err(|e| e.to_string())
}
