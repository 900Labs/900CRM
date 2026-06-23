//! Activity CRUD operations and scheduling queries for 900CRM.
//!
//! Activities represent tasks, calls, meetings, and emails. They can be
//! attached to a contact, a deal, or both.
//!
//! # Activity Types
//!
//! The `activity_type` field is a freeform string. Standard values are:
//! `"task"`, `"call"`, `"meeting"`, `"email"`, `"note"`.
//!
//! # Soft Delete
//!
//! Activities support the same soft-delete pattern as contacts and deals.

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

/// An activity (task, call, meeting, or email) in the CRM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// UUID v4 primary key.
    pub id: String,

    /// Activity type: `"task"`, `"call"`, `"meeting"`, `"email"`, etc.
    pub activity_type: String,

    /// Short title or subject of the activity.
    pub title: String,

    /// Longer description or body.
    pub description: String,

    /// Optional due date (ISO 8601 or `YYYY-MM-DD`).
    pub due_date: Option<String>,

    /// Whether the activity has been completed (`true`) or is still pending.
    pub completed: bool,

    /// Optional associated contact UUID.
    pub contact_id: Option<String>,

    /// Optional associated deal UUID.
    pub deal_id: Option<String>,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

/// Aggregate activity counts used by dashboard orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStatsCounts {
    pub total: i64,
    pub completed: i64,
    pub overdue: i64,
    pub due_today: i64,
}

/// Loads aggregate activity counts for dashboard statistics.
pub fn get_activity_stats_counts(
    conn: &Connection,
    now: &str,
    today_prefix: &str,
) -> CrmResult<ActivityStatsCounts> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM activities WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM activities WHERE deleted_at IS NULL AND completed = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let overdue: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(*) FROM activities
            WHERE deleted_at IS NULL
              AND completed = 0
              AND due_date IS NOT NULL
              AND due_date < ?1
            "#,
            params![now],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let due_today: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(*) FROM activities
            WHERE deleted_at IS NULL
              AND completed = 0
              AND due_date LIKE ?1
            "#,
            params![format!("{today_prefix}%")],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(ActivityStatsCounts {
        total,
        completed,
        overdue,
        due_today,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new activity record.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn create_activity(
    conn: &Connection,
    activity_type: &str,
    title: &str,
    description: &str,
    due_date: Option<&str>,
    contact_id: Option<&str>,
    deal_id: Option<&str>,
    device_id: &str,
) -> CrmResult<Activity> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO activities
            (id, activity_type, title, description, due_date, completed,
             contact_id, deal_id, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            id,
            activity_type,
            title,
            description,
            due_date,
            contact_id,
            deal_id,
            now,
            now,
            device_id
        ],
    )?;

    log::debug!("Created activity id={} type={}", id, activity_type);
    get_activity(conn, &id)
}

/// Retrieves a single active activity by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_activity(conn: &Connection, id: &str) -> CrmResult<Activity> {
    conn.query_row(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_activity,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Activity '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Lists all active activities, ordered by due date ascending (nulls last).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities(conn: &Connection) -> CrmResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#,
    )?;

    let rows = stmt.query_map([], |row| row_to_activity(row))?;
    let activities: Vec<Activity> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_activities: {} results", activities.len());
    Ok(activities)
}

/// Lists all active activities for a specific contact.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities_for_contact(
    conn: &Connection,
    contact_id: &str,
) -> CrmResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE contact_id = ?1 AND deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![contact_id], |row| row_to_activity(row))?;
    let activities: Vec<Activity> = rows.filter_map(|r| r.ok()).collect();

    log::debug!(
        "list_activities_for_contact contact_id={}: {} results",
        contact_id,
        activities.len()
    );
    Ok(activities)
}

/// Lists all active activities for a specific deal.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities_for_deal(conn: &Connection, deal_id: &str) -> CrmResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deal_id = ?1 AND deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![deal_id], |row| row_to_activity(row))?;
    let activities: Vec<Activity> = rows.filter_map(|r| r.ok()).collect();

    log::debug!(
        "list_activities_for_deal deal_id={}: {} results",
        deal_id,
        activities.len()
    );
    Ok(activities)
}

