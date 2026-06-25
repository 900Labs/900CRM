//! Flat import/export row helpers for contacts, deals, organizations, activities, and notes.
//!
//! This module provides utilities for reading and writing CSV files and parsing
//! JSON arrays used in the 900CRM import/export feature. It wraps the [`csv`]
//! crate for CSV operations and reuses the same flat row structs for JSON.
//!
//! # Contact CSV Format
//!
//! | Column       | Required | Notes |
//! |--------------|----------|-------|
//! | `first_name` | yes      | |
//! | `last_name`  | no       | |
//! | `org_name`   | no       | |
//! | `email`      | no       | |
//! | `phone`      | no       | |
//! | `address`    | no       | |
//! | `city`       | no       | |
//! | `country`    | no       | |
//! | `notes`      | no       | |
//!
//! # Deal CSV Format
//!
//! | Column           | Required | Notes |
//! |------------------|----------|-------|
//! | `title`          | yes      | |
//! | `value`          | no       | Decimal number |
//! | `currency`       | no       | ISO 4217 code (default: USD) |
//! | `stage`          | no       | Pipeline stage name |
//! | `expected_close` | no       | YYYY-MM-DD or ISO 8601 |
//! | `notes`          | no       | |

//! # Activity CSV Format
//!
//! | Column          | Required | Notes |
//! |-----------------|----------|-------|
//! | `activity_type` | yes      | Freeform activity type |
//! | `title`         | yes      | |
//! | `description`   | no       | |
//! | `due_date`      | no       | YYYY-MM-DD or ISO 8601 |
//! | `completed`     | no       | `true`/`false` |
//! | `contact_id`    | no       | Existing local contact UUID |
//! | `deal_id`       | no       | Existing local deal UUID |
//!
//! # Note CSV Format
//!
//! | Column        | Required | Notes |
//! |---------------|----------|-------|
//! | `entity_type` | yes      | `contact`, `organization`, `deal`, or `activity` |
//! | `entity_id`   | yes      | Existing active local entity UUID |
//! | `content`     | yes      | Note body |
//!
//! # Organization CSV Format
//!
//! | Column          | Required | Notes |
//! |-----------------|----------|-------|
//! | `name`          | yes      | |
//! | `email`         | no       | |
//! | `phone`         | no       | |
//! | `website`       | no       | |
//! | `address_line1` | no       | |
//! | `address_line2` | no       | |
//! | `city`          | no       | |
//! | `region`        | no       | |
//! | `country`       | no       | |
//! | `postal_code`   | no       | |
//! | `description`   | no       | |

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::errors::{CrmError, CrmResult};

pub const CUSTOM_FIELD_PREFIX: &str = "custom:";

/// Frontend-provided CSV mapping: source CSV header -> target CRM field.
///
/// `None` means the source header is intentionally skipped. Source headers not
/// present in the map are also skipped.
pub type ImportColumnMapping = HashMap<String, Option<String>>;

/// Read-only preview generated from a JSON import file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonImportPreview {
    pub total_rows: usize,
    pub headers: Vec<String>,
    pub rows: Vec<JsonImportPreviewRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonImportPreviewRow {
    pub row_number: usize,
    pub values: HashMap<String, String>,
}

const CONTACT_IMPORT_TARGET_FIELDS: &[&str] = &[
    "first_name",
    "last_name",
    "org_name",
    "email",
    "phone",
    "address",
    "city",
    "country",
    "notes",
];

const ORGANIZATION_IMPORT_TARGET_FIELDS: &[&str] = &[
    "name",
    "email",
    "phone",
    "website",
    "address_line1",
    "address_line2",
    "city",
    "region",
    "country",
    "postal_code",
    "description",
];

const DEAL_IMPORT_TARGET_FIELDS: &[&str] = &[
    "title",
    "value",
    "currency",
    "stage",
    "expected_close",
    "notes",
];

const ACTIVITY_IMPORT_TARGET_FIELDS: &[&str] = &[
    "activity_type",
    "title",
    "description",
    "due_date",
    "completed",
    "contact_id",
    "deal_id",
];

const NOTE_IMPORT_TARGET_FIELDS: &[&str] = &["entity_type", "entity_id", "content"];

// ─────────────────────────────────────────────────────────────────────────────
// Contact CSV record
// ─────────────────────────────────────────────────────────────────────────────

/// A flat CSV record representing one contact row.
///
/// All fields are `Option<String>` except `first_name` (required). Missing
/// columns in the CSV are deserialized as `None` via `serde`'s default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCsvRow {
    /// Contact's given name. Required.
    pub first_name: String,

    /// Contact's family name.
    #[serde(default)]
    pub last_name: Option<String>,

    /// The organization this contact belongs to.
    #[serde(default)]
    pub org_name: Option<String>,

    /// Primary email address.
    #[serde(default)]
    pub email: Option<String>,

    /// Primary phone number.
    #[serde(default)]
    pub phone: Option<String>,

    /// Street address.
    #[serde(default)]
    pub address: Option<String>,

    /// City / locality.
    #[serde(default)]
    pub city: Option<String>,

    /// Country.
    #[serde(default)]
    pub country: Option<String>,

    /// Freeform notes.
    #[serde(default)]
    pub notes: Option<String>,

    /// User-defined custom field values keyed by `custom:<field_name>`.
    #[serde(default, flatten)]
    pub custom_fields: BTreeMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Deal CSV record
