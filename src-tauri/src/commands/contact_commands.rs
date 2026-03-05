//! Tauri IPC commands for contact management.
//!
//! All commands in this module take a `State<'_, AppState>` parameter and
//! return `Result<T, String>` (error is serialized to string for IPC transport).
//!
//! # Commands
//!
//! | Command            | Description |
//! |--------------------|-------------|
//! | `create_contact`   | Creates a new contact after validation |
//! | `get_contact`      | Retrieves a single contact by ID |
//! | `list_contacts`    | Paginated, sorted, filtered contact list |
//! | `update_contact`   | Updates contact fields |
//! | `delete_contact`   | Soft-deletes a contact |
//! | `restore_contact`  | Restores a soft-deleted contact |
//! | `search_contacts`  | Full-text search via FTS5 |
//! | `merge_contacts`   | Merges two contacts into one |

use tauri::State;

use crate::crm_engine::contacts::{validate_contact_for_create, ContactInput};
use crate::storage::contacts::{
    self, Contact, ContactListParams, ContactListResult,
};
use crate::storage::sync;
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// create_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new CRM contact.
///
/// Validates the input using [`crate::crm_engine::contacts::validate_contact_for_create`],
/// writes the record to SQLite, and records a sync changelog entry.
///
/// # Parameters (all from frontend JSON payload)
///
/// - `contact_type` — `"person"` or `"organization"`.
/// - `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
///   `city`, `country`, `org_id`, `notes` — contact fields.
///
/// # Errors
///
/// Returns a `String` error message on validation or database failure.
#[tauri::command]
pub async fn create_contact(
    state: State<'_, AppState>,
    contact_type: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    org_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    org_id: Option<String>,
    notes: Option<String>,
) -> Result<Contact, String> {
    let input = ContactInput {
        contact_type: contact_type.clone(),
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        org_name: org_name.clone(),
        email: email.clone(),
        phone: phone.clone(),
        address: address.clone(),
        city: city.clone(),
        country: country.clone(),
        org_id: org_id.clone(),
        notes: notes.clone(),
    };

    validate_contact_for_create(&input).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let contact = contacts::create_contact(
        &db.conn,
        contact_type.as_deref().unwrap_or("person"),
        first_name.as_deref().unwrap_or(""),
        last_name.as_deref().unwrap_or(""),
        org_name.as_deref().unwrap_or(""),
        email.as_deref().unwrap_or(""),
        phone.as_deref().unwrap_or(""),
        address.as_deref().unwrap_or(""),
        city.as_deref().unwrap_or(""),
        country.as_deref().unwrap_or(""),
        org_id.as_deref(),
        notes.as_deref().unwrap_or(""),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "contact",
        &contact.id,
        "__create__",
        None,
        Some(&contact.id),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    log::info!("Command: create_contact id={}", contact.id);
    Ok(contact)
}

// ─────────────────────────────────────────────────────────────────────────────
// get_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Retrieves a single contact by UUID.
///
/// # Errors
///
/// - `"Not found: ..."` if the ID doesn't exist or is soft-deleted.
/// - `"Database error: ..."` on SQL failure.
#[tauri::command]
pub async fn get_contact(
    state: State<'_, AppState>,
    id: String,
) -> Result<Contact, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: get_contact id={}", id);
    contacts::get_contact(&db.conn, &id).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// list_contacts
// ─────────────────────────────────────────────────────────────────────────────

/// Lists contacts with pagination, sorting, and optional filtering.
///
/// # Parameters
///
/// - `params` — [`ContactListParams`] JSON object.
///
/// # Returns
///
/// A [`ContactListResult`] with the current page of contacts and the total count.
#[tauri::command]
pub async fn list_contacts(
    state: State<'_, AppState>,
    params: Option<ContactListParams>,
) -> Result<ContactListResult, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let p = params.unwrap_or_default();
    log::debug!("Command: list_contacts page={} per_page={}", p.page, p.per_page);
    contacts::list_contacts(&db.conn, &p).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// update_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Updates a contact's fields.
///
/// Only fields that are explicitly provided (non-null in the JSON) are updated.
/// Omitted fields retain their current values.
///
/// Records a sync changelog entry for each changed field.
#[tauri::command]
pub async fn update_contact(
    state: State<'_, AppState>,
    id: String,
    contact_type: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    org_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    notes: Option<String>,
) -> Result<Contact, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let contact = contacts::update_contact(
        &db.conn,
        &id,
        contact_type.as_deref(),
        first_name.as_deref(),
        last_name.as_deref(),
        org_name.as_deref(),
        email.as_deref(),
        phone.as_deref(),
        address.as_deref(),
        city.as_deref(),
        country.as_deref(),
        None, // org_id not updated via this command
        notes.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "contact",
        &id,
        "__update__",
        None,
        Some(&id),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    log::info!("Command: update_contact id={}", id);
    Ok(contact)
}

// ─────────────────────────────────────────────────────────────────────────────
// delete_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Soft-deletes a contact.
///
/// The contact is hidden from all list/search operations but not permanently
/// removed from the database.
#[tauri::command]
pub async fn delete_contact(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    contacts::soft_delete_contact(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "contact", &id, "__delete__", Some(&id), None, &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: delete_contact id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// restore_contact
// ─────────────────────────────────────────────────────────────────────────────

/// Restores a previously soft-deleted contact.
#[tauri::command]
pub async fn restore_contact(
    state: State<'_, AppState>,
    id: String,
) -> Result<Contact, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let contact = contacts::restore_contact(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(&db.conn, "contact", &id, "__restore__", None, Some(&id), &device_id)
        .map_err(|e| e.to_string())?;

    log::info!("Command: restore_contact id={}", id);
    Ok(contact)
}

// ─────────────────────────────────────────────────────────────────────────────
// search_contacts
// ─────────────────────────────────────────────────────────────────────────────

/// Full-text searches contacts using the FTS5 virtual table.
///
/// # Parameters
///
/// - `query` — Search string. Returns empty list if blank.
///
/// # Returns
///
/// Up to 50 matching contacts ordered by FTS5 rank.
#[tauri::command]
pub async fn search_contacts(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Contact>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    log::debug!("Command: search_contacts query={}", query);
    contacts::search_contacts(&db.conn, &query).map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// merge_contacts
// ─────────────────────────────────────────────────────────────────────────────

/// Merges `source_id` into `target_id`.
///
/// The source contact is soft-deleted after the merge. The merged target
/// contact is returned.
///
/// # Errors
///
/// - `"Invalid input: ..."` if target and source are the same ID.
/// - `"Not found: ..."` if either contact doesn't exist.
#[tauri::command]
pub async fn merge_contacts(
    state: State<'_, AppState>,
    target_id: String,
    source_id: String,
) -> Result<Contact, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();
    log::info!("Command: merge_contacts target={} source={}", target_id, source_id);
    crate::crm_engine::contacts::merge_contacts(&db.conn, &target_id, &source_id, &device_id)
        .map_err(|e| e.to_string())
}
