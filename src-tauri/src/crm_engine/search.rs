//! Unified cross-entity search for 900CRM.
//!
//! This module provides the [`unified_search`] function that searches across
//! contacts, deals, and activities simultaneously and returns a combined list
//! of [`SearchResult`] items.
//!
//! # Search Strategy
//!
//! - **Contacts**: FTS5 full-text search via `contacts_fts`.
//! - **Deals**: `LIKE '%query%'` on `title` and `notes`.
//! - **Activities**: `LIKE '%query%'` on `title` and `description`.
//!
//! Results from all three sources are combined, with contacts ranked first
//! (FTS5 match quality), then deals, then activities.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::errors::CrmResult;

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A single search result from the unified search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Entity type: `"contact"`, `"deal"`, or `"activity"`.
    pub entity_type: String,

    /// UUID of the matching entity.
    pub entity_id: String,

    /// Primary display title (e.g. full name, deal title, activity title).
    pub title: String,

    /// Secondary subtitle (e.g. email, stage, activity type).
    pub subtitle: String,

    /// Which field produced the match (e.g. `"email"`, `"title"`, `"description"`).
    pub match_field: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified search
// ─────────────────────────────────────────────────────────────────────────────

/// Searches contacts, deals, and activities for `query`.
///
/// Returns at most `limit` results (default 30), sorted by entity type:
/// contacts first, then deals, then activities.
///
/// # Parameters
///
/// - `conn` — SQLite connection.
/// - `query` — Search string. Trimmed before use.
/// - `limit` — Maximum number of results to return. Clamped to [1, 100].
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on SQL failure.
pub fn unified_search(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> CrmResult<Vec<SearchResult>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }

    let clamped_limit = limit.max(1).min(100) as i64;
    let like_pattern = format!("%{}%", q);
    let fts_query = format!("{}*", q);

    let mut results: Vec<SearchResult> = Vec::new();

    // ── Contacts (FTS5) ──────────────────────────────────────────────────────
    let contact_results = search_contacts_fts(conn, &fts_query, clamped_limit)?;
    results.extend(contact_results);

    // If FTS5 returned nothing, fall back to LIKE search on contacts.
    if results.is_empty() {
        let fallback = search_contacts_like(conn, &like_pattern, clamped_limit)?;
        results.extend(fallback);
    }

    // ── Deals (LIKE) ─────────────────────────────────────────────────────────
    let deal_results = search_deals_like(conn, &like_pattern, clamped_limit)?;
    results.extend(deal_results);

    // ── Activities (LIKE) ────────────────────────────────────────────────────
    let activity_results = search_activities_like(conn, &like_pattern, clamped_limit)?;
    results.extend(activity_results);

    // Truncate to global limit.
    results.truncate(clamped_limit as usize);

    log::debug!(
        "unified_search query='{}' -> {} results",
        q,
        results.len()
    );

    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal search helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Searches contacts via FTS5.
fn search_contacts_fts(
    conn: &Connection,
    fts_query: &str,
    limit: i64,
) -> CrmResult<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT c.id,
               c.first_name || ' ' || c.last_name AS full_name,
               c.email,
               c.org_name
        FROM contacts c
        INNER JOIN contacts_fts fts ON c.rowid = fts.rowid
        WHERE contacts_fts MATCH ?1 AND c.deleted_at IS NULL
        ORDER BY rank
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        let id: String = row.get(0)?;
        let full_name: String = row.get(1)?;
        let email: String = row.get(2)?;
        let org_name: String = row.get(3)?;
        Ok((id, full_name, email, org_name))
    })?;

    let mut results = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, full_name, email, org_name) = row;
        let subtitle = if !email.is_empty() { email.clone() } else { org_name };
        results.push(SearchResult {
            entity_type: "contact".to_string(),
            entity_id: id,
            title: full_name.trim().to_string(),
            subtitle,
            match_field: "fts".to_string(),
        });
    }
    Ok(results)
}

/// Fallback LIKE search on contacts.
fn search_contacts_like(
    conn: &Connection,
    like_pattern: &str,
    limit: i64,
) -> CrmResult<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id,
               first_name || ' ' || last_name AS full_name,
               email,
               org_name
        FROM contacts
        WHERE deleted_at IS NULL
          AND (first_name LIKE ?1 OR last_name LIKE ?1 OR email LIKE ?1
               OR org_name LIKE ?1 OR phone LIKE ?1)
        ORDER BY first_name ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![like_pattern, limit], |row| {
        let id: String = row.get(0)?;
        let full_name: String = row.get(1)?;
        let email: String = row.get(2)?;
        let org_name: String = row.get(3)?;
        Ok((id, full_name, email, org_name))
    })?;

    let mut results = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, full_name, email, org_name) = row;
        let subtitle = if !email.is_empty() { email } else { org_name };
        results.push(SearchResult {
            entity_type: "contact".to_string(),
            entity_id: id,
            title: full_name.trim().to_string(),
            subtitle,
            match_field: "name_or_email".to_string(),
        });
    }
    Ok(results)
}

/// LIKE search on deals.
fn search_deals_like(
    conn: &Connection,
    like_pattern: &str,
    limit: i64,
) -> CrmResult<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, stage, value, currency
        FROM deals
        WHERE deleted_at IS NULL
          AND (title LIKE ?1 OR notes LIKE ?1)
        ORDER BY updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![like_pattern, limit], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let stage: String = row.get(2)?;
        let value: f64 = row.get(3)?;
        let currency: String = row.get(4)?;
        Ok((id, title, stage, value, currency))
    })?;

    let mut results = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, title, stage, value, currency) = row;
        results.push(SearchResult {
            entity_type: "deal".to_string(),
            entity_id: id,
            title,
            subtitle: format!("{} — {:.0} {}", stage, value, currency),
            match_field: "title".to_string(),
        });
    }
    Ok(results)
}

/// LIKE search on activities.
fn search_activities_like(
    conn: &Connection,
    like_pattern: &str,
    limit: i64,
) -> CrmResult<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, activity_type, due_date
        FROM activities
        WHERE deleted_at IS NULL
          AND (title LIKE ?1 OR description LIKE ?1)
        ORDER BY due_date ASC NULLS LAST
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![like_pattern, limit], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let activity_type: String = row.get(2)?;
        let due_date: Option<String> = row.get(3)?;
        Ok((id, title, activity_type, due_date))
    })?;

    let mut results = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, title, activity_type, due_date) = row;
        let subtitle = match due_date {
            Some(d) => format!("{} — due {}", activity_type, &d[..10.min(d.len())]),
            None => activity_type,
        };
        results.push(SearchResult {
            entity_type: "activity".to_string(),
            entity_id: id,
            title,
            subtitle,
            match_field: "title".to_string(),
        });
    }
    Ok(results)
}
