use std::fs;
use std::io::BufWriter;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::audit::{ACTOR_DESKTOP_APP, ACTOR_IMPORT};
use crate::crm_engine::{activities as activity_engine, deals as deal_engine, CrmEngine};
use crate::result::CrmResult;
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
        parse_contacts_csv, parse_deals_csv, write_contacts_csv, write_deals_csv, ContactCsvRow,
        DealCsvRow,
    },
    datetime::now_iso8601,
    errors::{CrmError, CrmResult as InternalCrmResult},
    uuid::new_uuid,
};

mod audit;
mod backup;
mod contacts;
mod migration_readiness;
mod organizations;
mod proposed_actions;
mod settings;

pub use backup::{LocalBackup, LocalBackupMetadata, LocalBackupValidation, LocalRestoreResult};
pub use migration_readiness::NormalizationMigrationPreflight;

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
    pub skipped: u32,
    pub errors: Vec<String>,
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

    pub fn create_deal(
        &mut self,
        title: String,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<String>,
        contact_id: Option<String>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
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
            notes.as_deref().unwrap_or(""),
            &device_id,
        )?;
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

    pub fn update_deal(
        &mut self,
        id: &str,
        title: Option<String>,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<String>,
        contact_id: Option<String>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
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
            Some(expected_close.as_deref()),
            Some(contact_id.as_deref()),
            notes.as_deref(),
        )?;
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
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = storage::activities::create_activity(
            &tx,
            &activity_type,
            &title,
            description.as_deref().unwrap_or(""),
            due_date.as_deref(),
            contact_id.as_deref(),
            deal_id.as_deref(),
            &device_id,
        )?;
        storage::sync::record_change(
            &tx,
            "activity",
            &activity.id,
            "__create__",
            None,
            Some(&activity.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("activity"),
            Some(&activity.id),
            None::<&()>,
            Some(&activity),
            &device_id,
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

    pub fn update_activity(
        &mut self,
        id: &str,
        activity_type: Option<String>,
        title: Option<String>,
        description: Option<String>,
        due_date: Option<String>,
        completed: Option<bool>,
        contact_id: Option<String>,
        deal_id: Option<String>,
    ) -> CrmResult<Activity> {
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = storage::activities::update_activity(
            &tx,
            id,
            activity_type.as_deref(),
            title.as_deref(),
            description.as_deref(),
            Some(due_date.as_deref()),
            completed,
            Some(contact_id.as_deref()),
            Some(deal_id.as_deref()),
        )?;
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
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv(file_content.as_slice())?;
        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (i, row) in rows.iter().enumerate() {
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
                    errors.push(format!("Row {}: {} ({})", i + 2, e, row.first_name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            skipped,
            errors,
        })
    }

    pub fn export_contacts_csv(&self, file_path: &str) -> CrmResult<u32> {
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
        let file = fs::File::create(file_path)?;
        write_contacts_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn import_deals_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv(file_content.as_slice())?;
        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for (i, row) in rows.iter().enumerate() {
            let value = row
                .value
                .as_deref()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            match self.create_deal(
                row.title.clone(),
                Some(value),
                row.currency.clone(),
                row.stage.clone(),
                Some(0),
                row.expected_close.clone(),
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
                    errors.push(format!("Row {}: {} ({})", i + 2, e, row.title));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            skipped,
            errors,
        })
    }

    pub fn export_deals_csv(&self, file_path: &str) -> CrmResult<u32> {
        let all_deals = storage::deals::list_deals(&self.db.conn)?;
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
        let file = fs::File::create(file_path)?;
        write_deals_csv(BufWriter::new(file), &rows)?;
        Ok(count)
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
