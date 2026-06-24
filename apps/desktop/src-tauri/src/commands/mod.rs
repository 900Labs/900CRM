use std::ops::{Deref, DerefMut};
use std::sync::MutexGuard;

use crm_core::CrmCore;
use tauri::State;

use crate::AppState;

pub mod activity_commands;
pub mod backup_commands;
pub mod contact_commands;
pub mod custom_field_commands;
pub mod dashboard_commands;
pub mod deal_commands;
pub mod email_commands;
pub mod import_export;
pub mod note_commands;
pub mod organization_commands;
pub mod report_commands;
pub mod settings_commands;
pub mod sync_commands;
pub mod tag_commands;

pub(crate) struct CoreGuard<'a> {
    guard: MutexGuard<'a, Option<CrmCore>>,
}

impl Deref for CoreGuard<'_> {
    type Target = CrmCore;

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_ref()
            .expect("CrmCore slot should be populated after lock_core succeeds")
    }
}

impl DerefMut for CoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .as_mut()
            .expect("CrmCore slot should be populated after lock_core succeeds")
    }
}

pub(crate) fn lock_core<'a>(state: &'a State<'_, AppState>) -> Result<CoreGuard<'a>, String> {
    let guard = state
        .core
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    if guard.is_none() {
        return Err("CrmCore is unavailable during local restore".to_string());
    }
    Ok(CoreGuard { guard })
}