// ─────────────────────────────────────────────────────────────────────────────

/// A flat CSV record representing one deal row.
///
/// The `value` field is stored as a string in CSV and parsed to `f64` on import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealCsvRow {
    /// Deal title. Required.
    pub title: String,

    /// Monetary value as a decimal string (e.g. `"12500.00"`).
    #[serde(default)]
    pub value: Option<String>,

    /// ISO 4217 currency code (e.g. `"USD"`, `"EUR"`). Defaults to `"USD"`.
    #[serde(default)]
    pub currency: Option<String>,

    /// Pipeline stage name (e.g. `"Lead"`, `"Proposal"`).
    #[serde(default)]
    pub stage: Option<String>,

    /// Expected close date in `YYYY-MM-DD` or ISO 8601 format.
    #[serde(default)]
    pub expected_close: Option<String>,

    /// Freeform notes about the deal.
    #[serde(default)]
    pub notes: Option<String>,

    /// User-defined custom field values keyed by `custom:<field_name>`.
    #[serde(default, flatten)]
    pub custom_fields: BTreeMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Activity CSV record
// ─────────────────────────────────────────────────────────────────────────────

/// A flat CSV record representing one activity row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCsvRow {
    /// Activity type. Required.
    pub activity_type: String,

    /// Activity title. Required.
    pub title: String,

    /// Longer description/body.
    #[serde(default)]
    pub description: Option<String>,

    /// Due date in `YYYY-MM-DD` or ISO 8601 format.
    #[serde(default)]
    pub due_date: Option<String>,

    /// Whether the imported activity should be marked complete after creation.
    #[serde(default)]
    pub completed: Option<bool>,

    /// Optional existing local contact UUID.
    #[serde(default)]
    pub contact_id: Option<String>,

    /// Optional existing local deal UUID.
    #[serde(default)]
    pub deal_id: Option<String>,

    /// User-defined custom field values keyed by `custom:<field_name>`.
    #[serde(default, flatten)]
    pub custom_fields: BTreeMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Organization CSV record
// ─────────────────────────────────────────────────────────────────────────────

/// A flat CSV record representing one organization row.
///
/// All fields are `Option<String>` except `name` (required). Missing columns in
/// the CSV are deserialized as `None` via `serde`'s default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCsvRow {
    /// Organization display name. Required.
    pub name: String,

    /// Primary email address.
    #[serde(default)]
    pub email: Option<String>,

    /// Primary phone number.
    #[serde(default)]
    pub phone: Option<String>,

    /// Website URL.
    #[serde(default)]
    pub website: Option<String>,

    /// Street address line 1.
    #[serde(default)]
    pub address_line1: Option<String>,

    /// Street address line 2.
    #[serde(default)]
    pub address_line2: Option<String>,

    /// City / locality.
    #[serde(default)]
    pub city: Option<String>,

    /// Region / state / province.
    #[serde(default)]
    pub region: Option<String>,

    /// Country.
    #[serde(default)]
    pub country: Option<String>,

    /// Postal or ZIP code.
    #[serde(default)]
    pub postal_code: Option<String>,

    /// Freeform organization description.
    #[serde(default)]
    pub description: Option<String>,

    /// User-defined custom field values keyed by `custom:<field_name>`.
    #[serde(default, flatten)]
    pub custom_fields: BTreeMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Note CSV record
// ─────────────────────────────────────────────────────────────────────────────

/// A flat CSV record representing one generic note row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteCsvRow {
    /// Parent entity type: `"contact"`, `"organization"`, `"deal"`, or `"activity"`.
    pub entity_type: String,

    /// Existing active local parent entity UUID.
    pub entity_id: String,

    /// Note content/body.
    pub content: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Import helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Parses CSV data from a byte reader into a `Vec<ContactCsvRow>`.
