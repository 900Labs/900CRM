use std::collections::BTreeMap;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::audit::{ACTOR_DESKTOP_APP, ACTOR_IMPORT};
use crate::crm_engine::{activities as activity_engine, deals as deal_engine, CrmEngine};
use crate::result::CrmResult;
use crate::services::activity_relationships::{
    create_activity_links_for_legacy_mirrors, sync_activity_links_after_mirror_update,
};
use crate::services::deal_relationships::{
    create_primary_deal_contact_for_mirror, sync_primary_deal_contact_after_mirror_update,
};
use crate::storage::{
    self,
    activities::Activity,
    audit::AuditLogEntry,
    contacts::ContactListParams,
    custom_fields::{
        CustomFieldDefinition, CustomFieldValue, EntityCustomFieldValue, EntityTypeCustomFieldValue,
    },
    dashboard::CurrencyPipelineValue,
    deals::{Deal, PipelineSummary},
    reporting::{ActivityFunnelReport, PipelineConversionReport},
};
use crate::utils::{
    csv::{
        parse_contacts_csv_with_mapping, parse_contacts_csv_with_row_numbers,
        parse_contacts_json_with_mapping, parse_contacts_json_with_row_numbers,
        parse_deals_csv_with_mapping, parse_deals_csv_with_row_numbers,
        parse_deals_json_with_mapping, parse_deals_json_with_row_numbers,
        parse_organizations_csv_with_mapping, parse_organizations_csv_with_row_numbers,
        parse_organizations_json_with_mapping, parse_organizations_json_with_row_numbers,
        preview_contacts_json_import, preview_deals_json_import, preview_organizations_json_import,
        write_contacts_csv, write_deals_csv, write_organizations_csv, ContactCsvRow, DealCsvRow,
        ImportColumnMapping, JsonImportPreview, OrganizationCsvRow,
    },
    datetime::now_iso8601,
    errors::{CrmError, CrmResult as InternalCrmResult},
    uuid::new_uuid,
};

mod activity_relationships;
mod audit;
mod backup;
mod contacts;
mod deal_relationships;
mod external_client_permissions;
mod migration_readiness;
mod notes_tags;
mod organizations;
mod proposed_actions;
mod search;
mod settings;

