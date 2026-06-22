use std::sync::MutexGuard;

use crm_core::CrmCore;
use tauri::State;

use crate::AppState;

pub mod activity_commands;
pub mod contact_commands;
pub mod custom_field_commands;
pub mod dashboard_commands;
pub mod deal_commands;
pub mod import_export;
pub mod organization_commands;
pub mod settings_commands;
pub mod sync_commands;

pub(crate) fn lock_core<'a>(
    state: &'a State<'_, AppState>,
) -> Result<MutexGuard<'a, CrmCore>, String> {
    state.core.lock().map_err(|e| format!("Lock error: {}", e))
}
