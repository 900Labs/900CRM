//! Tauri IPC commands for deal management.
//!
//! # Commands
//!
//! | Command              | Description |
//! |----------------------|-------------|
//! | `create_deal`        | Creates a new deal |
//! | `get_deal`           | Retrieves a deal by ID |
//! | `list_deals`         | Lists all active deals |
//! | `list_deals_by_stage`| Lists deals in a specific stage |
//! | `update_deal`        | Updates deal fields |
//! | `move_deal_stage`    | Moves a deal to a new pipeline stage |
//! | `delete_deal`        | Soft-deletes a deal |
//! | `get_pipeline_summary`| Returns per-stage counts and values |

use tauri::State;

use crate::crm_engine::deals::{validate_deal_for_create, DealInput};
use crate::storage::deals::{self, Deal, PipelineSummary};
use crate::storage::sync;
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// create_deal
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new deal and records a sync changelog entry.
///
/// Validates input via [`validate_deal_for_create`] before writing.
///
/// # Errors
///
/// Returns a `String` error on validation or database failure.
#[tauri::command]
pub async fn create_deal(
    state: State<'_, AppState>,
    title: String,
    value: Option<f64>,
    currency: Option<String>,
    stage: Option<String>,
    probability: Option<i32>,
    expected_close: Option<String>,
    contact_id: Option<String>,
    notes: Option<String>,
) -> Result<Deal, String> {
    let input = DealInput {
        title: Some(title.clone()),
        value,
        currency: currency.clone(),
        stage: stage.clone(),
        probability,
        expected_close: expected_close.clone(),
        contact_id: contact_id.clone(),
        notes: notes.clone(),
    };

    validate_deal_for_create(&input).map_err(|e| e.to_string())?;

    // Acquire engine first (locking order: engine → db).
    let prob = probability.unwrap_or_else(|| {
        let stage_name = stage.as_deref().unwrap_or("Lead");
        let engine = state.engine.lock().unwrap();
        engine.default_probability(stage_name)
    });

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let deal = deals::create_deal(
        &db.conn,
        &title,
        value.unwrap_or(0.0),
        currency.as_deref().unwrap_or("USD"),
        stage.as_deref().unwrap_or("Lead"),
        prob,
        expected_close.as_deref(),
        contact_id.as_deref(),
        notes.as_deref().unwrap_or(""),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "deal", &deal.id, "__create__", None, Some(&deal.id), &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: create_deal id={} title={}", deal.id, deal.title);
    Ok(deal)
}

// ─────────────────────────────────────────────────────────────────────────────
// get_deal
// ─────────────────────────────────────────────────────────────────────────────

/// Retrieves a single deal by UUID.
#[tauri::command]
pub async fn get_deal(
    state: State<'_, AppState>,
    id: String,
) -> Result<Deal, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_deal id={}", id);
    deals::get_deal(&db.conn, &id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_deals
// ─────────────────────────────────────────────────────────────────────────────

/// Lists all active deals, ordered by creation date descending.
#[tauri::command]
pub async fn list_deals(state: State<'_, AppState>) -> Result<Vec<Deal>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: list_deals");
    deals::list_deals(&db.conn).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_deals_by_stage
// ─────────────────────────────────────────────────────────────────────────────

/// Lists all active deals in a specific pipeline stage.
///
/// # Parameters
///
/// - `stage` — Pipeline stage name (e.g. `"Proposal"`).
#[tauri::command]
pub async fn list_deals_by_stage(
    state: State<'_, AppState>,
    stage: String,
) -> Result<Vec<Deal>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: list_deals_by_stage stage={}", stage);
    deals::list_deals_by_stage(&db.conn, &stage).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// update_deal
// ─────────────────────────────────────────────────────────────────────────────

/// Updates a deal's fields.
///
/// Fields not provided (null in JSON) retain their current values.
#[tauri::command]
pub async fn update_deal(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    value: Option<f64>,
    currency: Option<String>,
    stage: Option<String>,
    probability: Option<i32>,
    expected_close: Option<String>,
    contact_id: Option<String>,
    notes: Option<String>,
) -> Result<Deal, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let deal = deals::update_deal(
        &db.conn,
        &id,
        title.as_deref(),
        value,
        currency.as_deref(),
        stage.as_deref(),
        probability,
        Some(expected_close.as_deref()),
        Some(contact_id.as_deref()),
        notes.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "deal", &id, "__update__", None, Some(&id), &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: update_deal id={}", id);
    Ok(deal)
}

// ─────────────────────────────────────────────────────────────────────────────
// move_deal_stage
// ─────────────────────────────────────────────────────────────────────────────

/// Moves a deal to a new pipeline stage.
///
/// Automatically sets the default probability for the target stage unless
/// `probability` is explicitly provided.
#[tauri::command]
pub async fn move_deal_stage(
    state: State<'_, AppState>,
    id: String,
    stage: String,
    probability: Option<i32>,
) -> Result<Deal, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let deal = deals::move_deal_stage(&db.conn, &id, &stage, probability)
        .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "deal",
        &id,
        "stage",
        None,
        Some(&stage),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    log::info!("Command: move_deal_stage id={} stage={}", id, stage);
    Ok(deal)
}

// ─────────────────────────────────────────────────────────────────────────────
// delete_deal
// ─────────────────────────────────────────────────────────────────────────────

/// Soft-deletes a deal.
#[tauri::command]
pub async fn delete_deal(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    deals::soft_delete_deal(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "deal", &id, "__delete__", Some(&id), None, &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: delete_deal id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// get_pipeline_summary
// ─────────────────────────────────────────────────────────────────────────────

/// Returns an aggregated count and value summary for each pipeline stage.
///
/// Used by the Kanban board and pipeline analytics views.
#[tauri::command]
pub async fn get_pipeline_summary(
    state: State<'_, AppState>,
) -> Result<Vec<PipelineSummary>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_pipeline_summary");
    deals::get_pipeline_summary(&db.conn).map_err(|e| e.to_string())
}
