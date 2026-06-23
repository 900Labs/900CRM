use std::path::Path;

use crm_core::{
    services::{LocalBackup, LocalBackupValidation, LocalRestoreResult},
    CrmCore,
};
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn create_local_backup(
    state: State<'_, AppState>,
    backup_dir: String,
) -> Result<LocalBackup, String> {
    let core = super::lock_core(&state)?;
    core.create_local_backup(Path::new(&backup_dir))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_local_backup(
    state: State<'_, AppState>,
    backup_dir: String,
) -> Result<LocalBackupValidation, String> {
    let core = super::lock_core(&state)?;
    core.validate_local_backup(Path::new(&backup_dir))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_local_backup_to_app_data(
    state: State<'_, AppState>,
    backup_dir: String,
    confirm_destructive_restore: bool,
) -> Result<LocalRestoreResult, String> {
    let app_data_dir = state.data_dir.clone();
    close_active_core(&state)?;

    let restore_result = CrmCore::restore_local_backup_to_app_data(
        &app_data_dir,
        Path::new(&backup_dir),
        confirm_destructive_restore,
    )
    .map_err(|e| e.to_string());

    let reopen_result = reopen_core(&state, &app_data_dir);
    match (restore_result, reopen_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(restore_error), Ok(())) => Err(restore_error),
        (Ok(_), Err(reopen_error)) => Err(format!(
            "Local restore completed but CrmCore failed to reopen: {}",
            reopen_error
        )),
        (Err(restore_error), Err(reopen_error)) => Err(format!(
            "Local restore failed: {}; CrmCore also failed to reopen: {}",
            restore_error, reopen_error
        )),
    }
}

fn close_active_core(state: &State<'_, AppState>) -> Result<(), String> {
    let mut core_slot = state
        .core
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let active_core = core_slot
        .take()
        .ok_or_else(|| "CrmCore is unavailable during local restore".to_string())?;
    drop(active_core);
    Ok(())
}

fn reopen_core(state: &State<'_, AppState>, app_data_dir: &Path) -> Result<(), String> {
    let reopened_core = CrmCore::open(app_data_dir).map_err(|e| e.to_string())?;
    let mut core_slot = state
        .core
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    *core_slot = Some(reopened_core);
    Ok(())
}
