//! CSV import and export helpers for contacts, deals, and organizations.
//!
//! This module provides utilities for reading and writing CSV files used in the
//! 900CRM import/export feature. It wraps the [`csv`] crate and maps between
//! CSV rows and the application's domain structs.
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

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::utils::errors::{CrmError, CrmResult};

/// Frontend-provided CSV mapping: source CSV header -> target CRM field.
///
/// `None` means the source header is intentionally skipped. Source headers not
/// present in the map are also skipped.
pub type ImportColumnMapping = HashMap<String, Option<String>>;

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

    let mut rows = Vec::new();
    for (index, result) in rdr.deserialize::<ContactCsvRow>().enumerate() {
        let row_number = index + 2;
        match result {
            Ok(row) => {
                if row.first_name.trim().is_empty() {
                    log::debug!("Skipping CSV row with blank first_name");
                    continue;
                }
                rows.push((row_number, row));
            }
            Err(e) => {
                log::error!("CSV parse error: {}", e);
                return Err(CrmError::Csv(e.to_string()));
            }
        }
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
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments = validate_import_mapping(&headers, mapping, CONTACT_IMPORT_TARGET_FIELDS)?;

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
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let mut rows = Vec::new();
    for result in rdr.deserialize::<DealCsvRow>() {
        match result {
            Ok(row) => {
                if row.title.trim().is_empty() {
                    log::debug!("Skipping CSV row with blank title");
                    continue;
                }
                rows.push(row);
            }
            Err(e) => {
                log::error!("CSV parse error: {}", e);
                return Err(CrmError::Csv(e.to_string()));
            }
        }
    }

    log::info!("Parsed {} deal rows from CSV", rows.len());
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

    let mut rows = Vec::new();
    for (index, result) in rdr.deserialize::<OrganizationCsvRow>().enumerate() {
        let row_number = index + 2;
        match result {
            Ok(row) => {
                if row.name.trim().is_empty() {
                    log::debug!("Skipping CSV row with blank name");
                    continue;
                }
                rows.push((row_number, row));
            }
            Err(e) => {
                log::error!("CSV parse error: {}", e);
                return Err(CrmError::Csv(e.to_string()));
            }
        }
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
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let assignments =
        validate_import_mapping(&headers, mapping, ORGANIZATION_IMPORT_TARGET_FIELDS)?;

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

fn validate_import_mapping(
    headers: &csv::StringRecord,
    mapping: &ImportColumnMapping,
    allowed_targets: &[&str],
) -> CrmResult<Vec<Option<String>>> {
    let header_set: HashSet<&str> = headers.iter().collect();
    let allowed_target_set: HashSet<&str> = allowed_targets.iter().copied().collect();
    let mut assigned_targets: HashMap<String, String> = HashMap::new();

    for (source_header, target) in mapping {
        if !header_set.contains(source_header.as_str()) {
            return Err(CrmError::InvalidInput(format!(
                "Mapped source header '{}' is not present in the CSV",
                source_header
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

fn contact_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> ContactCsvRow {
    let mut row = ContactCsvRow {
        first_name: String::new(),
        last_name: None,
        org_name: None,
        email: None,
        phone: None,
        address: None,
        city: None,
        country: None,
        notes: None,
    };

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
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
            _ => {}
        }
    }

    row
}

fn organization_row_from_mapped_record(
    record: &csv::StringRecord,
    assignments: &[Option<String>],
) -> OrganizationCsvRow {
    let mut row = OrganizationCsvRow {
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
    };

    for (index, target) in assignments.iter().enumerate() {
        let Some(target) = target.as_deref() else {
            continue;
        };
        let value = record.get(index).unwrap_or_default().trim();
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
            _ => {}
        }
    }

    row
}

fn optional_csv_value(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
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
        .has_headers(true)
        .from_writer(writer);

    for row in rows {
        wtr.serialize(row)
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
        .has_headers(true)
        .from_writer(writer);

    for row in rows {
        wtr.serialize(row)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} deal rows to CSV", rows.len());
    Ok(())
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
        .has_headers(true)
        .from_writer(writer);

    for row in rows {
        wtr.serialize(row)
            .map_err(|e| CrmError::Csv(e.to_string()))?;
    }

    wtr.flush().map_err(|e| CrmError::Csv(e.to_string()))?;
    log::info!("Wrote {} organization rows to CSV", rows.len());
    Ok(())
}