///
/// Requires a header row with at minimum a `first_name` column.
/// Rows where `first_name` is blank are skipped with a debug log.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if the CSV is malformed or required columns
/// are missing.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::csv::parse_contacts_csv;
///
/// let data = b"first_name,email\nAlice,alice@example.com\n";
/// let rows = parse_contacts_csv(&data[..]).unwrap();
/// assert_eq!(rows.len(), 1);
/// ```
pub fn parse_contacts_csv<R: Read>(reader: R) -> CrmResult<Vec<ContactCsvRow>> {
    Ok(parse_contacts_csv_with_row_numbers(reader)?
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Parses contact CSV data and preserves the original 1-based source row
/// number, including the header row offset.
pub fn parse_contacts_csv_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    require_csv_header(&headers, "first_name")?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = contact_row_from_record(&headers, &record);
        if row.first_name.trim().is_empty() {
            log::debug!("Skipping CSV row with blank first_name");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} contact rows from CSV", rows.len());
    Ok(rows)
}

/// Parses arbitrary-header contact CSV data with a frontend-provided mapping.
///
/// Rows where the mapped `first_name` is blank are skipped, matching the
/// standard contact CSV import behavior.
pub fn parse_contacts_csv_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    parse_contacts_csv_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_contacts_csv_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments = validate_import_mapping(
        &headers,
        mapping,
        CONTACT_IMPORT_TARGET_FIELDS,
        custom_targets,
    )?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = contact_row_from_mapped_record(&record, &assignments);
        if row.first_name.trim().is_empty() {
            log::debug!("Skipping CSV row with blank first_name");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped contact rows from CSV", rows.len());
    Ok(rows)
}

/// Parses CSV data from a byte reader into a `Vec<DealCsvRow>`.
///
/// Requires a header row with at minimum a `title` column.
/// Rows where `title` is blank are skipped.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if the CSV is malformed.
pub fn parse_deals_csv<R: Read>(reader: R) -> CrmResult<Vec<DealCsvRow>> {
    Ok(parse_deals_csv_with_row_numbers(reader)?
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Parses deal CSV data and preserves the original 1-based source row number,
/// including the header row offset.
pub fn parse_deals_csv_with_row_numbers<R: Read>(reader: R) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    require_csv_header(&headers, "title")?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = deal_row_from_record(&headers, &record);
        if row.title.trim().is_empty() {
            log::debug!("Skipping CSV row with blank title");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} deal rows from CSV", rows.len());
    Ok(rows)
}

/// Parses arbitrary-header deal CSV data with a frontend-provided mapping.
///
/// Rows where the mapped `title` is blank are skipped, matching the standard
/// deal CSV import behavior.
pub fn parse_deals_csv_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    parse_deals_csv_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_deals_csv_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments =
        validate_import_mapping(&headers, mapping, DEAL_IMPORT_TARGET_FIELDS, custom_targets)?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = deal_row_from_mapped_record(&record, &assignments);
        if row.title.trim().is_empty() {
            log::debug!("Skipping CSV row with blank title");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped deal rows from CSV", rows.len());
    Ok(rows)
}

/// Parses CSV data from a byte reader into a `Vec<ActivityCsvRow>`.
///
/// Requires a header row with at minimum `activity_type` and `title` columns.
/// Rows where either required value is blank are skipped.
pub fn parse_activities_csv<R: Read>(reader: R) -> CrmResult<Vec<ActivityCsvRow>> {
    Ok(parse_activities_csv_with_row_numbers(reader)?
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Parses activity CSV data and preserves the original 1-based source row
/// number, including the header row offset.
pub fn parse_activities_csv_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    require_csv_header(&headers, "activity_type")?;
    require_csv_header(&headers, "title")?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = activity_row_from_record(&headers, &record);
        if row.activity_type.trim().is_empty() || row.title.trim().is_empty() {
            log::debug!("Skipping CSV row with blank activity_type or title");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} activity rows from CSV", rows.len());
    Ok(rows)
}

/// Parses arbitrary-header activity CSV data with a frontend-provided mapping.
pub fn parse_activities_csv_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    parse_activities_csv_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_activities_csv_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments = validate_import_mapping(
        &headers,
        mapping,
        ACTIVITY_IMPORT_TARGET_FIELDS,
        custom_targets,
    )?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = activity_row_from_mapped_record(&record, &assignments);
        if row.activity_type.trim().is_empty() || row.title.trim().is_empty() {
            log::debug!("Skipping CSV row with blank activity_type or title");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped activity rows from CSV", rows.len());
    Ok(rows)
}

/// Parses CSV data from a byte reader into a `Vec<OrganizationCsvRow>`.
///
/// Requires a header row with at minimum a `name` column. Rows where `name` is
/// blank are skipped.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if the CSV is malformed.
pub fn parse_organizations_csv<R: Read>(reader: R) -> CrmResult<Vec<OrganizationCsvRow>> {
    Ok(parse_organizations_csv_with_row_numbers(reader)?
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Parses organization CSV data and preserves the original 1-based source row
/// number, including the header row offset.
pub fn parse_organizations_csv_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    require_csv_header(&headers, "name")?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = organization_row_from_record(&headers, &record);
        if row.name.trim().is_empty() {
            log::debug!("Skipping CSV row with blank name");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} organization rows from CSV", rows.len());
    Ok(rows)
}

/// Parses arbitrary-header organization CSV data with a frontend-provided
/// mapping.
///
/// Rows where the mapped `name` is blank are skipped, matching the standard
/// organization CSV import behavior.
pub fn parse_organizations_csv_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    parse_organizations_csv_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_organizations_csv_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments = validate_import_mapping(
        &headers,
        mapping,
        ORGANIZATION_IMPORT_TARGET_FIELDS,
        custom_targets,
    )?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        let row = organization_row_from_mapped_record(&record, &assignments);
        if row.name.trim().is_empty() {
            log::debug!("Skipping CSV row with blank name");
            continue;
        }
        rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped organization rows from CSV", rows.len());
    Ok(rows)
}

/// Parses generic note CSV data and preserves source row numbers.
pub fn parse_notes_csv_with_row_numbers<R: Read>(reader: R) -> CrmResult<Vec<(usize, NoteCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    require_csv_header(&headers, "entity_type")?;
    require_csv_header(&headers, "entity_id")?;
    require_csv_header(&headers, "content")?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        rows.push((row_number, note_row_from_record(&headers, &record)));
    }

    log::info!("Parsed {} note rows from CSV", rows.len());
    Ok(rows)
}

/// Parses arbitrary-header generic note CSV data with a frontend-provided mapping.
pub fn parse_notes_csv_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, NoteCsvRow)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments = validate_import_mapping(&headers, mapping, NOTE_IMPORT_TARGET_FIELDS, &[])?;

    let mut rows = Vec::new();
    for (index, result) in rdr.records().enumerate() {
        let row_number = index + 2;
        let record = result.map_err(|e| CrmError::Csv(e.to_string()))?;
        rows.push((
            row_number,
            note_row_from_mapped_record(&record, &assignments),
        ));
    }

    log::info!("Parsed {} mapped note rows from CSV", rows.len());
    Ok(rows)
}

