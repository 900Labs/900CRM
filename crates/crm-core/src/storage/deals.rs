//! Deal CRUD operations and pipeline queries for 900CRM.
//!
//! This module provides all database operations for the `deals` table,
//! including pipeline stage management and aggregate summary queries.
//!
//! # Pipeline Stages
//!
//! The default pipeline stages (in order) are:
//! `Lead → Qualified → Proposal → Negotiation → Closed Won → Closed Lost`
//!
//! # Soft Delete
//!
//! Deals are never physically deleted. `soft_delete_deal` sets `deleted_at`
//! and all list queries exclude deleted records by default.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A sales deal moving through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    /// UUID v4 primary key.
    pub id: String,

    /// Human-readable deal title.
    pub title: String,

    /// Monetary value of the deal.
    pub value: f64,

    /// ISO 4217 currency code (e.g. `"USD"`, `"EUR"`).
    pub currency: String,

    /// Current pipeline stage (e.g. `"Lead"`, `"Proposal"`).
    pub stage: String,

    /// Probability of closing (0–100).
    pub probability: i32,

    /// Expected close date in `YYYY-MM-DD` or ISO 8601 format.
    pub expected_close: Option<String>,

    /// Optional associated contact UUID.
    pub contact_id: Option<String>,

    /// Freeform notes about the deal.
    pub notes: String,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

/// Aggregated summary for a single pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSummary {
    /// Stage name (e.g. `"Lead"`, `"Proposal"`).
    pub stage: String,

    /// Number of active deals in this stage.
    pub count: i64,

    /// Sum of `value` across all active deals in this stage.
    pub total_value: f64,

    /// Probability-weighted sum: `SUM(value * probability / 100)`.
    pub weighted_value: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new deal record.
///
/// Generates a UUID for `id`, sets `created_at` and `updated_at` to now.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn create_deal(
    conn: &Connection,
    title: &str,
    value: f64,
    currency: &str,
    stage: &str,
    probability: i32,
    expected_close: Option<&str>,
    contact_id: Option<&str>,
    notes: &str,
    device_id: &str,
) -> CrmResult<Deal> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO deals
            (id, title, value, currency, stage, probability, expected_close,
             contact_id, notes, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![
            id,
            title,
            value,
            currency,
            stage,
            probability,
            expected_close,
            contact_id,
            notes,
            now,
            now,
            device_id
        ],
    )?;

    log::debug!("Created deal id={} title={}", id, title);
    get_deal(conn, &id)
}

/// Retrieves a single active deal by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_deal(conn: &Connection, id: &str) -> CrmResult<Deal> {
    conn.query_row(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, notes, created_at, updated_at, deleted_at, device_id
        FROM deals
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_deal,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Deal '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Lists all active deals, ordered by creation date descending.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_deals(conn: &Connection) -> CrmResult<Vec<Deal>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, notes, created_at, updated_at, deleted_at, device_id
        FROM deals
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )?;

    let rows = stmt.query_map([], |row| row_to_deal(row))?;
    let deals: Vec<Deal> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_deals: {} results", deals.len());
    Ok(deals)
}

/// Lists all active deals in a specific pipeline stage.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_deals_by_stage(conn: &Connection, stage: &str) -> CrmResult<Vec<Deal>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, notes, created_at, updated_at, deleted_at, device_id
        FROM deals
        WHERE stage = ?1 AND deleted_at IS NULL
        ORDER BY updated_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![stage], |row| row_to_deal(row))?;
    let deals: Vec<Deal> = rows.filter_map(|r| r.ok()).collect();

    log::debug!(
        "list_deals_by_stage stage={}: {} results",
        stage,
        deals.len()
    );
    Ok(deals)
}

/// Updates a deal's fields.
///
/// All `Option` parameters are applied only if `Some`. Sets `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is deleted.
/// - [`CrmError::Database`] — SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn update_deal(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    value: Option<f64>,
    currency: Option<&str>,
    stage: Option<&str>,
    probability: Option<i32>,
    expected_close: Option<Option<&str>>,
    contact_id: Option<Option<&str>>,
    notes: Option<&str>,
) -> CrmResult<Deal> {
    let current = get_deal(conn, id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE deals SET
            title          = ?1,
            value          = ?2,
            currency       = ?3,
            stage          = ?4,
            probability    = ?5,
            expected_close = ?6,
            contact_id     = ?7,
            notes          = ?8,
            updated_at     = ?9
        WHERE id = ?10 AND deleted_at IS NULL
        "#,
        params![
            title.unwrap_or(&current.title),
            value.unwrap_or(current.value),
            currency.unwrap_or(&current.currency),
            stage.unwrap_or(&current.stage),
            probability.unwrap_or(current.probability),
            expected_close.unwrap_or(current.expected_close.as_deref()),
            contact_id.unwrap_or(current.contact_id.as_deref()),
            notes.unwrap_or(&current.notes),
            now,
            id
        ],
    )?;

    log::debug!("Updated deal id={}", id);
    get_deal(conn, id)
}

