use crm_core::search::SearchResult;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command]
pub async fn global_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let core = lock_core(&state)?;
    core.global_search(&query, limit).map_err(|e| e.to_string())
}