/// Parses contact JSON data from a top-level array of flat row objects.
///
/// Row numbers are reported with the same data-row offset as CSV imports:
/// the first JSON array item is row 2.
pub fn parse_contacts_json_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = contact_row_from_json_object(object, &headers);
        if row.first_name.trim().is_empty() {
            log::debug!("Skipping JSON row with blank first_name");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} contact rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses contact JSON data with frontend-provided source-field mapping.
pub fn parse_contacts_json_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    parse_contacts_json_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_contacts_json_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, ContactCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let assignments = validate_import_mapping_sources(
        &headers,
        mapping,
        CONTACT_IMPORT_TARGET_FIELDS,
        custom_targets,
        "field",
        "JSON",
    )?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = contact_row_from_mapped_json_object(object, &headers, &assignments);
        if row.first_name.trim().is_empty() {
            log::debug!("Skipping JSON row with blank first_name");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped contact rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses deal JSON data from a top-level array of flat row objects.
///
/// Row numbers are reported with the same data-row offset as CSV imports:
/// the first JSON array item is row 2.
pub fn parse_deals_json_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = deal_row_from_json_object(object, &headers);
        if row.title.trim().is_empty() {
            log::debug!("Skipping JSON row with blank title");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} deal rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses deal JSON data with frontend-provided source-field mapping.
pub fn parse_deals_json_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    parse_deals_json_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_deals_json_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, DealCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let assignments = validate_import_mapping_sources(
        &headers,
        mapping,
        DEAL_IMPORT_TARGET_FIELDS,
        custom_targets,
        "field",
        "JSON",
    )?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = deal_row_from_mapped_json_object(object, &headers, &assignments);
        if row.title.trim().is_empty() {
            log::debug!("Skipping JSON row with blank title");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} mapped deal rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses activity JSON data from a top-level array of flat row objects.
///
/// Row numbers are reported with the same data-row offset as CSV imports:
/// the first JSON array item is row 2.
pub fn parse_activities_json_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = activity_row_from_json_object(object, &headers);
        if row.activity_type.trim().is_empty() || row.title.trim().is_empty() {
            log::debug!("Skipping JSON row with blank activity_type or title");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} activity rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses activity JSON data with frontend-provided source-field mapping.
pub fn parse_activities_json_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    parse_activities_json_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_activities_json_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, ActivityCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let assignments = validate_import_mapping_sources(
        &headers,
        mapping,
        ACTIVITY_IMPORT_TARGET_FIELDS,
        custom_targets,
        "field",
        "JSON",
    )?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = activity_row_from_mapped_json_object(object, &headers, &assignments);
        if row.activity_type.trim().is_empty() || row.title.trim().is_empty() {
            log::debug!("Skipping JSON row with blank activity_type or title");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!(
        "Parsed {} mapped activity rows from JSON",
        parsed_rows.len()
    );
    Ok(parsed_rows)
}

/// Parses organization JSON data from a top-level array of flat row objects.
///
/// Row numbers are reported with the same data-row offset as CSV imports:
/// the first JSON array item is row 2.
pub fn parse_organizations_json_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = organization_row_from_json_object(object, &headers);
        if row.name.trim().is_empty() {
            log::debug!("Skipping JSON row with blank name");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!("Parsed {} organization rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses organization JSON data with frontend-provided source-field mapping.
pub fn parse_organizations_json_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    parse_organizations_json_with_mapping_targets(reader, mapping, &[])
}

pub fn parse_organizations_json_with_mapping_targets<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
    custom_targets: &[String],
) -> CrmResult<Vec<(usize, OrganizationCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let assignments = validate_import_mapping_sources(
        &headers,
        mapping,
        ORGANIZATION_IMPORT_TARGET_FIELDS,
        custom_targets,
        "field",
        "JSON",
    )?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        let row = organization_row_from_mapped_json_object(object, &headers, &assignments);
        if row.name.trim().is_empty() {
            log::debug!("Skipping JSON row with blank name");
            continue;
        }
        parsed_rows.push((row_number, row));
    }

    log::info!(
        "Parsed {} mapped organization rows from JSON",
        parsed_rows.len()
    );
    Ok(parsed_rows)
}

/// Parses generic note JSON data from a top-level array of flat row objects.
///
/// Row numbers are reported with the same data-row offset as CSV imports:
/// the first JSON array item is row 2.
pub fn parse_notes_json_with_row_numbers<R: Read>(
    reader: R,
) -> CrmResult<Vec<(usize, NoteCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        parsed_rows.push((row_number, note_row_from_json_object(object, &headers)));
    }

    log::info!("Parsed {} note rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

/// Parses generic note JSON data with frontend-provided source-field mapping.
pub fn parse_notes_json_with_mapping<R: Read>(
    reader: R,
    mapping: &ImportColumnMapping,
) -> CrmResult<Vec<(usize, NoteCsvRow)>> {
    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let assignments = validate_import_mapping_sources(
        &headers,
        mapping,
        NOTE_IMPORT_TARGET_FIELDS,
        &[],
        "field",
        "JSON",
    )?;
    let mut parsed_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        parsed_rows.push((
            row_number,
            note_row_from_mapped_json_object(object, &headers, &assignments),
        ));
    }

    log::info!("Parsed {} mapped note rows from JSON", parsed_rows.len());
    Ok(parsed_rows)
}

