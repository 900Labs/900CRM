//! CSV import and export commands for contacts and deals.
//!
//! These commands use Tauri's file dialog plugin to let the user choose
//! source/destination files, then read or write CSV data via the
//! [`crate::utils::csv`] helpers.
//!
//! # Commands
//!
//! | Command                | Description |
//! |------------------------|-------------|
//! | `import_contacts_csv`  | Reads a CSV file and creates contacts |
//! | `export_contacts_csv`  | Writes all contacts to a CSV file |
//! | `import_deals_csv`     | Reads a CSV file and creates deals |
//! | `export_deals_csv`     | Writes all deals to a CSV file |

use std::fs;
use std::io::BufWriter;

use tauri::State;
use serde::{Deserialize, Serialize};

use crate::storage::{contacts, deals, sync};
use crate::utils::csv::{
    parse_contacts_csv, parse_deals_csv, write_contacts_csv, write_deals_csv,
    ContactCsvRow, DealCsvRow,
};
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Result structs
// ─────────────────────────────────────────────────────────────────────────────

/// Summary of a completed import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of records successfully created.
    pub created: u32,

    /// Number of rows that were skipped due to errors.
    pub skipped: u32,

    /// Human-readable messages for any skipped rows.
    pub errors: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// import_contacts_csv
// ─────────────────────────────────────────────────────────────────────────────

/// Imports contacts from a CSV file at the given path.
///
/// Each row in the CSV becomes a new contact. Rows that fail validation are
/// skipped and reported in the [`ImportResult::errors`] list.
///
/// # Parameters
///
/// - `file_path` — Absolute path to the CSV file to import.
///
/// # Errors
///
/// Returns a `String` error if the file cannot be read or is not valid CSV.
#[tauri::command]
pub async fn import_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    log::info!("Command: import_contacts_csv path={}", file_path);

    let file_content = fs::read(&file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    let rows = parse_contacts_csv(file_content.as_slice()).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let mut created = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        match contacts::create_contact(
            &db.conn,
            "person",
            &row.first_name,
            row.last_name.as_deref().unwrap_or(""),
            row.org_name.as_deref().unwrap_or(""),
            row.email.as_deref().unwrap_or(""),
            row.phone.as_deref().unwrap_or(""),
            row.address.as_deref().unwrap_or(""),
            row.city.as_deref().unwrap_or(""),
            row.country.as_deref().unwrap_or(""),
            None,
            row.notes.as_deref().unwrap_or(""),
            &device_id,
        ) {
            Ok(contact) => {
                let _ = sync::record_change(
                    &db.conn,
                    "contact",
                    &contact.id,
                    "__create__",
                    None,
                    Some(&contact.id),
                    &device_id,
                );
                created += 1;
            }
            Err(e) => {
                let msg = format!("Row {}: {} ({})", i + 2, e, row.first_name);
                log::warn!("import_contacts_csv skip: {}", msg);
                errors.push(msg);
                skipped += 1;
            }
        }
    }

    log::info!(
        "import_contacts_csv: created={} skipped={}",
        created,
        skipped
    );
    Ok(ImportResult { created, skipped, errors })
}

// ─────────────────────────────────────────────────────────────────────────────
// export_contacts_csv
// ─────────────────────────────────────────────────────────────────────────────

