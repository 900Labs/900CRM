use crate::result::CrmResult;
use crate::storage::{self, external_clients::ExternalClient, proposed_actions::ProposedAction};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn list_pending_proposed_actions(&self) -> CrmResult<Vec<ProposedAction>> {
        storage::proposed_actions::list_pending_proposed_actions(&self.db.conn)
    }

    pub fn list_external_clients(&self) -> CrmResult<Vec<ExternalClient>> {
        storage::external_clients::list_external_clients(&self.db.conn)
    }

    pub fn create_external_client_placeholder(
        &mut self,
        name: &str,
        client_type: &str,
    ) -> CrmResult<ExternalClient> {
        let name = required_external_client_field("name", name)?;
        let client_type = required_external_client_field("client_type", client_type)?;

        storage::external_clients::create_external_client_placeholder(
            &self.db.conn,
            &name,
            &client_type,
            &self.device_id,
        )
    }

    pub fn create_external_proposed_action_stub(
        &mut self,
        client_id: Option<String>,
        action_type: String,
        tool_name: String,
        entity_type: Option<String>,
        entity_id: Option<String>,
        input_json: String,
        proposed_output_json: Option<String>,
    ) -> CrmResult<ProposedAction> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let proposed_action = storage::proposed_actions::create_proposed_action(
            &tx,
            client_id.as_deref(),
            &action_type,
            &tool_name,
            entity_type.as_deref(),
            entity_id.as_deref(),
            &input_json,
            proposed_output_json.as_deref(),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            "mcp_client",
            "propose_action",
            Some("proposed_action"),
            Some(&proposed_action.id),
            None::<&()>,
            Some(&proposed_action),
            &device_id,
        )?;
        tx.commit()?;
        Ok(proposed_action)
    }
}

fn required_external_client_field(field: &str, value: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!(
            "External client {} is required",
            field
        )));
    }

    Ok(trimmed.to_string())
}
