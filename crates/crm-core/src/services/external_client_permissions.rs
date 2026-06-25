use std::{fs, io::BufWriter};

use rusqlite::Connection;

use crate::audit::ACTOR_DESKTOP_APP;
use crate::permissions::{
    evaluate_tool_draft_permission, evaluate_tool_read_permission, ExternalClientPermissionMode,
    ToolPermissionEvaluation,
};
use crate::result::CrmResult;
use crate::storage::{
    self, external_client_permissions::ExternalClientPermission, external_clients::ExternalClient,
};
use crate::utils::{
    csv::{write_external_client_permissions_csv, ExternalClientPermissionCsvRow},
    errors::CrmError,
};

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn list_external_client_permissions(
        &self,
        client_id: &str,
    ) -> CrmResult<Vec<ExternalClientPermission>> {
        let client_id = required_external_client_field("client_id", client_id)?;
        require_existing_external_client(&self.db.conn, &client_id)?;

        storage::external_client_permissions::list_permissions_for_client(&self.db.conn, &client_id)
    }

    pub fn export_external_client_permissions_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_permission_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_external_client_permissions_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_external_client_permissions_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_permission_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_external_client_permission_rows(
        &self,
    ) -> CrmResult<Vec<ExternalClientPermissionCsvRow>> {
        Ok(
            storage::external_client_permissions::list_all_permissions_for_export(&self.db.conn)?
                .into_iter()
                .map(external_client_permission_export_row)
                .collect(),
        )
    }

    pub fn upsert_external_client_tool_permission(
        &mut self,
        client_id: &str,
        tool_name: &str,
        can_read: bool,
        can_write: bool,
        requires_confirmation: bool,
    ) -> CrmResult<ExternalClientPermission> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;
        validate_initial_permission_write(can_write, requires_confirmation)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        require_initial_external_client_mode(&tx, &client_id)?;
        let before = storage::external_client_permissions::get_permission_for_tool(
            &tx, &client_id, &tool_name,
        )?;
        let permission = storage::external_client_permissions::upsert_permission_for_tool(
            &tx,
            &client_id,
            &tool_name,
            can_read,
            can_write,
            requires_confirmation,
        )?;

        if before.as_ref() != Some(&permission) {
            let sync_field = if before.is_some() {
                "__update__"
            } else {
                "__create__"
            };
            storage::sync::record_change(
                &tx,
                "external_client_permission",
                &permission.id,
                sync_field,
                before.as_ref().map(|old| old.id.as_str()),
                Some(&permission.id),
                &device_id,
            )?;
            record_audit_json(
                &tx,
                ACTOR_DESKTOP_APP,
                "upsert_external_client_permission",
                Some("external_client_permission"),
                Some(&permission.id),
                before.as_ref(),
                Some(&permission),
                &device_id,
            )?;
        }

        tx.commit()?;
        Ok(permission)
    }

    pub fn evaluate_external_client_tool_read_permission(
        &self,
        client_id: &str,
        tool_name: &str,
    ) -> CrmResult<ToolPermissionEvaluation> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;

        evaluate_external_client_tool_read_permission(&self.db.conn, &client_id, &tool_name)
    }

    pub fn evaluate_external_client_draft_permission(
        &self,
        client_id: &str,
        tool_name: &str,
    ) -> CrmResult<ToolPermissionEvaluation> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;

        evaluate_external_client_draft_permission(&self.db.conn, &client_id, &tool_name)
    }
}

fn external_client_permission_export_row(
    permission: ExternalClientPermission,
) -> ExternalClientPermissionCsvRow {
    ExternalClientPermissionCsvRow {
        id: permission.id,
        client_id: permission.client_id,
        tool_name: permission.tool_name,
        can_read: permission.can_read,
        can_write: permission.can_write,
        requires_confirmation: permission.requires_confirmation,
        created_at: permission.created_at,
        updated_at: permission.updated_at,
    }
}

pub(super) fn ensure_external_client_draft_permission(
    conn: &Connection,
    client_id: &str,
    tool_name: &str,
) -> CrmResult<ToolPermissionEvaluation> {
    let evaluation = evaluate_external_client_draft_permission(conn, client_id, tool_name)?;
    if evaluation.allowed {
        return Ok(evaluation);
    }

    Err(CrmError::InvalidInput(format!(
        "External client '{}' may not create draft proposed actions for tool '{}': {}",
        client_id,
        tool_name,
        evaluation.reason.as_str()
    )))
}

pub(super) fn required_external_client_field(field: &str, value: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!(
            "External client {} is required",
            field
        )));
    }

    Ok(trimmed.to_string())
}

fn evaluate_external_client_tool_read_permission(
    conn: &Connection,
    client_id: &str,
    tool_name: &str,
) -> CrmResult<ToolPermissionEvaluation> {
    let mode = external_client_mode_for_evaluation(conn, client_id)?;
    let permission =
        storage::external_client_permissions::get_permission_for_tool(conn, client_id, tool_name)?;

    Ok(evaluate_tool_read_permission(
        mode,
        tool_name,
        permission.as_ref().map(ExternalClientPermission::grant),
    ))
}

fn evaluate_external_client_draft_permission(
    conn: &Connection,
    client_id: &str,
    tool_name: &str,
) -> CrmResult<ToolPermissionEvaluation> {
    let mode = external_client_mode_for_evaluation(conn, client_id)?;
    let permission =
        storage::external_client_permissions::get_permission_for_tool(conn, client_id, tool_name)?;

    Ok(evaluate_tool_draft_permission(
        mode,
        tool_name,
        permission.as_ref().map(ExternalClientPermission::grant),
    ))
}

fn require_existing_external_client(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClient> {
    storage::external_clients::get_active_external_client(conn, client_id)?.ok_or_else(|| {
        CrmError::NotFound(format!(
            "External client '{}' was not found or has been deleted",
            client_id
        ))
    })
}

fn require_initial_external_client_mode(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClientPermissionMode> {
    let client = require_existing_external_client(conn, client_id)?;
    let mode = parse_external_client_mode(&client)?;
    if !mode.is_supported_initial_mode() {
        return Err(CrmError::InvalidInput(format!(
            "External client '{}' has future permission mode '{}' that is not enabled in this sprint",
            client_id,
            mode.as_str()
        )));
    }

    Ok(mode)
}

fn external_client_mode_for_evaluation(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClientPermissionMode> {
    let client = require_existing_external_client(conn, client_id)?;
    if !client.enabled {
        return Ok(ExternalClientPermissionMode::Disabled);
    }

    parse_external_client_mode(&client)
}

fn parse_external_client_mode(client: &ExternalClient) -> CrmResult<ExternalClientPermissionMode> {
    ExternalClientPermissionMode::from_storage_value(&client.permission_mode).ok_or_else(|| {
        CrmError::InvalidInput(format!(
            "External client '{}' has unsupported permission mode '{}'",
            client.id, client.permission_mode
        ))
    })
}

fn validate_initial_permission_write(
    can_write: bool,
    requires_confirmation: bool,
) -> CrmResult<()> {
    if can_write && !requires_confirmation {
        return Err(CrmError::InvalidInput(
            "External client write permissions must require confirmation in this sprint"
                .to_string(),
        ));
    }

    Ok(())
}
