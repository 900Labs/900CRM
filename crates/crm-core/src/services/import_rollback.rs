use serde::{Deserialize, Serialize};

use crate::{
    storage::{contacts::Contact, deals::Deal, organizations::Organization},
    utils::{errors::CrmError, uuid::new_uuid},
};

use super::{CrmCore, CrmResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackPlan {
    pub token: String,
    pub actions: Vec<ImportRollbackAction>,
}

impl ImportRollbackPlan {
    pub(crate) fn from_actions(actions: Vec<ImportRollbackAction>) -> Option<Self> {
        if actions.is_empty() {
            return None;
        }

        Some(Self {
            token: new_uuid(),
            actions,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum ImportRollbackAction {
    Contact {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<ContactImportRollbackSnapshot>,
        post_import: ContactImportRollbackSnapshot,
    },
    Deal {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<DealImportRollbackSnapshot>,
        post_import: DealImportRollbackSnapshot,
    },
    Organization {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<OrganizationImportRollbackSnapshot>,
        post_import: OrganizationImportRollbackSnapshot,
    },
}

impl ImportRollbackAction {
    pub(crate) fn created_contact(row_number: usize, contact: &Contact) -> Self {
        Self::Contact {
            row_number: row_number as u32,
            entity_id: contact.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: ContactImportRollbackSnapshot::from(contact),
        }
    }

    pub(crate) fn merged_contact(
        row_number: usize,
        before_import: &Contact,
        post_import: &Contact,
    ) -> Option<Self> {
        let changed_fields = changed_contact_import_fields(before_import, post_import);
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Contact {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(ContactImportRollbackSnapshot::from(before_import)),
            post_import: ContactImportRollbackSnapshot::from(post_import),
        })
    }

    pub(crate) fn created_deal(row_number: usize, deal: &Deal) -> Self {
        Self::Deal {
            row_number: row_number as u32,
            entity_id: deal.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: DealImportRollbackSnapshot::from(deal),
        }
    }

    pub(crate) fn merged_deal(
        row_number: usize,
        before_import: &Deal,
        post_import: &Deal,
    ) -> Option<Self> {
        let changed_fields = changed_deal_import_fields(before_import, post_import);
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Deal {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(DealImportRollbackSnapshot::from(before_import)),
            post_import: DealImportRollbackSnapshot::from(post_import),
        })
    }

    pub(crate) fn created_organization(row_number: usize, organization: &Organization) -> Self {
        Self::Organization {
            row_number: row_number as u32,
            entity_id: organization.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: OrganizationImportRollbackSnapshot::from(organization),
        }
    }

    pub(crate) fn merged_organization(
        row_number: usize,
        before_import: &Organization,
        post_import: &Organization,
    ) -> Option<Self> {
        let changed_fields = changed_organization_import_fields(before_import, post_import);
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Organization {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(OrganizationImportRollbackSnapshot::from(before_import)),
            post_import: OrganizationImportRollbackSnapshot::from(post_import),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportRollbackOperation {
    Created,
    Merged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactImportRollbackSnapshot {
    pub contact_type: String,
    pub first_name: String,
    pub last_name: String,
    pub org_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub country: String,
    pub org_id: Option<String>,
    pub organization_id: Option<String>,
    pub notes: String,
    pub updated_at: String,
}

impl From<&Contact> for ContactImportRollbackSnapshot {
    fn from(contact: &Contact) -> Self {
        Self {
            contact_type: contact.contact_type.clone(),
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            org_name: contact.org_name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            address: contact.address.clone(),
            city: contact.city.clone(),
            country: contact.country.clone(),
            org_id: contact.org_id.clone(),
            organization_id: contact.organization_id.clone(),
            notes: contact.notes.clone(),
            updated_at: contact.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DealImportRollbackSnapshot {
    pub title: String,
    pub value: f64,
    pub currency: String,
    pub stage: String,
    pub probability: i32,
    pub expected_close: Option<String>,
    pub contact_id: Option<String>,
    pub organization_id: Option<String>,
    pub notes: String,
    pub updated_at: String,
}

impl From<&Deal> for DealImportRollbackSnapshot {
    fn from(deal: &Deal) -> Self {
        Self {
            title: deal.title.clone(),
            value: deal.value,
            currency: deal.currency.clone(),
            stage: deal.stage.clone(),
            probability: deal.probability,
            expected_close: deal.expected_close.clone(),
            contact_id: deal.contact_id.clone(),
            organization_id: deal.organization_id.clone(),
            notes: deal.notes.clone(),
            updated_at: deal.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationImportRollbackSnapshot {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub source: Option<String>,
    pub description: Option<String>,
    pub updated_at: String,
}

impl From<&Organization> for OrganizationImportRollbackSnapshot {
    fn from(organization: &Organization) -> Self {
        Self {
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
            source: organization.source.clone(),
            description: organization.description.clone(),
            updated_at: organization.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackResult {
    pub token: String,
    pub rolled_back: u32,
    pub skipped: u32,
    pub errors: Vec<ImportRollbackRowError>,
}

impl ImportRollbackResult {
    fn new(token: String) -> Self {
        Self {
            token,
            rolled_back: 0,
            skipped: 0,
            errors: Vec::new(),
        }
    }

    fn record_rolled_back(&mut self) {
        self.rolled_back += 1;
    }

    fn record_skipped(
        &mut self,
        entity_type: &str,
        entity_id: &str,
        row_number: u32,
        code: &str,
        message: String,
    ) {
        self.skipped += 1;
        self.errors.push(ImportRollbackRowError {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            row_number,
            code: code.to_string(),
            message,
        });
    }

    fn record_error(
        &mut self,
        entity_type: &str,
        entity_id: &str,
        row_number: u32,
        code: &str,
        message: String,
    ) {
        self.errors.push(ImportRollbackRowError {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            row_number,
            code: code.to_string(),
            message,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackRowError {
    pub entity_type: String,
    pub entity_id: String,
    pub row_number: u32,
    pub code: String,
    pub message: String,
}

impl CrmCore {
    pub fn rollback_completed_import(
        &mut self,
        plan: &ImportRollbackPlan,
    ) -> CrmResult<ImportRollbackResult> {
        let mut result = ImportRollbackResult::new(plan.token.clone());

        for action in &plan.actions {
            match action {
                ImportRollbackAction::Contact {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_contact(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => self.rollback_merged_contact(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
                ImportRollbackAction::Deal {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => {
                        self.rollback_created_deal(*row_number, entity_id, post_import, &mut result)
                    }
                    ImportRollbackOperation::Merged => self.rollback_merged_deal(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
                ImportRollbackAction::Organization {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_organization(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => self.rollback_merged_organization(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
            }
        }

        Ok(result)
    }

    fn rollback_created_contact(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &ContactImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_contact_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        if ContactImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "contact", entity_id, row_number);
            return;
        }

        match self.delete_contact(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("contact", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "contact",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_merged_contact(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&ContactImportRollbackSnapshot>,
        post_import: &ContactImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "contact", entity_id, row_number);
            return;
        };
        let Some(current) = current_contact_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        if ContactImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "contact", entity_id, row_number);
            return;
        }

        match self.update_contact(
            entity_id,
            string_field_update(changed_fields, "contact_type", &before_import.contact_type),
            string_field_update(changed_fields, "first_name", &before_import.first_name),
            string_field_update(changed_fields, "last_name", &before_import.last_name),
            string_field_update(changed_fields, "org_name", &before_import.org_name),
            string_field_update(changed_fields, "email", &before_import.email),
            string_field_update(changed_fields, "phone", &before_import.phone),
            string_field_update(changed_fields, "address", &before_import.address),
            string_field_update(changed_fields, "city", &before_import.city),
            string_field_update(changed_fields, "country", &before_import.country),
            string_field_update(changed_fields, "notes", &before_import.notes),
        ) {
            Ok(_) => result.record_rolled_back(),
            Err(err) => result.record_error(
                "contact",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_deal(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &DealImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_deal_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        if DealImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "deal", entity_id, row_number);
            return;
        }

        match self.delete_deal(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("deal", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "deal",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_merged_deal(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&DealImportRollbackSnapshot>,
        post_import: &DealImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "deal", entity_id, row_number);
            return;
        };
        let Some(current) = current_deal_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        if DealImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "deal", entity_id, row_number);
            return;
        }

        match self.update_deal(
            entity_id,
            string_field_update(changed_fields, "title", &before_import.title),
            f64_field_update(changed_fields, "value", before_import.value),
            string_field_update(changed_fields, "currency", &before_import.currency),
            string_field_update(changed_fields, "stage", &before_import.stage),
            i32_field_update(changed_fields, "probability", before_import.probability),
            option_string_field_update(
                changed_fields,
                "expected_close",
                &before_import.expected_close,
            ),
            option_string_field_update(changed_fields, "contact_id", &before_import.contact_id),
            option_string_field_update(
                changed_fields,
                "organization_id",
                &before_import.organization_id,
            ),
            string_field_update(changed_fields, "notes", &before_import.notes),
        ) {
            Ok(_) => result.record_rolled_back(),
            Err(err) => result.record_error(
                "deal",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_organization(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &OrganizationImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_organization_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        if OrganizationImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "organization", entity_id, row_number);
            return;
        }

        match self.delete_organization(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("organization", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "organization",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_merged_organization(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&OrganizationImportRollbackSnapshot>,
        post_import: &OrganizationImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "organization", entity_id, row_number);
            return;
        };
        let Some(current) = current_organization_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        if OrganizationImportRollbackSnapshot::from(&current) != *post_import {
            record_conflict(result, "organization", entity_id, row_number);
            return;
        }

        match self.update_organization(
            entity_id,
            string_field_update(changed_fields, "name", &before_import.name),
            option_string_field_update(changed_fields, "email", &before_import.email),
            option_string_field_update(changed_fields, "phone", &before_import.phone),
            option_string_field_update(changed_fields, "website", &before_import.website),
            option_string_field_update(
                changed_fields,
                "address_line1",
                &before_import.address_line1,
            ),
            option_string_field_update(
                changed_fields,
                "address_line2",
                &before_import.address_line2,
            ),
            option_string_field_update(changed_fields, "city", &before_import.city),
            option_string_field_update(changed_fields, "region", &before_import.region),
            option_string_field_update(changed_fields, "country", &before_import.country),
            option_string_field_update(changed_fields, "postal_code", &before_import.postal_code),
            option_string_field_update(changed_fields, "description", &before_import.description),
        ) {
            Ok(_) => result.record_rolled_back(),
            Err(err) => result.record_error(
                "organization",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }
}

fn current_contact_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Contact> {
    match core.get_contact(entity_id) {
        Ok(contact) => Some(contact),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("contact", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "contact",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_deal_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Deal> {
    match core.get_deal(entity_id) {
        Ok(deal) => Some(deal),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("deal", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "deal",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_organization_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Organization> {
    match core.get_organization(entity_id) {
        Ok(organization) => Some(organization),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("organization", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "organization",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn record_conflict(
    result: &mut ImportRollbackResult,
    entity_type: &str,
    entity_id: &str,
    row_number: u32,
) {
    result.record_skipped(
        entity_type,
        entity_id,
        row_number,
        "conflict",
        format!(
            "{} '{}' changed after import; row rollback skipped",
            entity_type, entity_id
        ),
    );
}

fn record_invalid_plan(
    result: &mut ImportRollbackResult,
    entity_type: &str,
    entity_id: &str,
    row_number: u32,
) {
    result.record_error(
        entity_type,
        entity_id,
        row_number,
        "invalid_plan",
        "rollback plan is missing before-import state for a merge action".to_string(),
    );
}

fn string_field_update(changed_fields: &[String], field: &str, value: &str) -> Option<String> {
    field_changed(changed_fields, field).then(|| value.to_string())
}

fn option_string_field_update(
    changed_fields: &[String],
    field: &str,
    value: &Option<String>,
) -> Option<Option<String>> {
    field_changed(changed_fields, field).then(|| value.clone())
}

fn f64_field_update(changed_fields: &[String], field: &str, value: f64) -> Option<f64> {
    field_changed(changed_fields, field).then_some(value)
}

fn i32_field_update(changed_fields: &[String], field: &str, value: i32) -> Option<i32> {
    field_changed(changed_fields, field).then_some(value)
}

fn field_changed(changed_fields: &[String], field: &str) -> bool {
    changed_fields.iter().any(|changed| changed == field)
}

fn changed_contact_import_fields(before: &Contact, after: &Contact) -> Vec<String> {
    let mut fields = Vec::new();
    push_changed_string(
        &mut fields,
        "first_name",
        &before.first_name,
        &after.first_name,
    );
    push_changed_string(
        &mut fields,
        "last_name",
        &before.last_name,
        &after.last_name,
    );
    push_changed_string(&mut fields, "org_name", &before.org_name, &after.org_name);
    push_changed_string(&mut fields, "email", &before.email, &after.email);
    push_changed_string(&mut fields, "phone", &before.phone, &after.phone);
    push_changed_string(&mut fields, "address", &before.address, &after.address);
    push_changed_string(&mut fields, "city", &before.city, &after.city);
    push_changed_string(&mut fields, "country", &before.country, &after.country);
    push_changed_string(&mut fields, "notes", &before.notes, &after.notes);
    fields
}

fn changed_deal_import_fields(before: &Deal, after: &Deal) -> Vec<String> {
    let mut fields = Vec::new();
    if before.value != after.value {
        fields.push("value".to_string());
    }
    push_changed_option_string(
        &mut fields,
        "expected_close",
        &before.expected_close,
        &after.expected_close,
    );
    push_changed_string(&mut fields, "notes", &before.notes, &after.notes);
    fields
}

fn changed_organization_import_fields(before: &Organization, after: &Organization) -> Vec<String> {
    let mut fields = Vec::new();
    push_changed_option_string(&mut fields, "email", &before.email, &after.email);
    push_changed_option_string(&mut fields, "phone", &before.phone, &after.phone);
    push_changed_option_string(&mut fields, "website", &before.website, &after.website);
    push_changed_option_string(
        &mut fields,
        "address_line1",
        &before.address_line1,
        &after.address_line1,
    );
    push_changed_option_string(
        &mut fields,
        "address_line2",
        &before.address_line2,
        &after.address_line2,
    );
    push_changed_option_string(&mut fields, "city", &before.city, &after.city);
    push_changed_option_string(&mut fields, "region", &before.region, &after.region);
    push_changed_option_string(&mut fields, "country", &before.country, &after.country);
    push_changed_option_string(
        &mut fields,
        "postal_code",
        &before.postal_code,
        &after.postal_code,
    );
    push_changed_option_string(
        &mut fields,
        "description",
        &before.description,
        &after.description,
    );
    fields
}

fn push_changed_string(fields: &mut Vec<String>, field: &str, before: &str, after: &str) {
    if before != after {
        fields.push(field.to_string());
    }
}

fn push_changed_option_string(
    fields: &mut Vec<String>,
    field: &str,
    before: &Option<String>,
    after: &Option<String>,
) {
    if before != after {
        fields.push(field.to_string());
    }
}
