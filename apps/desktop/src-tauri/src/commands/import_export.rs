use crm_core::services::{ImportPreflightReport, ImportResult};
use crm_core::utils::csv::ImportColumnMapping;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn import_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_contacts_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_contacts_csv_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_contacts_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_contacts_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_deals_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_deals_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_deals_csv_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_deals_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_deals_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_deals_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_deals_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_deals_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_organizations_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_organizations_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_organizations_csv_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let mut core = super::lock_core(&state)?;
    core.import_organizations_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_organizations_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_organizations_json(&file_path)
        .map_err(|e| e.to_string())
}
