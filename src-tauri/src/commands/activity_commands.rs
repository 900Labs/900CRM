//! Tauri IPC commands for activity management.
//!
//! # Commands
//!
//! | Command                      | Description |
//! |------------------------------|-------------|
//! | `create_activity`            | Creates a new activity |
//! | `get_activity`               | Retrieves an activity by ID |
//! | `list_activities`            | Lists all active activities |
//! | `list_activities_for_contact`| Lists activities for a contact |
//! | `list_activities_for_deal`   | Lists activities for a deal |
//! | `list_upcoming_activities`   | Lists upcoming (future) activities |
//! | `mark_activity_complete`     | Marks an activity as done |
//! | `update_activity`            | Updates activity fields |
//! | `delete_activity`            | Soft-deletes an activity |

use tauri::State;

use crate::crm_engine::activities::validate_activity_for_create;
use crate::storage::activities::{self, Activity};
use crate::storage::sync;
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// create_activity
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new activity after validation.
///
/// # Errors
///
/// Returns a `String` error on validation or database failure.
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
    validate_activity_for_create(&title, &activity_type).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let activity = activities::create_activity(
        &db.conn,
        &activity_type,
        &title,
        description.as_deref().unwrap_or(""),
        due_date.as_deref(),
        contact_id.as_deref(),
        deal_id.as_deref(),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "activity",
        &activity.id,
        "__create__",
        None,
        Some(&activity.id),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    log::info!("Command: create_activity id={} type={}", activity.id, activity_type);
    Ok(activity)
}

// ─────────────────────────────────────────────────────────────────────────────
// get_activity
// ─────────────────────────────────────────────────────────────────────────────

/// Retrieves a single activity by UUID.
#[tauri::command]
pub async fn get_activity(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_activity id={}", id);
    activities::get_activity(&db.conn, &id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_activities
// ─────────────────────────────────────────────────────────────────────────────

/// Lists all active activities, ordered by due date.
#[tauri::command]
pub async fn list_activities(state: State<'_, AppState>) -> Result<Vec<Activity>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: list_activities");
    activities::list_activities(&db.conn).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_activities_for_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Lists all activities associated with a specific contact.
#[tauri::command]
pub async fn list_activities_for_contact(
    state: State<'_, AppState>,
    contact_id: String,
) -> Result<Vec<Activity>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    activities::list_activities_for_contact(&db.conn, &contact_id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_activities_for_deal
// ─────────────────────────────────────────────────────────────────────────────

/// Lists all activities associated with a specific deal.
#[tauri::command]
pub async fn list_activities_for_deal(
    state: State<'_, AppState>,
    deal_id: String,
) -> Result<Vec<Activity>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    activities::list_activities_for_deal(&db.conn, &deal_id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_upcoming_activities
// ─────────────────────────────────────────────────────────────────────────────

/// Returns upcoming (incomplete, future-due) activities.
///
/// # Parameters
///
/// - `limit` — Maximum number of activities to return (default: 10).
#[tauri::command]
pub async fn list_upcoming_activities(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<Activity>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let limit = limit.unwrap_or(10);
    log::debug!("Command: list_upcoming_activities limit={}", limit);
    activities::list_upcoming_activities(&db.conn, limit).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// mark_activity_complete
// ─────────────────────────────────────────────────────────────────────────────

/// Marks an activity as completed.
#[tauri::command]
pub async fn mark_activity_complete(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let activity = activities::mark_complete(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "activity",
        &id,
        "completed",
        Some("0"),
        Some("1"),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    log::info!("Command: mark_activity_complete id={}", id);
    Ok(activity)
}

// ─────────────────────────────────────────────────────────────────────────────
// update_activity
// ─────────────────────────────────────────────────────────────────────────────

/// Updates an activity's fields.
///
/// Fields not provided (null in JSON) retain their current values.
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
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let activity = activities::update_activity(
        &db.conn,
        &id,
        activity_type.as_deref(),
        title.as_deref(),
        description.as_deref(),
        Some(due_date.as_deref()),
        completed,
        Some(contact_id.as_deref()),
        Some(deal_id.as_deref()),
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "activity", &id, "__update__", None, Some(&id), &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: update_activity id={}", id);
    Ok(activity)
}

// ─────────────────────────────────────────────────────────────────────────────
// delete_activity
// ─────────────────────────────────────────────────────────────────────────────

/// Soft-deletes an activity.
#[tauri::command]
pub async fn delete_activity(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    activities::soft_delete_activity(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "activity", &id, "__delete__", Some(&id), None, &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: delete_activity id={}", id);
    Ok(())
}