pub fn preview_contacts_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    preview_json_import(reader)
}

pub fn preview_deals_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    preview_json_import(reader)
}

pub fn preview_activities_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    preview_json_import(reader)
}

pub fn preview_organizations_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    preview_json_import(reader)
}

pub fn preview_notes_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    preview_json_import(reader)
}

fn parse_json_array_rows<R: Read>(reader: R) -> CrmResult<Vec<Value>> {
    let json: Value = serde_json::from_reader(reader)?;
    match json {
        Value::Array(rows) => Ok(rows),
        _ => Err(CrmError::InvalidInput(
            "JSON import expects a top-level array of objects".to_string(),
        )),
    }
}

fn preview_json_import<R: Read>(reader: R) -> CrmResult<JsonImportPreview> {
    const MAX_PREVIEW_ROWS: usize = 5;

    let rows = parse_json_array_rows(reader)?;
    let headers = collect_json_source_fields(&rows)?;
    let mut preview_rows = Vec::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;

        if preview_rows.len() < MAX_PREVIEW_ROWS {
            let values = headers
                .iter()
                .map(|header| {
                    (
                        header.clone(),
                        json_preview_cell(object.get(header.as_str())),
                    )
                })
                .collect();
            preview_rows.push(JsonImportPreviewRow { row_number, values });
        }
    }

    Ok(JsonImportPreview {
        total_rows: rows.len(),
        headers,
        rows: preview_rows,
    })
}

fn json_preview_cell(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}

fn collect_json_source_fields(rows: &[Value]) -> CrmResult<Vec<String>> {
    let mut fields = BTreeSet::new();

    for (index, value) in rows.iter().enumerate() {
        let row_number = index + 2;
        let object = value.as_object().ok_or_else(|| {
            CrmError::InvalidInput(format!("JSON row {} must be an object", row_number))
        })?;
        fields.extend(object.keys().cloned());
    }

    Ok(fields.into_iter().collect())
}

fn validate_import_mapping(
    headers: &csv::StringRecord,
    mapping: &ImportColumnMapping,
    allowed_targets: &[&str],
    custom_targets: &[String],
) -> CrmResult<Vec<Option<String>>> {
    let headers = headers.iter().map(str::to_string).collect::<Vec<_>>();
    validate_import_mapping_sources(
        &headers,
        mapping,
        allowed_targets,
        custom_targets,
        "header",
        "CSV",
    )
}

fn validate_import_mapping_sources(
    headers: &[String],
    mapping: &ImportColumnMapping,
    allowed_targets: &[&str],
    custom_targets: &[String],
    source_label: &str,
    data_label: &str,
) -> CrmResult<Vec<Option<String>>> {
    let header_set: HashSet<&str> = headers.iter().map(String::as_str).collect();
    let mut allowed_target_set: HashSet<String> = allowed_targets
        .iter()
        .map(|target| (*target).to_string())
        .collect();
    allowed_target_set.extend(custom_targets.iter().cloned());
    let mut assigned_targets: HashMap<String, String> = HashMap::new();

    for (source_header, target) in mapping {
        if !header_set.contains(source_header.as_str()) {
            return Err(CrmError::InvalidInput(format!(
                "Mapped source {} '{}' is not present in the {}",
                source_label, source_header, data_label
            )));
        }

        if let Some(target) = target {
            let target = target.trim();
            if !allowed_target_set.contains(target) {
                return Err(CrmError::InvalidInput(format!(
                    "Unknown import target field '{}'",
                    target
                )));
            }

            if let Some(previous_source) =
                assigned_targets.insert(target.to_string(), source_header.clone())
            {
                return Err(CrmError::InvalidInput(format!(
                    "Import target field '{}' is mapped more than once ('{}' and '{}')",
                    target, previous_source, source_header
                )));
            }
        }
    }

    Ok(headers
        .iter()
        .map(|source_header| {
            mapping
                .get(source_header)
                .and_then(|target| target.as_ref())
                .map(|target| target.trim().to_string())
        })
        .collect())
}

fn require_csv_header(headers: &csv::StringRecord, required: &str) -> CrmResult<()> {
    if headers.iter().any(|header| header == required) {
        return Ok(());
    }

    Err(CrmError::Csv(format!(
        "CSV missing required header '{}'",
        required
    )))
}

fn contact_row_from_record(
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
) -> ContactCsvRow {
    let mut row = default_contact_row();

    for (index, header) in headers.iter().enumerate() {
        let value = record.get(index).unwrap_or_default().trim();
        assign_contact_value(&mut row, header.trim(), value);
    }

    row
}

fn contact_row_from_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
) -> ContactCsvRow {
    let mut row = default_contact_row();

    for header in headers {
        let value = json_preview_cell(object.get(header.as_str()));
        assign_contact_value(&mut row, header.trim(), value.trim());
    }

    row
}

fn contact_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> ContactCsvRow {
    let mut row = default_contact_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
        assign_contact_value(&mut row, target, value);
    }

    row
}

fn contact_row_from_mapped_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
    assignments: &[Option<String>],
) -> ContactCsvRow {
    let mut row = default_contact_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = json_preview_cell(object.get(headers[index].as_str()));
        let value = value.trim();
        assign_contact_value(&mut row, target, value);
    }

    row
}

