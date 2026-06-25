use std::{fs, io::BufWriter};

use crate::result::CrmResult;
use crate::storage::{self, external_clients::ExternalClient};
use crate::utils::csv::{write_external_clients_csv, ExternalClientCsvRow};

use super::external_client_permissions::required_external_client_field;
use super::CrmCore;

impl CrmCore {
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

    pub fn export_external_clients_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_external_clients_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_external_clients_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_external_client_rows(&self) -> CrmResult<Vec<ExternalClientCsvRow>> {
        Ok(
            storage::external_clients::list_external_clients_for_export(&self.db.conn)?
                .into_iter()
                .map(external_client_export_row)
                .collect(),
        )
    }
}

fn external_client_export_row(client: ExternalClient) -> ExternalClientCsvRow {
    ExternalClientCsvRow {
        id: client.id,
        name: client.name,
        client_type: client.client_type,
        permission_mode: client.permission_mode,
        enabled: client.enabled,
        created_at: client.created_at,
        updated_at: client.updated_at,
        deleted_at: client.deleted_at,
        device_id: client.device_id,
    }
}