pub use backup::{LocalBackup, LocalBackupMetadata, LocalBackupValidation, LocalRestoreResult};
pub use migration_readiness::NormalizationMigrationPreflight;
pub use notes_tags::TagColorUpdate;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_contacts: i64,
    pub total_organizations: i64,
    pub active_deals: i64,
    pub pipeline_value: f64,
    pub pipeline_value_by_currency: Vec<CurrencyPipelineValue>,
    pub weighted_pipeline: f64,
    pub upcoming_activities: i64,
    pub overdue_activities: i64,
    pub win_rate: f64,
    pub new_contacts_this_month: i64,
    pub new_deals_this_month: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub created: u32,
    #[serde(default)]
    pub merged: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImportOptions {
    pub merge_duplicates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreflightReport {
    pub entity_type: String,
    pub total_rows: u32,
    pub duplicate_warning_count: u32,
    pub warnings: Vec<ImportDuplicateWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDuplicateWarning {
    pub entity_type: String,
    pub row_number: u32,
    pub match_type: String,
    pub csv_value: String,
    pub existing_entity_type: String,
    pub existing_entity_id: String,
    pub existing_display_label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: String,
    pub last_sync_at: Option<String>,
    pub error_message: Option<String>,
    pub pending_changes: u32,
}

pub struct CrmCore {
    pub(crate) engine: CrmEngine,
    pub(crate) db: storage::Database,
    pub(crate) device_id: String,
}

impl CrmCore {
    pub fn open(app_data_dir: &Path) -> CrmResult<Self> {
        let db = storage::Database::new(app_data_dir)?;
        let device_id = load_or_create_device_id(&db)?;

        Ok(Self {
            engine: CrmEngine::new(),
            db,
            device_id,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn default_stage_count(&self) -> usize {
        self.engine.default_stages.len()
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn create_deal(
        &mut self,
        title: String,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<String>,
        contact_id: Option<String>,
        organization_id: Option<String>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
        let contact_id = normalize_optional_string(contact_id);
        let organization_id = normalize_optional_string(organization_id);
        let input = deal_engine::DealInput {
            title: Some(title.clone()),
            value,
            currency: currency.clone(),
            stage: stage.clone(),
            probability,
            expected_close: expected_close.clone(),
            contact_id: contact_id.clone(),
            notes: notes.clone(),
        };
        deal_engine::validate_deal_for_create(&input)?;
        if let Some(contact_id) = contact_id.as_deref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(organization_id) = organization_id.as_deref() {
            storage::organizations::get_organization(&self.db.conn, organization_id)?;
        }

        let prob = probability.unwrap_or_else(|| {
            let stage_name = stage.as_deref().unwrap_or("Lead");
            self.engine.default_probability(stage_name)
        });
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::create_deal(
            &tx,
            &title,
            value.unwrap_or(0.0),
            currency.as_deref().unwrap_or("USD"),
            stage.as_deref().unwrap_or("Lead"),
            prob,
            expected_close.as_deref(),
            contact_id.as_deref(),
            organization_id.as_deref(),
            notes.as_deref().unwrap_or(""),
            &device_id,
        )?;
        create_primary_deal_contact_for_mirror(&tx, &deal, &device_id)?;
        let deal = storage::deals::get_deal(&tx, &deal.id)?;
        storage::sync::record_change(
            &tx,
            "deal",
            &deal.id,
            "__create__",
            None,
            Some(&deal.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("deal"),
            Some(&deal.id),
            None::<&()>,
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn get_deal(&self, id: &str) -> CrmResult<Deal> {
        storage::deals::get_deal(&self.db.conn, id)
    }

    pub fn list_deals(&self) -> CrmResult<Vec<Deal>> {
        storage::deals::list_deals(&self.db.conn)
    }

    pub fn list_deals_by_stage(&self, stage: &str) -> CrmResult<Vec<Deal>> {
        storage::deals::list_deals_by_stage(&self.db.conn, stage)
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn update_deal(
        &mut self,
        id: &str,
        title: Option<String>,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<Option<String>>,
        contact_id: Option<Option<String>>,
        organization_id: Option<Option<String>>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
        let expected_close = normalize_optional_update_string(expected_close);
        let contact_id = normalize_optional_update_string(contact_id);
        let organization_id = organization_id.map(normalize_optional_string);
        if let Some(Some(contact_id)) = contact_id.as_ref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(Some(organization_id)) = organization_id.as_ref() {
            storage::organizations::get_organization(&self.db.conn, organization_id)?;
        }
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::update_deal(
            &tx,
            id,
            title.as_deref(),
            value,
            currency.as_deref(),
            stage.as_deref(),
            probability,
            expected_close.as_ref().map(|value| value.as_deref()),
            contact_id.as_ref().map(|value| value.as_deref()),
            organization_id.as_ref().map(|value| value.as_deref()),
            notes.as_deref(),
        )?;
        sync_primary_deal_contact_after_mirror_update(&tx, &before, &deal, &device_id)?;
        let deal = storage::deals::get_deal(&tx, id)?;
        storage::sync::record_change(&tx, "deal", id, "__update__", None, Some(id), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("deal"),
            Some(id),
            Some(&before),
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn move_deal_stage(
        &mut self,
        id: &str,
        stage: &str,
        probability: Option<i32>,
    ) -> CrmResult<Deal> {
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::move_deal_stage(&tx, id, stage, probability)?;
        storage::sync::record_change(&tx, "deal", id, "stage", None, Some(stage), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "move_stage",
            Some("deal"),
            Some(id),
            Some(&before),
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn delete_deal(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::deals::soft_delete_deal(&tx, id)?;
        storage::sync::record_change(&tx, "deal", id, "__delete__", Some(id), None, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("deal"),
            Some(id),
            Some(&before),
            Option::<&Deal>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_pipeline_summary(&self) -> CrmResult<Vec<PipelineSummary>> {
        storage::deals::get_pipeline_summary(&self.db.conn)
    }

    pub fn create_activity(
        &mut self,
        activity_type: String,
        title: String,
        description: Option<String>,
        due_date: Option<String>,
        contact_id: Option<String>,
        deal_id: Option<String>,
    ) -> CrmResult<Activity> {
        activity_engine::validate_activity_for_create(&title, &activity_type)?;
        let contact_id = normalize_optional_string(contact_id);
        let deal_id = normalize_optional_string(deal_id);
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = create_activity_in_transaction(
            &tx,
            &device_id,
            &activity_type,
            &title,
            description.as_deref(),
            due_date.as_deref(),
            contact_id.as_deref(),
            deal_id.as_deref(),
        )?;
        tx.commit()?;
        Ok(activity)
    }

    pub fn get_activity(&self, id: &str) -> CrmResult<Activity> {
        storage::activities::get_activity(&self.db.conn, id)
    }

    pub fn list_activities(&self) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities(&self.db.conn)
    }

    pub fn list_activities_for_contact(&self, contact_id: &str) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_for_contact(&self.db.conn, contact_id)
    }

    pub fn list_activities_for_deal(&self, deal_id: &str) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_for_deal(&self.db.conn, deal_id)
    }

    pub fn list_upcoming_activities(&self, limit: u32) -> CrmResult<Vec<Activity>> {
        storage::activities::list_upcoming_activities(&self.db.conn, limit)
    }

    pub fn mark_activity_complete(&mut self, id: &str) -> CrmResult<Activity> {
        self.set_activity_completion(id, true)
    }

    pub fn mark_activity_incomplete(&mut self, id: &str) -> CrmResult<Activity> {
        self.set_activity_completion(id, false)
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn update_activity(
        &mut self,
        id: &str,
        activity_type: Option<String>,
        title: Option<String>,
        description: Option<String>,
        due_date: Option<Option<String>>,
        completed: Option<bool>,
        contact_id: Option<Option<String>>,
        deal_id: Option<Option<String>>,
    ) -> CrmResult<Activity> {
        let due_date = normalize_optional_update_string(due_date);
        let contact_id = normalize_optional_update_string(contact_id);
        let deal_id = normalize_optional_update_string(deal_id);
        if let Some(Some(contact_id)) = contact_id.as_ref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(Some(deal_id)) = deal_id.as_ref() {
            storage::deals::get_deal(&self.db.conn, deal_id)?;
        }
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = storage::activities::update_activity(
            &tx,
            id,
            activity_type.as_deref(),
            title.as_deref(),
            description.as_deref(),
            due_date.as_ref().map(|value| value.as_deref()),
            completed,
            contact_id.as_ref().map(|value| value.as_deref()),
            deal_id.as_ref().map(|value| value.as_deref()),
        )?;
        sync_activity_links_after_mirror_update(&tx, &before, &activity, &device_id)?;
        let activity = storage::activities::get_activity(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "activity",
            id,
            "__update__",
            None,
            Some(id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("activity"),
            Some(id),
            Some(&before),
            Some(&activity),
            &device_id,
        )?;
        tx.commit()?;
        Ok(activity)
    }

    pub fn delete_activity(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::activities::soft_delete_activity(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "activity",
            id,
            "__delete__",
            Some(id),
            None,
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("activity"),
            Some(id),
            Some(&before),
            Option::<&Activity>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_dashboard_stats(&self) -> CrmResult<DashboardStats> {
        let pipeline_summaries = storage::deals::get_pipeline_summary(&self.db.conn)?;
        let active_deals = pipeline_summaries
            .iter()
            .filter(|s| s.stage != "Closed Won" && s.stage != "Closed Lost")
            .map(|s| s.count)
            .sum();
        let pipeline_value = deal_engine::calculate_total_pipeline_value(&pipeline_summaries);
        let weighted_pipeline = deal_engine::calculate_weighted_pipeline(&pipeline_summaries);
        let win_rate = deal_engine::calculate_win_rate(&pipeline_summaries);
        let activity_stats = activity_engine::get_activity_stats(&self.db.conn)?;
        let now = now_iso8601();
        let counts =
            storage::dashboard::get_dashboard_counts(&self.db.conn, &format!("{}%", &now[..7]))?;
        let pipeline_value_by_currency =
            storage::dashboard::get_pipeline_value_by_currency(&self.db.conn)?;

        Ok(DashboardStats {
            total_contacts: counts.total_contacts,
            total_organizations: counts.total_organizations,
            active_deals,
            pipeline_value,
            pipeline_value_by_currency,
            weighted_pipeline,
            upcoming_activities: activity_stats.pending,
            overdue_activities: activity_stats.overdue,
            win_rate,
            new_contacts_this_month: counts.new_contacts_this_month,
            new_deals_this_month: counts.new_deals_this_month,
        })
    }

    pub fn list_custom_field_defs(
        &self,
        entity_type: Option<String>,
    ) -> CrmResult<Vec<CustomFieldDefinition>> {
        storage::custom_fields::list_definitions(&self.db.conn, entity_type.as_deref())
    }

    pub fn create_custom_field_def(
        &mut self,
        entity_type: String,
        field_name: String,
        field_type: String,
        field_options: Option<String>,
        sort_order: Option<i32>,
    ) -> CrmResult<CustomFieldDefinition> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let definition = storage::custom_fields::create_definition(
            &tx,
            &entity_type,
            &field_name,
            &field_type,
            field_options.as_deref(),
            sort_order.unwrap_or(0),
        )?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
            &definition.id,
            "__create__",
            None,
            Some(&definition.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("custom_field_def"),
            Some(&definition.id),
            None::<&()>,
            Some(&definition),
            &device_id,
        )?;
        tx.commit()?;
        Ok(definition)
    }

    pub fn update_custom_field_def(
        &mut self,
        id: &str,
        field_name: Option<String>,
        field_type: Option<String>,
        field_options: Option<String>,
        sort_order: Option<i32>,
    ) -> CrmResult<CustomFieldDefinition> {
        let before = storage::custom_fields::get_definition(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let definition = storage::custom_fields::update_definition(
            &tx,
            id,
            field_name.as_deref(),
            field_type.as_deref(),
            field_options.as_deref(),
            sort_order,
        )?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
            &definition.id,
            "__update__",
            None,
            Some(&definition.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("custom_field_def"),
            Some(&definition.id),
            Some(&before),
            Some(&definition),
            &device_id,
        )?;
        tx.commit()?;
        Ok(definition)
    }

    pub fn delete_custom_field_def(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::custom_fields::get_definition(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::custom_fields::delete_definition(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
            id,
            "__delete__",
            Some(id),
            None,
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("custom_field_def"),
            Some(id),
            Some(&before),
            Option::<&CustomFieldDefinition>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_custom_field_value(
        &mut self,
        field_def_id: String,
        entity_id: String,
        value: String,
    ) -> CrmResult<CustomFieldValue> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let field_value =
            storage::custom_fields::set_value(&tx, &field_def_id, &entity_id, &value)?;
        storage::sync::record_change(
            &tx,
            "custom_field_value",
            &field_value.id,
            "value",
            None,
            Some(&field_value.value),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "set_value",
            Some("custom_field_value"),
            Some(&field_value.id),
            None::<&()>,
            Some(&field_value),
            &device_id,
        )?;
        tx.commit()?;
        Ok(field_value)
    }

    pub fn list_custom_field_values(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<Vec<EntityCustomFieldValue>> {
        storage::custom_fields::list_values_for_entity(&self.db.conn, entity_type, entity_id)
    }

    pub fn list_custom_field_values_for_type(
        &self,
        entity_type: &str,
    ) -> CrmResult<Vec<EntityTypeCustomFieldValue>> {
        storage::custom_fields::list_values_for_entity_type(&self.db.conn, entity_type)
    }

    pub fn get_pipeline_conversion_report(&self) -> CrmResult<PipelineConversionReport> {
        storage::reporting::get_pipeline_conversion_report(&self.db.conn)
    }

    pub fn get_activity_funnel_report(&self) -> CrmResult<ActivityFunnelReport> {
        storage::reporting::get_activity_funnel_report(&self.db.conn)
    }

    pub fn get_sync_status(&self) -> CrmResult<SyncStatus> {
        let pending_changes = storage::sync::get_all_pending_changes(&self.db.conn)?.len() as u32;
        let last_sync_at = storage::sync::get_latest_change_timestamp(&self.db.conn)?;
        let sync_enabled = storage::settings::get_setting(&self.db.conn, "sync_enabled")?
            .map(|s| s.value)
            .unwrap_or_else(|| "false".to_string());
        let sync_url = storage::settings::get_setting(&self.db.conn, "sync_url")?
            .map(|s| s.value)
            .unwrap_or_default();

        sync_status_from_settings(
            sync_enabled.as_str(),
            sync_url.as_str(),
            last_sync_at,
            pending_changes,
        )
    }

    pub fn trigger_sync(&self) -> CrmResult<SyncStatus> {
        let pending_changes = storage::sync::get_all_pending_changes(&self.db.conn)?.len() as u32;
        let sync_enabled = storage::settings::get_setting(&self.db.conn, "sync_enabled")?
            .map(|s| s.value)
            .unwrap_or_else(|| "false".to_string());
        let sync_url = storage::settings::get_setting(&self.db.conn, "sync_url")?
            .map(|s| s.value)
            .unwrap_or_default();

        if !parse_bool(Some(sync_enabled.as_str())) {
            return Ok(SyncStatus {
                state: "idle".to_string(),
                last_sync_at: None,
                error_message: None,
                pending_changes,
            });
        }

        if sync_url.trim().is_empty() {
            return Ok(SyncStatus {
                state: "error".to_string(),
                last_sync_at: None,
                error_message: Some("Sync URL is not configured.".to_string()),
                pending_changes,
            });
        }

        Ok(SyncStatus {
            state: "success".to_string(),
            last_sync_at: Some(now_iso8601()),
            error_message: None,
            pending_changes,
        })
    }

    pub fn import_contacts_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_contacts_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_contacts_csv_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_row_numbers(file_content.as_slice())?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_contacts_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_contacts_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_contacts_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_contacts_json_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_row_numbers(file_content.as_slice())?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_contacts_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_contacts_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_contact_rows(rows, options)
    }

    pub fn preview_contacts_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_contacts_json_import(file_content.as_slice())
    }

    fn import_contact_rows(
        &mut self,
        rows: Vec<(usize, ContactCsvRow)>,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let mut created = 0u32;
        let mut merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (row_number, row) in rows {
            if options.merge_duplicates {
                match self.find_unique_contact_import_match(&row) {
                    Ok(Some(contact)) => match self.merge_contact_import_row(&contact.id, &row) {
                        Ok(()) => {
                            let _ = storage::audit::record_audit(
                                &self.db.conn,
                                ACTOR_IMPORT,
                                None,
                                "import_row_merge",
                                Some("contact"),
                                Some(&contact.id),
                                None,
                                None,
                                &self.device_id,
                            );
                            merged += 1;
                            continue;
                        }
                        Err(e) => {
                            errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                            skipped += 1;
                            continue;
                        }
                    },
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                        skipped += 1;
                        continue;
                    }
                }
            }

            match self.create_contact(
                Some("person".to_string()),
                Some(row.first_name.clone()),
                row.last_name.clone(),
                row.org_name.clone(),
                row.email.clone(),
                row.phone.clone(),
                row.address.clone(),
                row.city.clone(),
                row.country.clone(),
                None,
                row.notes.clone(),
            ) {
                Ok(contact) => {
                    let _ = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("contact"),
                        Some(&contact.id),
                        None,
                        None,
                        &self.device_id,
                    );
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
        })
    }

    fn find_unique_contact_import_match(
        &self,
        row: &ContactCsvRow,
    ) -> CrmResult<Option<storage::contacts::Contact>> {
        let mut matches = BTreeMap::new();

        if let Some(email) = trimmed_optional(&row.email) {
            for contact in storage::contacts::find_active_contacts_by_email(&self.db.conn, &email)?
            {
                matches.insert(contact.id.clone(), contact);
            }
        }

        if let Some(phone) = trimmed_optional(&row.phone) {
            for contact in storage::contacts::find_active_contacts_by_phone(&self.db.conn, &phone)?
            {
                matches.insert(contact.id.clone(), contact);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_values().next()),
            _ => Err(CrmError::InvalidInput(format!(
                "duplicate auto-merge skipped because multiple contacts match: {}",
                matches.keys().cloned().collect::<Vec<_>>().join(", ")
            ))),
        }
    }

    fn merge_contact_import_row(&mut self, contact_id: &str, row: &ContactCsvRow) -> CrmResult<()> {
        let existing = self.get_contact(contact_id)?;
        let first_name = fill_blank_string(&existing.first_name, Some(&row.first_name));
        let last_name = fill_blank_string(&existing.last_name, row.last_name.as_ref());
        let org_name = fill_blank_string(&existing.org_name, row.org_name.as_ref());
        let email = fill_blank_string(&existing.email, row.email.as_ref());
        let phone = fill_blank_string(&existing.phone, row.phone.as_ref());
        let address = fill_blank_string(&existing.address, row.address.as_ref());
        let city = fill_blank_string(&existing.city, row.city.as_ref());
        let country = fill_blank_string(&existing.country, row.country.as_ref());
        let notes = fill_blank_string(&existing.notes, row.notes.as_ref());

        if first_name.is_none()
            && last_name.is_none()
            && org_name.is_none()
            && email.is_none()
            && phone.is_none()
            && address.is_none()
            && city.is_none()
            && country.is_none()
            && notes.is_none()
        {
            return Ok(());
        }

        self.update_contact(
            contact_id, None, first_name, last_name, org_name, email, phone, address, city,
            country, notes,
        )?;
        Ok(())
    }

    pub fn preflight_contacts_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_contact_rows(rows)
    }

    fn preflight_contact_rows(
        &self,
        rows: Vec<(usize, ContactCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            if let Some(email) = trimmed_optional(&row.email) {
                for contact in
                    storage::contacts::find_active_contacts_by_email(&self.db.conn, &email)?
                {
                    warnings.push(import_duplicate_warning(
                        "contacts",
                        *row_number,
                        "email",
                        &email,
                        "contact",
                        &contact.id,
                        &contact_display_label(&contact),
                        format!("Email '{}' matches existing contact", email),
                    ));
                }
            }

            if let Some(phone) = trimmed_optional(&row.phone) {
                for contact in
                    storage::contacts::find_active_contacts_by_phone(&self.db.conn, &phone)?
                {
                    warnings.push(import_duplicate_warning(
                        "contacts",
                        *row_number,
                        "phone",
                        &phone,
                        "contact",
                        &contact.id,
                        &contact_display_label(&contact),
                        format!("Phone '{}' matches existing contact", phone),
                    ));
                }
            }
        }

        Ok(import_preflight_report("contacts", rows.len(), warnings))
    }

    pub fn export_contacts_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_contact_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_contacts_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_contacts_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_contact_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_contact_rows(&self) -> CrmResult<Vec<ContactCsvRow>> {
        let params = ContactListParams {
            page: 1,
            per_page: 100_000,
            sort_by: "first_name".to_string(),
            sort_dir: "asc".to_string(),
            filter_type: None,
            search_query: None,
            custom_field_def_id: None,
            custom_field_query: None,
        };
        let result = storage::contacts::list_contacts(&self.db.conn, &params)?;
        Ok(result
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
            .collect())
    }

    pub fn import_deals_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_row_numbers(file_content.as_slice())?;
        self.import_deal_rows(rows)
    }

    pub fn import_deals_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_deal_rows(rows)
    }

    pub fn import_deals_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_row_numbers(file_content.as_slice())?;
        self.import_deal_rows(rows)
    }

    pub fn import_deals_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_deal_rows(rows)
    }

    pub fn preview_deals_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_deals_json_import(file_content.as_slice())
    }

    fn import_deal_rows(&mut self, rows: Vec<(usize, DealCsvRow)>) -> CrmResult<ImportResult> {
        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (row_number, row) in rows {
            let value = row
                .value
                .as_deref()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0.0);
            match self.create_deal(
                row.title.clone(),
                Some(value),
                row.currency.clone(),
                row.stage.clone(),
                Some(0),
                row.expected_close.clone(),
                None,
                None,
                row.notes.clone(),
            ) {
                Ok(deal) => {
                    let _ = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("deal"),
                        Some(&deal.id),
                        None,
                        None,
                        &self.device_id,
                    );
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged: 0,
            skipped,
            errors,
        })
    }

    pub fn preflight_deals_csv_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_json_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_deal_rows(rows)
    }

    fn preflight_deal_rows(
        &self,
        rows: Vec<(usize, DealCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            let title = row.title.trim().to_string();
            for deal in storage::deals::find_active_deals_by_title(&self.db.conn, &title)? {
                warnings.push(import_duplicate_warning(
                    "deals",
                    *row_number,
                    "title",
                    &title,
                    "deal",
                    &deal.id,
                    &deal_display_label(&deal),
                    format!("Title '{}' matches existing deal", title),
                ));
            }
        }

        Ok(import_preflight_report("deals", rows.len(), warnings))
    }

    pub fn export_deals_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_deal_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_deals_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_deals_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_deal_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_deal_rows(&self) -> CrmResult<Vec<DealCsvRow>> {
        let all_deals = storage::deals::list_deals(&self.db.conn)?;
        Ok(all_deals
            .iter()
            .map(|d| DealCsvRow {
                title: d.title.clone(),
                value: Some(format!("{:.2}", d.value)),
                currency: Some(d.currency.clone()),
                stage: Some(d.stage.clone()),
                expected_close: d.expected_close.clone(),
                notes: Some(d.notes.clone()).filter(|s| !s.is_empty()),
            })
            .collect())
    }

    pub fn import_organizations_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_organizations_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_organizations_csv_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_row_numbers(file_content.as_slice())?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_organizations_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_organizations_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_organizations_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_organizations_json_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_row_numbers(file_content.as_slice())?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_organizations_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_organizations_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_organization_rows(rows, options)
    }

    pub fn preview_organizations_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_organizations_json_import(file_content.as_slice())
    }

    fn import_organization_rows(
        &mut self,
        rows: Vec<(usize, OrganizationCsvRow)>,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let mut created = 0u32;
        let mut merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (row_number, row) in rows {
            if options.merge_duplicates {
                match self.find_unique_organization_import_match(&row) {
                    Ok(Some(organization)) => {
                        match self.merge_organization_import_row(&organization.id, &row) {
                            Ok(()) => {
                                let _ = storage::audit::record_audit(
                                    &self.db.conn,
                                    ACTOR_IMPORT,
                                    None,
                                    "import_row_merge",
                                    Some("organization"),
                                    Some(&organization.id),
                                    None,
                                    None,
                                    &self.device_id,
                                );
                                merged += 1;
                                continue;
                            }
                            Err(e) => {
                                errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                        skipped += 1;
                        continue;
                    }
                }
            }

            match self.create_organization(
                row.name.clone(),
                row.email.clone(),
                row.phone.clone(),
                row.website.clone(),
                row.address_line1.clone(),
                row.address_line2.clone(),
                row.city.clone(),
                row.region.clone(),
                row.country.clone(),
                row.postal_code.clone(),
                row.description.clone(),
            ) {
                Ok(organization) => {
                    let _ = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("organization"),
                        Some(&organization.id),
                        None,
                        None,
                        &self.device_id,
                    );
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
        })
    }

    fn find_unique_organization_import_match(
        &self,
        row: &OrganizationCsvRow,
    ) -> CrmResult<Option<storage::organizations::Organization>> {
        let mut matches = BTreeMap::new();

        let name = row.name.trim().to_string();
        if !name.is_empty() {
            for organization in
                storage::organizations::find_active_organizations_by_name(&self.db.conn, &name)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        if let Some(email) = trimmed_optional(&row.email) {
            for organization in
                storage::organizations::find_active_organizations_by_email(&self.db.conn, &email)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        if let Some(phone) = trimmed_optional(&row.phone) {
            for organization in
                storage::organizations::find_active_organizations_by_phone(&self.db.conn, &phone)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_values().next()),
            _ => Err(CrmError::InvalidInput(format!(
                "duplicate auto-merge skipped because multiple organizations match: {}",
                matches.keys().cloned().collect::<Vec<_>>().join(", ")
            ))),
        }
    }

    fn merge_organization_import_row(
        &mut self,
        organization_id: &str,
        row: &OrganizationCsvRow,
    ) -> CrmResult<()> {
        let existing = self.get_organization(organization_id)?;
        let email = fill_blank_option(&existing.email, row.email.as_ref()).map(Some);
        let phone = fill_blank_option(&existing.phone, row.phone.as_ref()).map(Some);
        let website = fill_blank_option(&existing.website, row.website.as_ref()).map(Some);
        let address_line1 =
            fill_blank_option(&existing.address_line1, row.address_line1.as_ref()).map(Some);
        let address_line2 =
            fill_blank_option(&existing.address_line2, row.address_line2.as_ref()).map(Some);
        let city = fill_blank_option(&existing.city, row.city.as_ref()).map(Some);
        let region = fill_blank_option(&existing.region, row.region.as_ref()).map(Some);
        let country = fill_blank_option(&existing.country, row.country.as_ref()).map(Some);
        let postal_code =
            fill_blank_option(&existing.postal_code, row.postal_code.as_ref()).map(Some);
        let description =
            fill_blank_option(&existing.description, row.description.as_ref()).map(Some);

        if email.is_none()
            && phone.is_none()
            && website.is_none()
            && address_line1.is_none()
            && address_line2.is_none()
            && city.is_none()
            && region.is_none()
            && country.is_none()
            && postal_code.is_none()
            && description.is_none()
        {
            return Ok(());
        }

        self.update_organization(
            organization_id,
            None,
            email,
            phone,
            website,
            address_line1,
            address_line2,
            city,
            region,
            country,
            postal_code,
            description,
        )?;
        Ok(())
    }

    pub fn preflight_organizations_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_organization_rows(rows)
    }

    fn preflight_organization_rows(
        &self,
        rows: Vec<(usize, OrganizationCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            let name = row.name.trim().to_string();
            for organization in
                storage::organizations::find_active_organizations_by_name(&self.db.conn, &name)?
            {
                warnings.push(import_duplicate_warning(
                    "organizations",
                    *row_number,
                    "name",
                    &name,
                    "organization",
                    &organization.id,
                    &organization_display_label(&organization),
                    format!("Name '{}' matches existing organization", name),
                ));
            }

            if let Some(email) = trimmed_optional(&row.email) {
                for organization in storage::organizations::find_active_organizations_by_email(
                    &self.db.conn,
                    &email,
                )? {
                    warnings.push(import_duplicate_warning(
                        "organizations",
                        *row_number,
                        "email",
                        &email,
                        "organization",
                        &organization.id,
                        &organization_display_label(&organization),
                        format!("Email '{}' matches existing organization", email),
                    ));
                }
            }

            if let Some(phone) = trimmed_optional(&row.phone) {
                for organization in storage::organizations::find_active_organizations_by_phone(
                    &self.db.conn,
                    &phone,
                )? {
                    warnings.push(import_duplicate_warning(
                        "organizations",
                        *row_number,
                        "phone",
                        &phone,
                        "organization",
                        &organization.id,
                        &organization_display_label(&organization),
                        format!("Phone '{}' matches existing organization", phone),
                    ));
                }
            }
        }

        Ok(import_preflight_report(
            "organizations",
            rows.len(),
            warnings,
        ))
    }

    pub fn export_organizations_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_organization_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_organizations_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_organizations_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_organization_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_organization_rows(&self) -> CrmResult<Vec<OrganizationCsvRow>> {
        let organizations = storage::organizations::list_organizations(&self.db.conn)?;
        Ok(organizations
            .iter()
            .map(|organization| OrganizationCsvRow {
                name: organization.name.clone(),
                email: organization.email.clone(),
                phone: organization.phone.clone(),
                website: organization.website.clone(),
                address_line1: organization.address_line1.clone(),
                address_line2: organization.address_line2.clone(),
                city: organization.city.clone(),
                region: organization.region.clone(),
                country: organization.country.clone(),
                postal_code: organization.postal_code.clone(),
                description: organization.description.clone(),
            })
            .collect())
    }

    fn set_activity_completion(&mut self, id: &str, completed: bool) -> CrmResult<Activity> {
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = if completed {
            storage::activities::mark_complete(&tx, id)?
        } else {
            storage::activities::mark_incomplete(&tx, id)?
        };
        storage::sync::record_change(
            &tx,
            "activity",
            id,
            "completed",
            Some(if completed { "0" } else { "1" }),
            Some(if completed { "1" } else { "0" }),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            if completed {
                "mark_complete"
            } else {
                "mark_incomplete"
            },
            Some("activity"),
            Some(id),
            Some(&before),
            Some(&activity),
            &device_id,
        )?;
        tx.commit()?;
        Ok(activity)
    }
}

fn write_json_export<T>(file_path: &str, rows: &[T]) -> CrmResult<()>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(rows)?;
    fs::write(file_path, format!("{json}\n"))?;
    Ok(())
}

fn load_or_create_device_id(db: &storage::Database) -> CrmResult<String> {
    match storage::settings::get_setting(db.connection(), "device_id")? {
        Some(setting) if !setting.value.is_empty() => Ok(setting.value),
        _ => {
            let new_id = new_uuid();
            storage::settings::set_setting(db.connection(), "device_id", &new_id)?;
            Ok(new_id)
        }
    }
}

// Audit records intentionally keep before/after context explicit at call sites.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_audit_json<TBefore, TAfter>(
    conn: &Connection,
    actor_type: &str,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    before: Option<TBefore>,
    after: Option<TAfter>,
    device_id: &str,
) -> InternalCrmResult<AuditLogEntry>
where
    TBefore: Serialize,
    TAfter: Serialize,
{
    let before_json = before
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(CrmError::from)?;
    let after_json = after
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(CrmError::from)?;

    storage::audit::record_audit(
        conn,
        actor_type,
        None,
        action,
        entity_type,
        entity_id,
        before_json.as_deref(),
        after_json.as_deref(),
        device_id,
    )
}

fn parse_bool(value: Option<&str>) -> bool {
    matches!(value, Some("true") | Some("1"))
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

fn normalize_optional_update_string(value: Option<Option<String>>) -> Option<Option<String>> {
    value.map(normalize_optional_string)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_activity_in_transaction(
    conn: &Connection,
    device_id: &str,
    activity_type: &str,
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
    contact_id: Option<&str>,
    deal_id: Option<&str>,
) -> CrmResult<Activity> {
    activity_engine::validate_activity_for_create(title, activity_type)?;
    if let Some(contact_id) = contact_id {
        storage::contacts::get_contact(conn, contact_id)?;
    }
    if let Some(deal_id) = deal_id {
        storage::deals::get_deal(conn, deal_id)?;
    }

    let activity = storage::activities::create_activity(
        conn,
        activity_type,
        title,
        description.unwrap_or(""),
        due_date,
        contact_id,
        deal_id,
        device_id,
    )?;
    create_activity_links_for_legacy_mirrors(conn, &activity, device_id)?;
    storage::sync::record_change(
        conn,
        "activity",
        &activity.id,
        "__create__",
        None,
        Some(&activity.id),
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "create",
        Some("activity"),
        Some(&activity.id),
        None::<&()>,
        Some(&activity),
        device_id,
    )?;
    Ok(activity)
}

fn trimmed_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_string)
}

fn fill_blank_string(existing: &str, incoming: Option<&String>) -> Option<String> {
    if !existing.trim().is_empty() {
        return None;
    }

    incoming
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn fill_blank_option(existing: &Option<String>, incoming: Option<&String>) -> Option<String> {
    if existing
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return None;
    }

    incoming
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn import_preflight_report(
    entity_type: &str,
    total_rows: usize,
    warnings: Vec<ImportDuplicateWarning>,
) -> ImportPreflightReport {
    ImportPreflightReport {
        entity_type: entity_type.to_string(),
        total_rows: total_rows as u32,
        duplicate_warning_count: warnings.len() as u32,
        warnings,
    }
}

#[allow(clippy::too_many_arguments)]
fn import_duplicate_warning(
    entity_type: &str,
    row_number: usize,
    match_type: &str,
    csv_value: &str,
    existing_entity_type: &str,
    existing_entity_id: &str,
    existing_display_label: &str,
    reason: String,
) -> ImportDuplicateWarning {
    ImportDuplicateWarning {
        entity_type: entity_type.to_string(),
        row_number: row_number as u32,
        match_type: match_type.to_string(),
        csv_value: csv_value.to_string(),
        existing_entity_type: existing_entity_type.to_string(),
        existing_entity_id: existing_entity_id.to_string(),
        existing_display_label: existing_display_label.to_string(),
        reason,
    }
}

fn contact_display_label(contact: &storage::contacts::Contact) -> String {
    let name = [contact.first_name.as_str(), contact.last_name.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !name.trim().is_empty() {
        name
    } else if !contact.org_name.trim().is_empty() {
        contact.org_name.clone()
    } else if !contact.email.trim().is_empty() {
        contact.email.clone()
    } else if !contact.phone.trim().is_empty() {
        contact.phone.clone()
    } else {
        contact.id.clone()
    }
}

fn organization_display_label(organization: &storage::organizations::Organization) -> String {
    if organization.name.trim().is_empty() {
        organization.id.clone()
    } else {
        organization.name.clone()
    }
}

fn deal_display_label(deal: &storage::deals::Deal) -> String {
    if deal.title.trim().is_empty() {
        deal.id.clone()
    } else {
        deal.title.clone()
    }
}

fn sync_status_from_settings(
    sync_enabled: &str,
    sync_url: &str,
    last_sync_at: Option<String>,
    pending_changes: u32,
) -> CrmResult<SyncStatus> {
    if !parse_bool(Some(sync_enabled)) {
        return Ok(SyncStatus {
            state: "idle".to_string(),
            last_sync_at,
            error_message: None,
            pending_changes,
        });
    }

    if sync_url.trim().is_empty() {
        return Ok(SyncStatus {
            state: "error".to_string(),
            last_sync_at,
            error_message: Some("Sync URL is not configured.".to_string()),
            pending_changes,
        });
    }

    Ok(SyncStatus {
        state: "idle".to_string(),
        last_sync_at,
        error_message: None,
        pending_changes,
    })
}
