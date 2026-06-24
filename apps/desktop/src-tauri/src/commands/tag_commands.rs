use crm_core::storage::tags::Tag;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let mut core = lock_core(&state)?;
    core.create_tag(name, color).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tag(state: State<'_, AppState>, id: String) -> Result<Tag, String> {
    let core = lock_core(&state)?;
    core.get_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let core = lock_core(&state)?;
    core.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    color: Option<String>,
) -> Result<Tag, String> {
    let mut core = lock_core(&state)?;
    core.update_tag(&id, name, color).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_tag_to_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    tag_id: String,
) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.apply_tag_to_entity(entity_type, entity_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag_from_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    tag_id: String,
) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.remove_tag_from_entity(entity_type, entity_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_tags_for_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<Tag>, String> {
    let core = lock_core(&state)?;
    core.list_tags_for_entity(entity_type, entity_id)
        .map_err(|e| e.to_string())
}
