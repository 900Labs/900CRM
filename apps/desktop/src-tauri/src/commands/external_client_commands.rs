use crm_core::storage::external_clients::ExternalClient;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command]
pub async fn list_external_clients(
    state: State<'_, AppState>,
) -> Result<Vec<ExternalClient>, String> {
    let core = lock_core(&state)?;
    core.list_external_clients().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_external_client_placeholder(
    state: State<'_, AppState>,
    name: String,
    client_type: String,
) -> Result<ExternalClient, String> {
    let mut core = lock_core(&state)?;
    core.create_external_client_placeholder(&name, &client_type)
        .map_err(|e| e.to_string())
}
