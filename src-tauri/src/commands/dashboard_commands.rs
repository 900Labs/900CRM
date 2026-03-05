//! Tauri IPC command for the dashboard statistics endpoint.
//!
//! The dashboard aggregates data from contacts, deals, and activities into
//! a single [`DashboardStats`] payload consumed by the frontend home screen.

use tauri::State;
use serde::{Deserialize, Serialize};

use crate::crm_engine::{activities as activity_engine, deals as deal_engine};
use crate::storage::deals;
use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// DashboardStats
// ─────────────────────────────────────────────────────────────────────────────

/// Aggregated statistics displayed on the 900CRM dashboard home screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Total number of active (non-deleted) person contacts.
    pub total_contacts: i64,

    /// Total number of active organization contacts.
    pub total_organizations: i64,

    /// Number of active deals (not closed, not deleted).
    pub active_deals: i64,

    /// Sum of `value` across all active (non-closed) deals.
    pub pipeline_value: f64,

    /// Active pipeline totals grouped by currency code.
    pub pipeline_value_by_currency: Vec<CurrencyPipelineValue>,

    /// Probability-weighted pipeline value.
    pub weighted_pipeline: f64,

    /// Number of upcoming (future-due, incomplete) activities.
    pub upcoming_activities: i64,

    /// Number of overdue (past-due, incomplete) activities.
    pub overdue_activities: i64,

    /// Win rate: closed_won / (closed_won + closed_lost). Between 0.0 and 1.0.
    pub win_rate: f64,

    /// Total contacts created this month.
    pub new_contacts_this_month: i64,

    /// Total deals created this month.
    pub new_deals_this_month: i64,
}

/// Pipeline aggregate for a single currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPipelineValue {
    /// ISO 4217 currency code.
    pub currency: String,
    /// Sum of active deal values in this currency.
    pub total_value: f64,
    /// Number of active deals in this currency.
    pub deal_count: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// get_dashboard_stats
// ─────────────────────────────────────────────────────────────────────────────

/// Returns aggregate CRM statistics for the dashboard home screen.
///
/// Queries contacts, deals, and activities in a single call and assembles
/// the [`DashboardStats`] payload.
///
/// # Errors
///
/// Returns a `String` error message on database failure.
#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let conn = &db.conn;

    // ── Contact counts ────────────────────────────────────────────────────────
    let total_contacts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND contact_type = 'person'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_organizations: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND contact_type = 'organization'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Deal metrics ──────────────────────────────────────────────────────────
    let pipeline_summaries = deals::get_pipeline_summary(conn).map_err(|e| e.to_string())?;

    let active_deals: i64 = pipeline_summaries
        .iter()
        .filter(|s| s.stage != "Closed Won" && s.stage != "Closed Lost")
        .map(|s| s.count)
        .sum();

    let pipeline_value = deal_engine::calculate_total_pipeline_value(&pipeline_summaries);
    let weighted_pipeline = deal_engine::calculate_weighted_pipeline(&pipeline_summaries);
    let win_rate = deal_engine::calculate_win_rate(&pipeline_summaries);
    let pipeline_value_by_currency = {
        let mut stmt = conn
            .prepare(
                "SELECT currency, COALESCE(SUM(value), 0), COUNT(*)
                 FROM deals
                 WHERE deleted_at IS NULL
                   AND stage NOT IN ('Closed Won', 'Closed Lost')
                 GROUP BY currency
                 ORDER BY ABS(SUM(value)) DESC, currency ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CurrencyPipelineValue {
                    currency: row.get::<_, String>(0).unwrap_or_else(|_| "USD".to_string()),
                    total_value: row.get::<_, f64>(1).unwrap_or(0.0),
                    deal_count: row.get::<_, i64>(2).unwrap_or(0),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut values = Vec::new();
        for row in rows {
            values.push(row.map_err(|e| e.to_string())?);
        }
        values
    };

    // ── Activity metrics ──────────────────────────────────────────────────────
    let activity_stats = activity_engine::get_activity_stats(conn).map_err(|e| e.to_string())?;
    let upcoming_activities = activity_stats.pending;
    let overdue_activities = activity_stats.overdue;

    // ── This-month metrics ────────────────────────────────────────────────────
    let month_prefix = {
        let now = crate::utils::datetime::now_iso8601();
        format!("{}%", &now[..7]) // "2024-03%"
    };

    let new_contacts_this_month: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND created_at LIKE ?1",
            rusqlite::params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_deals_this_month: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deals WHERE deleted_at IS NULL AND created_at LIKE ?1",
            rusqlite::params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let stats = DashboardStats {
        total_contacts,
        total_organizations,
        active_deals,
        pipeline_value,
        pipeline_value_by_currency,
        weighted_pipeline,
        upcoming_activities,
        overdue_activities,
        win_rate,
        new_contacts_this_month,
        new_deals_this_month,
    };

    log::debug!(
        "Command: get_dashboard_stats contacts={} deals={} pipeline={:.2}",
        total_contacts,
        active_deals,
        pipeline_value
    );

    Ok(stats)
}
