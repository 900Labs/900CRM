//! Tauri IPC commands for reporting and analytics metrics.

use tauri::State;

use crate::storage::reporting::{
    self, ActivityFunnelReport, PipelineConversionReport,
};
use crate::AppState;

#[tauri::command]
pub async fn get_pipeline_conversion_report(
    state: State<'_, AppState>,
) -> Result<PipelineConversionReport, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    reporting::get_pipeline_conversion_report(&db.conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_funnel_report(
    state: State<'_, AppState>,
) -> Result<ActivityFunnelReport, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    reporting::get_activity_funnel_report(&db.conn).map_err(|e| e.to_string())
}
