//! Unified cross-entity search for 900CRM.
//!
//! This module provides the [`unified_search`] function that searches across
//! contacts, deals, and activities simultaneously and returns a combined list
//! of [`SearchResult`] items.
//!
//! # Search Strategy
//!
//! - Contacts use the storage full-text repository first.
//! - Deals use the storage text repository over title and notes.
//! - Activities use the storage text repository over title and description.
//!
//! Results from all three sources are combined, with contacts ranked first,
//! then deals, then activities.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::storage::search::{
    self as search_storage, ActivitySearchRecord, ContactSearchRecord, DealSearchRecord,
};
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
/// - `conn` — database connection.
/// - `query` — Search string. Trimmed before use.
/// - `limit` — Maximum number of results to return. Clamped to [1, 100].
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on storage failure.
pub fn unified_search(conn: &Connection, query: &str, limit: u32) -> CrmResult<Vec<SearchResult>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }

    let clamped_limit = limit.max(1).min(100) as i64;
    let mut results: Vec<SearchResult> = Vec::new();

    let contact_results = search_storage::search_contacts_full_text(conn, q, clamped_limit)?;
    results.extend(contact_results.into_iter().map(contact_record_to_result));

    if results.is_empty() {
        let fallback = search_storage::search_contacts_fallback(conn, q, clamped_limit)?;
        results.extend(fallback.into_iter().map(contact_record_to_result));
    }

    let deal_results = search_storage::search_deals_text(conn, q, clamped_limit)?;
    results.extend(deal_results.into_iter().map(deal_record_to_result));

    let activity_results = search_storage::search_activities_text(conn, q, clamped_limit)?;
    results.extend(activity_results.into_iter().map(activity_record_to_result));

    results.truncate(clamped_limit as usize);

    log::debug!("unified_search query='{}' -> {} results", q, results.len());

    Ok(results)
}

// ─────────────────────────────────────────────────────────────────────────────
// Result mapping
// ─────────────────────────────────────────────────────────────────────────────

fn contact_record_to_result(record: ContactSearchRecord) -> SearchResult {
    let subtitle = if !record.email.is_empty() {
        record.email
    } else {
        record.org_name
    };

    SearchResult {
        entity_type: "contact".to_string(),
        entity_id: record.id,
        title: record.full_name.trim().to_string(),
        subtitle,
        match_field: record.match_field,
    }
}

fn deal_record_to_result(record: DealSearchRecord) -> SearchResult {
    SearchResult {
        entity_type: "deal".to_string(),
        entity_id: record.id,
        title: record.title,
        subtitle: format!("{} — {:.0} {}", record.stage, record.value, record.currency),
        match_field: "title".to_string(),
    }
}

fn activity_record_to_result(record: ActivitySearchRecord) -> SearchResult {
    let subtitle = match record.due_date {
        Some(d) => format!("{} — due {}", record.activity_type, &d[..10.min(d.len())]),
        None => record.activity_type,
    };

    SearchResult {
        entity_type: "activity".to_string(),
        entity_id: record.id,
        title: record.title,
        subtitle,
        match_field: "title".to_string(),
    }
}
