use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{datetime::now_iso8601, errors::CrmResult, uuid::new_uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    pub id: String,
    pub client_id: Option<String>,
    pub action_type: String,
    pub tool_name: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub input_json: String,
    pub proposed_output_json: Option<String>,
    pub status: String,
    pub created_at: String,
    pub approved_at: Option<String>,
    pub rejected_at: Option<String>,
    pub executed_at: Option<String>,
    pub device_id: String,
}

#[allow(clippy::too_many_arguments)]
pub fn create_proposed_action(
    conn: &Connection,
    client_id: Option<&str>,
    action_type: &str,
    tool_name: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    input_json: &str,
    proposed_output_json: Option<&str>,
    device_id: &str,
) -> CrmResult<ProposedAction> {
    let id = new_uuid();
    let created_at = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO proposed_actions
            (id, client_id, action_type, tool_name, entity_type, entity_id,
             input_json, proposed_output_json, status, created_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9, ?10)
        "#,
        params![
            id,
            client_id,
            action_type,
            tool_name,
            entity_type,
            entity_id,
            input_json,
            proposed_output_json,
            created_at,
            device_id
        ],
    )?;

    Ok(ProposedAction {
        id,
        client_id: client_id.map(str::to_string),
        action_type: action_type.to_string(),
        tool_name: tool_name.to_string(),
        entity_type: entity_type.map(str::to_string),
        entity_id: entity_id.map(str::to_string),
        input_json: input_json.to_string(),
        proposed_output_json: proposed_output_json.map(str::to_string),
        status: "pending".to_string(),
        created_at,
        approved_at: None,
        rejected_at: None,
        executed_at: None,
        device_id: device_id.to_string(),
    })
}

pub fn list_pending_proposed_actions(conn: &Connection) -> CrmResult<Vec<ProposedAction>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, client_id, action_type, tool_name, entity_type, entity_id,
               input_json, proposed_output_json, status, created_at,
               approved_at, rejected_at, executed_at, device_id
        FROM proposed_actions
        WHERE status = 'pending'
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ProposedAction {
            id: row.get(0)?,
            client_id: row.get(1)?,
            action_type: row.get(2)?,
            tool_name: row.get(3)?,
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
            input_json: row.get(6)?,
            proposed_output_json: row.get(7)?,
            status: row.get(8)?,
            created_at: row.get(9)?,
            approved_at: row.get(10)?,
            rejected_at: row.get(11)?,
            executed_at: row.get(12)?,
            device_id: row.get(13)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