/// Lists upcoming (incomplete, future due date) activities.
///
/// Returns at most `limit` activities ordered by `due_date` ascending.
/// Excludes completed and soft-deleted activities.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_upcoming_activities(conn: &Connection, limit: u32) -> CrmResult<Vec<Activity>> {
    let now = now_iso8601();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
          AND completed = 0
          AND due_date >= ?1
        ORDER BY due_date ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![now, limit as i64], |row| row_to_activity(row))?;
    let activities: Vec<Activity> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_upcoming_activities: {} results", activities.len());
    Ok(activities)
}

/// Lists overdue (incomplete, past due date) activities.
///
/// Returns activities ordered by `due_date` descending (most overdue first).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_overdue_activities(conn: &Connection) -> CrmResult<Vec<Activity>> {
    let now = now_iso8601();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
          AND completed = 0
          AND due_date < ?1
          AND due_date IS NOT NULL
        ORDER BY due_date ASC
        "#,
    )?;

    let rows = stmt.query_map(params![now], |row| row_to_activity(row))?;
    let activities: Vec<Activity> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_overdue_activities: {} results", activities.len());
    Ok(activities)
}

/// Marks an activity as completed.
///
/// Sets `completed = 1` and `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn mark_complete(conn: &Connection, id: &str) -> CrmResult<Activity> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET completed = 1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Activity '{}' not found", id)));
    }

    log::info!("Marked activity id={} as complete", id);
    get_activity(conn, id)
}

/// Marks an activity as incomplete.
///
/// Sets `completed = 0` and `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn mark_incomplete(conn: &Connection, id: &str) -> CrmResult<Activity> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET completed = 0, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Activity '{}' not found", id)));
    }

    log::info!("Marked activity id={} as incomplete", id);
    get_activity(conn, id)
}

/// Updates an activity's fields.
///
/// All `Option` parameters are applied only if `Some`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or deleted.
/// - [`CrmError::Database`] — SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn update_activity(
    conn: &Connection,
    id: &str,
    activity_type: Option<&str>,
    title: Option<&str>,
    description: Option<&str>,
    due_date: Option<Option<&str>>,
    completed: Option<bool>,
    contact_id: Option<Option<&str>>,
    deal_id: Option<Option<&str>>,
) -> CrmResult<Activity> {
    let current = get_activity(conn, id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE activities SET
            activity_type = ?1,
            title         = ?2,
            description   = ?3,
            due_date      = ?4,
            completed     = ?5,
            contact_id    = ?6,
            deal_id       = ?7,
            updated_at    = ?8
        WHERE id = ?9 AND deleted_at IS NULL
        "#,
        params![
            activity_type.unwrap_or(&current.activity_type),
            title.unwrap_or(&current.title),
            description.unwrap_or(&current.description),
            due_date.unwrap_or(current.due_date.as_deref()),
            completed
                .map(|c| c as i32)
                .unwrap_or(current.completed as i32),
            contact_id.unwrap_or(current.contact_id.as_deref()),
            deal_id.unwrap_or(current.deal_id.as_deref()),
            now,
            id
        ],
    )?;

    log::debug!("Updated activity id={}", id);
    get_activity(conn, id)
}

/// Soft-deletes an activity.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_activity(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Activity '{}' not found or already deleted",
            id
        )));
    }

    log::info!("Soft-deleted activity id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to an [`Activity`].
fn row_to_activity(row: &rusqlite::Row<'_>) -> rusqlite::Result<Activity> {
    let completed_int: i32 = row.get(5)?;
    Ok(Activity {
        id: row.get(0)?,
        activity_type: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        due_date: row.get(4)?,
        completed: completed_int != 0,
        contact_id: row.get(6)?,
        deal_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        device_id: row.get(11)?,
    })
}
