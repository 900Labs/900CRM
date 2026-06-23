use crm_core::storage::activities::Activity;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn create_activity(
    state: State<'_, AppState>,
    activity_type: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    contact_id: Option<String>,
    deal_id: Option<String>,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.create_activity(
        activity_type,
        title,
        description,
        due_date,
        contact_id,
        deal_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity(state: State<'_, AppState>, id: String) -> Result<Activity, String> {
    let core = super::lock_core(&state)?;
    core.get_activity(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_activities(state: State<'_, AppState>) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_activities_for_contact(
    state: State<'_, AppState>,
    contact_id: String,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities_for_contact(&contact_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_activities_for_deal(
    state: State<'_, AppState>,
    deal_id: String,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities_for_deal(&deal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_upcoming_activities(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_upcoming_activities(limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_activity_complete(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.mark_activity_complete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_activity_incomplete(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.mark_activity_incomplete(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_activity(
    state: State<'_, AppState>,
    id: String,
    activity_type: Option<String>,
    title: Option<String>,
    description: Option<String>,
    due_date: Option<String>,
    completed: Option<bool>,
    contact_id: Option<String>,
    deal_id: Option<String>,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.update_activity(
        &id,
        activity_type,
        title,
        description,
        due_date,
        completed,
        contact_id,
        deal_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_activity(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = super::lock_core(&state)?;
    core.delete_activity(&id).map_err(|e| e.to_string())
}