fn default_contact_row() -> ContactCsvRow {
    ContactCsvRow {
        first_name: String::new(),
        last_name: None,
        org_name: None,
        email: None,
        phone: None,
        address: None,
        city: None,
        country: None,
        notes: None,
        custom_fields: BTreeMap::new(),
    }
}

fn assign_contact_value(row: &mut ContactCsvRow, target: &str, value: &str) {
    match target {
        "first_name" => row.first_name = value.to_string(),
        "last_name" => row.last_name = optional_csv_value(value),
        "org_name" => row.org_name = optional_csv_value(value),
        "email" => row.email = optional_csv_value(value),
        "phone" => row.phone = optional_csv_value(value),
        "address" => row.address = optional_csv_value(value),
        "city" => row.city = optional_csv_value(value),
        "country" => row.country = optional_csv_value(value),
        "notes" => row.notes = optional_csv_value(value),
        _ if is_custom_field_target(target) => {
            if let Some(value) = optional_csv_value(value) {
                row.custom_fields.insert(target.to_string(), value);
            }
        }
        _ => {}
    }
}

fn organization_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> OrganizationCsvRow {
    let mut row = default_organization_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
        assign_organization_value(&mut row, target, value);
    }

    row
}

fn organization_row_from_record(
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
) -> OrganizationCsvRow {
    let mut row = default_organization_row();

    for (index, header) in headers.iter().enumerate() {
        let value = record.get(index).unwrap_or_default().trim();
        assign_organization_value(&mut row, header.trim(), value);
    }

    row
}

fn organization_row_from_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
) -> OrganizationCsvRow {
    let mut row = default_organization_row();

    for header in headers {
        let value = json_preview_cell(object.get(header.as_str()));
        assign_organization_value(&mut row, header.trim(), value.trim());
    }

    row
}

fn organization_row_from_mapped_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
    assignments: &[Option<String>],
) -> OrganizationCsvRow {
    let mut row = default_organization_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = json_preview_cell(object.get(headers[index].as_str()));
        let value = value.trim();
        assign_organization_value(&mut row, target, value);
    }

    row
}

fn default_organization_row() -> OrganizationCsvRow {
    OrganizationCsvRow {
        name: String::new(),
        email: None,
        phone: None,
        website: None,
        address_line1: None,
        address_line2: None,
        city: None,
        region: None,
        country: None,
        postal_code: None,
        description: None,
        custom_fields: BTreeMap::new(),
    }
}

fn assign_organization_value(row: &mut OrganizationCsvRow, target: &str, value: &str) {
    match target {
        "name" => row.name = value.to_string(),
        "email" => row.email = optional_csv_value(value),
        "phone" => row.phone = optional_csv_value(value),
        "website" => row.website = optional_csv_value(value),
        "address_line1" => row.address_line1 = optional_csv_value(value),
        "address_line2" => row.address_line2 = optional_csv_value(value),
        "city" => row.city = optional_csv_value(value),
        "region" => row.region = optional_csv_value(value),
        "country" => row.country = optional_csv_value(value),
        "postal_code" => row.postal_code = optional_csv_value(value),
        "description" => row.description = optional_csv_value(value),
        _ if is_custom_field_target(target) => {
            if let Some(value) = optional_csv_value(value) {
                row.custom_fields.insert(target.to_string(), value);
            }
        }
        _ => {}
    }
}

fn deal_row_from_record(headers: &csv::StringRecord, record: &csv::StringRecord) -> DealCsvRow {
    let mut row = default_deal_row();

    for (index, header) in headers.iter().enumerate() {
        let value = record.get(index).unwrap_or_default().trim();
        assign_deal_value(&mut row, header.trim(), value);
    }

    row
}

fn deal_row_from_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
) -> DealCsvRow {
    let mut row = default_deal_row();

    for header in headers {
        let value = json_preview_cell(object.get(header.as_str()));
        assign_deal_value(&mut row, header.trim(), value.trim());
    }

    row
}

fn deal_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> DealCsvRow {
    let mut row = default_deal_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
        assign_deal_value(&mut row, target, value);
    }

    row
}

fn deal_row_from_mapped_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
    assignments: &[Option<String>],
) -> DealCsvRow {
    let mut row = default_deal_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = json_preview_cell(object.get(headers[index].as_str()));
        let value = value.trim();
        assign_deal_value(&mut row, target, value);
    }

    row
}

fn activity_row_from_record(
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
) -> ActivityCsvRow {
    let mut row = default_activity_row();

    for (index, header) in headers.iter().enumerate() {
        let value = record.get(index).unwrap_or_default().trim();
        assign_activity_value(&mut row, header.trim(), value);
    }

    row
}

fn activity_row_from_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
) -> ActivityCsvRow {
    let mut row = default_activity_row();

    for header in headers {
        let value = json_preview_cell(object.get(header.as_str()));
        assign_activity_value(&mut row, header.trim(), value.trim());
    }

    row
}

fn activity_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> ActivityCsvRow {
    let mut row = default_activity_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
        assign_activity_value(&mut row, target, value);
    }

    row
}

fn activity_row_from_mapped_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
    assignments: &[Option<String>],
) -> ActivityCsvRow {
    let mut row = default_activity_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = json_preview_cell(object.get(headers[index].as_str()));
        let value = value.trim();
        assign_activity_value(&mut row, target, value);
    }

    row
}

