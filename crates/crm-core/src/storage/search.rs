//! Repository queries for cross-entity search.

use rusqlite::{params, Connection};

use crate::utils::errors::CrmResult;

#[derive(Debug, Clone)]
pub struct ContactSearchRecord {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub org_name: String,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct DealSearchRecord {
    pub id: String,
    pub title: String,
    pub stage: String,
    pub value: f64,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct ActivitySearchRecord {
    pub id: String,
    pub title: String,
    pub activity_type: String,
    pub due_date: Option<String>,
}

pub fn search_contacts_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ContactSearchRecord>> {
    let fts_query = format!("{}*", query.trim());
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
        Ok(ContactSearchRecord {
            id: row.get(0)?,
            full_name: row.get(1)?,
            email: row.get(2)?,
            org_name: row.get(3)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn search_contacts_fallback(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ContactSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
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

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(ContactSearchRecord {
            id: row.get(0)?,
            full_name: row.get(1)?,
            email: row.get(2)?,
            org_name: row.get(3)?,
            match_field: "name_or_email".to_string(),
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn search_deals_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<DealSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
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

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(DealSearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            stage: row.get(2)?,
            value: row.get(3)?,
            currency: row.get(4)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn search_activities_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ActivitySearchRecord>> {
    let pattern = format!("%{}%", query.trim());
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

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(ActivitySearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            activity_type: row.get(2)?,
            due_date: row.get(3)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}
