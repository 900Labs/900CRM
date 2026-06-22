//! Activity business logic — scheduling, overdue detection, and summary stats.
//!
//! This module provides domain-level logic for activities:
//!
//! - [`ActivityStats`] — aggregated counts for the dashboard.
//! - [`validate_activity_input`] — validates an activity before create/update.
//! - [`get_activity_stats`] — computes completed, pending, and overdue counts.
//! - [`is_overdue`] — tests whether an activity's due date has passed.
//! - [`ACTIVITY_TYPES`] — the standard set of activity type strings.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::storage::activities as activity_storage;
use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
};

// ─────────────────────────────────────────────────────────────────────────────
// Activity type constants
// ─────────────────────────────────────────────────────────────────────────────

/// Standard activity type strings.
///
/// Custom types are allowed by the storage layer; these are the canonical values
/// that the frontend uses for icon and label selection.
pub const ACTIVITY_TYPES: &[&str] = &["task", "call", "meeting", "email", "note", "follow_up"];

/// Returns `true` if `activity_type` is one of the standard activity types.
pub fn is_standard_type(activity_type: &str) -> bool {
    ACTIVITY_TYPES.contains(&activity_type)
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validates an activity before creation.
///
/// # Rules
///
/// - `title` must be non-empty.
/// - `activity_type`, if provided, must be non-empty.
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] on failure.
pub fn validate_activity_for_create(title: &str, activity_type: &str) -> CrmResult<()> {
    if title.trim().is_empty() {
        return Err(CrmError::InvalidInput(
            "Activity title is required".to_string(),
        ));
    }
    if activity_type.trim().is_empty() {
        return Err(CrmError::InvalidInput(
            "Activity type is required".to_string(),
        ));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Overdue detection
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if `due_date` is in the past (compared to the current UTC time).
///
/// Returns `false` if `due_date` is `None` (no deadline set).
///
/// # Parameters
///
/// - `due_date` — ISO 8601 or `YYYY-MM-DD` date string, or `None`.
/// - `completed` — If `true`, the activity is never considered overdue.
///
/// # Example
///
/// ```rust,ignore
/// use crate::crm_engine::activities::is_overdue;
///
/// assert!(is_overdue(Some("2020-01-01T00:00:00Z"), false));
/// assert!(!is_overdue(Some("2099-01-01T00:00:00Z"), false));
/// assert!(!is_overdue(None, false));
/// assert!(!is_overdue(Some("2020-01-01T00:00:00Z"), true)); // completed
/// ```
pub fn is_overdue(due_date: Option<&str>, completed: bool) -> bool {
    if completed {
        return false;
    }
    let Some(due) = due_date else { return false };
    let now = now_iso8601();
    due < now.as_str()
}

// ─────────────────────────────────────────────────────────────────────────────
// Activity statistics
// ─────────────────────────────────────────────────────────────────────────────

/// Aggregated activity statistics for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    /// Total number of active (non-deleted) activities.
    pub total: i64,

    /// Number of completed activities.
    pub completed: i64,

    /// Number of pending (not completed, not overdue) activities.
    pub pending: i64,

    /// Number of overdue activities (past due, not completed).
    pub overdue: i64,

    /// Number of activities due today.
    pub due_today: i64,
}

/// Computes [`ActivityStats`] from repository-provided activity counts.
///
/// Used by the dashboard command to show activity health at a glance.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on storage failure.
pub fn get_activity_stats(conn: &Connection) -> CrmResult<ActivityStats> {
    let now = now_iso8601();
    let today_prefix = &now[..10];
    let counts = activity_storage::get_activity_stats_counts(conn, &now, today_prefix)?;
    let pending = counts.total - counts.completed - counts.overdue;

    Ok(ActivityStats {
        total: counts.total,
        completed: counts.completed,
        pending: pending.max(0),
        overdue: counts.overdue,
        due_today: counts.due_today,
    })
}