fn default_activity_row() -> ActivityCsvRow {
    ActivityCsvRow {
        activity_type: String::new(),
        title: String::new(),
        description: None,
        due_date: None,
        completed: None,
        contact_id: None,
        deal_id: None,
        custom_fields: BTreeMap::new(),
    }
}

fn assign_activity_value(row: &mut ActivityCsvRow, target: &str, value: &str) {
    match target {
        "activity_type" => row.activity_type = value.to_string(),
        "title" => row.title = value.to_string(),
        "description" => row.description = optional_csv_value(value),
        "due_date" => row.due_date = optional_csv_value(value),
        "completed" => row.completed = optional_bool_value(value),
        "contact_id" => row.contact_id = optional_csv_value(value),
        "deal_id" => row.deal_id = optional_csv_value(value),
        _ if is_custom_field_target(target) => {
            if let Some(value) = optional_csv_value(value) {
                row.custom_fields.insert(target.to_string(), value);
            }
        }
        _ => {}
    }
}

fn note_row_from_record(headers: &csv::StringRecord, record: &csv::StringRecord) -> NoteCsvRow {
    let mut row = default_note_row();

    for (index, header) in headers.iter().enumerate() {
        let value = record.get(index).unwrap_or_default().trim();
        assign_note_value(&mut row, header.trim(), value);
    }

    row
}

fn note_row_from_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
) -> NoteCsvRow {
    let mut row = default_note_row();

    for header in headers {
        let value = json_preview_cell(object.get(header.as_str()));
        assign_note_value(&mut row, header.trim(), value.trim());
    }

    row
}

fn note_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> NoteCsvRow {
    let mut row = default_note_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
        assign_note_value(&mut row, target, value);
    }

    row
}

fn note_row_from_mapped_json_object(
    object: &serde_json::Map<String, Value>,
    headers: &[String],
    assignments: &[Option<String>],
) -> NoteCsvRow {
    let mut row = default_note_row();

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = json_preview_cell(object.get(headers[index].as_str()));
        assign_note_value(&mut row, target, value.trim());
    }

    row
}

fn default_note_row() -> NoteCsvRow {
    NoteCsvRow {
        entity_type: String::new(),
        entity_id: String::new(),
        content: String::new(),
    }
}

fn assign_note_value(row: &mut NoteCsvRow, target: &str, value: &str) {
    match target {
        "entity_type" => row.entity_type = value.to_string(),
        "entity_id" => row.entity_id = value.to_string(),
        "content" => row.content = value.to_string(),
        _ => {}
    }
}

fn default_deal_row() -> DealCsvRow {
    DealCsvRow {
        title: String::new(),
        value: None,
        currency: None,
        stage: None,
        expected_close: None,
        notes: None,
        custom_fields: BTreeMap::new(),
    }
}

fn assign_deal_value(row: &mut DealCsvRow, target: &str, value: &str) {
    match target {
        "title" => row.title = value.to_string(),
        "value" => row.value = optional_csv_value(value),
        "currency" => row.currency = optional_csv_value(value),
        "stage" => row.stage = optional_csv_value(value),
        "expected_close" => row.expected_close = optional_csv_value(value),
        "notes" => row.notes = optional_csv_value(value),
        _ if is_custom_field_target(target) => {
            if let Some(value) = optional_csv_value(value) {
                row.custom_fields.insert(target.to_string(), value);
            }
        }
        _ => {}
    }
}

fn is_custom_field_target(target: &str) -> bool {
    target.starts_with(CUSTOM_FIELD_PREFIX) && target.len() > CUSTOM_FIELD_PREFIX.len()
}

fn optional_csv_value(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn optional_bool_value(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Export helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Serializes a slice of [`ContactCsvRow`] to CSV bytes.
///
/// The output always includes a header row. Fields are quoted if they contain
/// commas, newlines, or double-quotes.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if writing fails.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::csv::{ContactCsvRow, write_contacts_csv};
///
/// let rows = vec![ContactCsvRow { first_name: "Alice".into(), ..Default::default() }];
/// let bytes = write_contacts_csv(&rows).unwrap();
/// ```
pub fn write_contacts_csv<W: Write>(writer: W, rows: &[ContactCsvRow]) -> CrmResult<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    let custom_headers = collect_custom_headers(rows.iter().map(|row| &row.custom_fields));
    let mut headers = vec![
        "first_name".to_string(),
        "last_name".to_string(),
        "org_name".to_string(),
        "email".to_string(),
        "phone".to_string(),
        "address".to_string(),
        "city".to_string(),
        "country".to_string(),
        "notes".to_string(),
    ];
    headers.extend(custom_headers.iter().cloned());
    wtr.write_record(&headers)
        .map_err(|e| CrmError::Csv(e.to_string()))?;

    for row in rows {
        let mut record = vec![
            row.first_name.clone(),
            row.last_name.clone().unwrap_or_default(),
            row.org_name.clone().unwrap_or_default(),
            row.email.clone().unwrap_or_default(),
            row.phone.clone().unwrap_or_default(),
            row.address.clone().unwrap_or_default(),
            row.city.clone().unwrap_or_default(),
            row.country.clone().unwrap_or_default(),
            row.notes.clone().unwrap_or_default(),
        ];
        record.extend(
            custom_headers
                .iter()
                .map(|header| row.custom_fields.get(header).cloned().unwrap_or_default()),
        );
        wtr.write_record(&record)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} contact rows to CSV", rows.len());
    Ok(())
}

/// Serializes a slice of [`DealCsvRow`] to CSV bytes.
///
/// The output always includes a header row.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if writing fails.
pub fn write_deals_csv<W: Write>(writer: W, rows: &[DealCsvRow]) -> CrmResult<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    let custom_headers = collect_custom_headers(rows.iter().map(|row| &row.custom_fields));
    let mut headers = vec![
        "title".to_string(),
        "value".to_string(),
        "currency".to_string(),
        "stage".to_string(),
        "expected_close".to_string(),
        "notes".to_string(),
    ];
    headers.extend(custom_headers.iter().cloned());
    wtr.write_record(&headers)
        .map_err(|e| CrmError::Csv(e.to_string()))?;

    for row in rows {
        let mut record = vec![
            row.title.clone(),
            row.value.clone().unwrap_or_default(),
            row.currency.clone().unwrap_or_default(),
            row.stage.clone().unwrap_or_default(),
            row.expected_close.clone().unwrap_or_default(),
            row.notes.clone().unwrap_or_default(),
        ];
        record.extend(
            custom_headers
                .iter()
                .map(|header| row.custom_fields.get(header).cloned().unwrap_or_default()),
        );
        wtr.write_record(&record)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} deal rows to CSV", rows.len());
    Ok(())
}