/// Exports all active contacts to a CSV file at the given path.
///
/// # Parameters
///
/// - `file_path` — Absolute path where the CSV file will be written.
///
/// # Errors
///
/// Returns a `String` error if the file cannot be written.
#[tauri::command]
pub async fn export_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    log::info!("Command: export_contacts_csv path={}", file_path);

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let params = contacts::ContactListParams {
        page: 1,
        per_page: 100_000,
        sort_by: "first_name".to_string(),
        sort_dir: "asc".to_string(),
        filter_type: None,
        search_query: None,
    };

    let result = contacts::list_contacts(&db.conn, &params).map_err(|e| e.to_string())?;

    let rows: Vec<ContactCsvRow> = result
        .contacts
        .iter()
        .map(|c| ContactCsvRow {
            first_name: c.first_name.clone(),
            last_name: Some(c.last_name.clone()).filter(|s| !s.is_empty()),
            org_name: Some(c.org_name.clone()).filter(|s| !s.is_empty()),
            email: Some(c.email.clone()).filter(|s| !s.is_empty()),
            phone: Some(c.phone.clone()).filter(|s| !s.is_empty()),
            address: Some(c.address.clone()).filter(|s| !s.is_empty()),
            city: Some(c.city.clone()).filter(|s| !s.is_empty()),
            country: Some(c.country.clone()).filter(|s| !s.is_empty()),
            notes: Some(c.notes.clone()).filter(|s| !s.is_empty()),
        })
        .collect();

    let count = rows.len() as u32;

    let file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create file '{}': {}", file_path, e))?;
    let writer = BufWriter::new(file);
    write_contacts_csv(writer, &rows).map_err(|e| e.to_string())?;

    log::info!("export_contacts_csv: wrote {} contacts", count);
    Ok(count)
}

// ─────────────────────────────────────────────────────────────────────────────
// import_deals_csv
// ─────────────────────────────────────────────────────────────────────────────

/// Imports deals from a CSV file at the given path.
///
/// Each row in the CSV becomes a new deal. Rows that fail are skipped.
///
/// # Parameters
///
/// - `file_path` — Absolute path to the CSV file to import.
///
/// # Errors
///
/// Returns a `String` error if the file cannot be read or is not valid CSV.
#[tauri::command]
pub async fn import_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    log::info!("Command: import_deals_csv path={}", file_path);

    let file_content = fs::read(&file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    let rows = parse_deals_csv(file_content.as_slice()).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let mut created = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        let value: f64 = row
            .value
            .as_deref()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        match deals::create_deal(
            &db.conn,
            &row.title,
            value,
            row.currency.as_deref().unwrap_or("USD"),
            row.stage.as_deref().unwrap_or("Lead"),
            0,
            row.expected_close.as_deref(),
            None,
            row.notes.as_deref().unwrap_or(""),
            &device_id,
        ) {
            Ok(deal) => {
                let _ = sync::record_change(
                    &db.conn,
                    "deal",
                    &deal.id,
                    "__create__",
                    None,
                    Some(&deal.id),
                    &device_id,
                );
                created += 1;
            }
            Err(e) => {
                let msg = format!("Row {}: {} ({})", i + 2, e, row.title);
                log::warn!("import_deals_csv skip: {}", msg);
                errors.push(msg);
                skipped += 1;
            }
        }
    }

    log::info!("import_deals_csv: created={} skipped={}", created, skipped);
    Ok(ImportResult { created, skipped, errors })
}

// ─────────────────────────────────────────────────────────────────────────────
// export_deals_csv
// ─────────────────────────────────────────────────────────────────────────────

/// Exports all active deals to a CSV file at the given path.
///
/// # Parameters
///
/// - `file_path` — Absolute path where the CSV file will be written.
///
/// # Errors
///
/// Returns a `String` error if the file cannot be written.
#[tauri::command]
pub async fn export_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    log::info!("Command: export_deals_csv path={}", file_path);

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let all_deals = deals::list_deals(&db.conn).map_err(|e| e.to_string())?;

    let rows: Vec<DealCsvRow> = all_deals
        .iter()
        .map(|d| DealCsvRow {
            title: d.title.clone(),
            value: Some(format!("{:.2}", d.value)),
            currency: Some(d.currency.clone()),
            stage: Some(d.stage.clone()),
            expected_close: d.expected_close.clone(),
            notes: Some(d.notes.clone()).filter(|s| !s.is_empty()),
        })
        .collect();

    let count = rows.len() as u32;

    let file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create file '{}': {}", file_path, e))?;
    let writer = BufWriter::new(file);
    write_deals_csv(writer, &rows).map_err(|e| e.to_string())?;

    log::info!("export_deals_csv: wrote {} deals", count);
    Ok(count)
}
