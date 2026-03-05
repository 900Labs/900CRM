//! Tauri IPC commands for custom field definitions and values.

use tauri::State;

use crate::storage::custom_fields::{
    self, CustomFieldDefinition, CustomFieldValue, EntityCustomFieldValue,
};
use crate::storage::sync;
use crate::AppState;

#[tauri::command]
pub async fn list_custom_field_defs(
    state: State<'_, AppState>,
    entity_type: Option<String>,
) -> Result<Vec<CustomFieldDefinition>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    custom_fields::list_definitions(&db.conn, entity_type.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_custom_field_def(
    state: State<'_, AppState>,
    entity_type: String,
    field_name: String,
    field_type: String,
    field_options: Option<String>,
    sort_order: Option<i32>,
) -> Result<CustomFieldDefinition, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let definition = custom_fields::create_definition(
        &db.conn,
        &entity_type,
        &field_name,
        &field_type,
        field_options.as_deref(),
        sort_order.unwrap_or(0),
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "custom_field_def",
        &definition.id,
        "__create__",
        None,
        Some(&definition.id),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    Ok(definition)
}

#[tauri::command]
pub async fn update_custom_field_def(
    state: State<'_, AppState>,
    id: String,
    field_name: Option<String>,
    field_type: Option<String>,
    field_options: Option<String>,
    sort_order: Option<i32>,
) -> Result<CustomFieldDefinition, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let definition = custom_fields::update_definition(
        &db.conn,
        &id,
        field_name.as_deref(),
        field_type.as_deref(),
        field_options.as_deref(),
        sort_order,
    )
    .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "custom_field_def",
        &definition.id,
        "__update__",
        None,
        Some(&definition.id),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    Ok(definition)
}

#[tauri::command]
pub async fn delete_custom_field_def(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    custom_fields::delete_definition(&db.conn, &id).map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "custom_field_def",
        &id,
        "__delete__",
        Some(&id),
        None,
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_custom_field_value(
    state: State<'_, AppState>,
    field_def_id: String,
    entity_id: String,
    value: String,
) -> Result<CustomFieldValue, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let device_id = state.device_id.clone();

    let field_value = custom_fields::set_value(&db.conn, &field_def_id, &entity_id, &value)
        .map_err(|e| e.to_string())?;

    sync::record_change(
        &db.conn,
        "custom_field_value",
        &field_value.id,
        "value",
        None,
        Some(&field_value.value),
        &device_id,
    )
    .map_err(|e| e.to_string())?;

    Ok(field_value)
}

#[tauri::command]
pub async fn list_custom_field_values(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<EntityCustomFieldValue>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    custom_fields::list_values_for_entity(&db.conn, &entity_type, &entity_id)
        .map_err(|e| e.to_string())
}