/// Serializes a slice of [`ActivityCsvRow`] to CSV bytes.
///
/// The output always includes a header row.
pub fn write_activities_csv<W: Write>(writer: W, rows: &[ActivityCsvRow]) -> CrmResult<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    let custom_headers = collect_custom_headers(rows.iter().map(|row| &row.custom_fields));
    let mut headers = vec![
        "activity_type".to_string(),
        "title".to_string(),
        "description".to_string(),
        "due_date".to_string(),
        "completed".to_string(),
        "contact_id".to_string(),
        "deal_id".to_string(),
    ];
    headers.extend(custom_headers.iter().cloned());
    wtr.write_record(&headers)
        .map_err(|e| CrmError::Csv(e.to_string()))?;

    for row in rows {
        let mut record = vec![
            row.activity_type.clone(),
            row.title.clone(),
            row.description.clone().unwrap_or_default(),
            row.due_date.clone().unwrap_or_default(),
            row.completed
                .map(|completed| completed.to_string())
                .unwrap_or_default(),
            row.contact_id.clone().unwrap_or_default(),
            row.deal_id.clone().unwrap_or_default(),
        ];
        record.extend(
            custom_headers
                .iter()
                .map(|header| row.custom_fields.get(header).cloned().unwrap_or_default()),
        );
        wtr.write_record(&record)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} activity rows to CSV", rows.len());
    Ok(())
}

fn collect_custom_headers<'a, I>(custom_fields: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a BTreeMap<String, String>>,
{
    let mut headers = BTreeSet::new();
    for fields in custom_fields {
        headers.extend(
            fields
                .keys()
                .filter(|key| is_custom_field_target(key))
                .cloned(),
        );
    }
    headers.into_iter().collect()
}

/// Serializes a slice of [`OrganizationCsvRow`] to CSV bytes.
///
/// The output always includes a header row.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if writing fails.
pub fn write_organizations_csv<W: Write>(writer: W, rows: &[OrganizationCsvRow]) -> CrmResult<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    let custom_headers = collect_custom_headers(rows.iter().map(|row| &row.custom_fields));
    let mut headers = vec![
        "name".to_string(),
        "email".to_string(),
        "phone".to_string(),
        "website".to_string(),
        "address_line1".to_string(),
        "address_line2".to_string(),
        "city".to_string(),
        "region".to_string(),
        "country".to_string(),
        "postal_code".to_string(),
        "description".to_string(),
    ];
    headers.extend(custom_headers.iter().cloned());
    wtr.write_record(&headers)
        .map_err(|e| CrmError::Csv(e.to_string()))?;

    for row in rows {
        let mut record = vec![
            row.name.clone(),
            row.email.clone().unwrap_or_default(),
            row.phone.clone().unwrap_or_default(),
            row.website.clone().unwrap_or_default(),
            row.address_line1.clone().unwrap_or_default(),
            row.address_line2.clone().unwrap_or_default(),
            row.city.clone().unwrap_or_default(),
            row.region.clone().unwrap_or_default(),
            row.country.clone().unwrap_or_default(),
            row.postal_code.clone().unwrap_or_default(),
            row.description.clone().unwrap_or_default(),
        ];
        record.extend(
            custom_headers
                .iter()
                .map(|header| row.custom_fields.get(header).cloned().unwrap_or_default()),
        );
        wtr.write_record(&record)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} organization rows to CSV", rows.len());
    Ok(())
}

/// Serializes a slice of [`NoteCsvRow`] to CSV bytes.
///
/// The output always includes a header row with `entity_type`, `entity_id`, and
/// `content`.
///
/// # Errors
///
/// Returns [`CrmError::Csv`] if writing fails.
pub fn write_notes_csv<W: Write>(writer: W, rows: &[NoteCsvRow]) -> CrmResult<()> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);

    wtr.write_record(["entity_type", "entity_id", "content"])
        .map_err(|e| CrmError::Csv(e.to_string()))?;

    for row in rows {
        wtr.write_record([&row.entity_type, &row.entity_id, &row.content])
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} note rows to CSV", rows.len());
    Ok(())
}