/// Moves a deal to a new pipeline stage and optionally updates probability.
///
/// If `probability` is `None`, the default probability for the target stage
/// is applied automatically:
/// - Lead → 10
/// - Qualified → 25
/// - Proposal → 50
/// - Negotiation → 75
/// - Closed Won → 100
/// - Closed Lost → 0
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn move_deal_stage(
    conn: &Connection,
    id: &str,
    new_stage: &str,
    probability: Option<i32>,
) -> CrmResult<Deal> {
    let prob = probability.unwrap_or_else(|| default_probability_for_stage(new_stage));
    let now = now_iso8601();

    let changed = conn.execute(
        "UPDATE deals SET stage = ?1, probability = ?2, updated_at = ?3 WHERE id = ?4 AND deleted_at IS NULL",
        params![new_stage, prob, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Deal '{}' not found", id)));
    }

    log::info!("Moved deal id={} to stage={}", id, new_stage);
    get_deal(conn, id)
}

/// Soft-deletes a deal by setting `deleted_at`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_deal(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE deals SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Deal '{}' not found or already deleted",
            id
        )));
    }

    log::info!("Soft-deleted deal id={}", id);
    Ok(())
}

/// Returns a [`PipelineSummary`] for each pipeline stage.
///
/// Only active (non-deleted) deals are included. Stages with no deals are
/// included with `count = 0` and `total_value = 0.0`.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_pipeline_summary(conn: &Connection) -> CrmResult<Vec<PipelineSummary>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            stage,
            COUNT(*)                                AS count,
            COALESCE(SUM(value), 0.0)               AS total_value,
            COALESCE(SUM(value * probability / 100.0), 0.0) AS weighted_value
        FROM deals
        WHERE deleted_at IS NULL
        GROUP BY stage
        ORDER BY CASE stage
            WHEN 'Lead'        THEN 1
            WHEN 'Qualified'   THEN 2
            WHEN 'Proposal'    THEN 3
            WHEN 'Negotiation' THEN 4
            WHEN 'Closed Won'  THEN 5
            WHEN 'Closed Lost' THEN 6
            ELSE 7
        END
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PipelineSummary {
            stage: row.get(0)?,
            count: row.get(1)?,
            total_value: row.get(2)?,
            weighted_value: row.get(3)?,
        })
    })?;

    let summaries: Vec<PipelineSummary> = rows.filter_map(|r| r.ok()).collect();
    log::debug!("get_pipeline_summary: {} stages", summaries.len());
    Ok(summaries)
}

/// Returns the average age in days for active, non-deleted deals.
pub fn get_average_active_deal_age_days(conn: &Connection) -> CrmResult<f64> {
    let average = conn
        .query_row(
            r#"
            SELECT AVG(
                CAST(
                    (julianday('now') - julianday(created_at))
                AS REAL)
            )
            FROM deals
            WHERE deleted_at IS NULL
              AND stage NOT IN ('Closed Won', 'Closed Lost')
            "#,
            [],
            |row| row.get::<_, Option<f64>>(0),
        )?
        .unwrap_or(0.0);

    Ok(average)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to a [`Deal`].
fn row_to_deal(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deal> {
    Ok(Deal {
        id: row.get(0)?,
        title: row.get(1)?,
        value: row.get(2)?,
        currency: row.get(3)?,
        stage: row.get(4)?,
        probability: row.get(5)?,
        expected_close: row.get(6)?,
        contact_id: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
        device_id: row.get(12)?,
    })
}

/// Returns the default win probability (0–100) for a given pipeline stage.
fn default_probability_for_stage(stage: &str) -> i32 {
    match stage {
        "Lead" => 10,
        "Qualified" => 25,
        "Proposal" => 50,
        "Negotiation" => 75,
        "Closed Won" => 100,
        "Closed Lost" => 0,
        _ => 20,
    }
}
