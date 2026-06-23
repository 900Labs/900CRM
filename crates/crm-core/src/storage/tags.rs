//! Tag CRUD and entity-tagging operations for 900CRM.
//!
//! Tags are reusable color-labeled taxonomy labels that can be applied to any
//! entity type (`contact`, `deal`, `activity`). The many-to-many relationship
//! is stored in the `entity_tags` join table.
//!
//! # Tag Design
//!
//! - Tag names are unique (enforced by `UNIQUE` constraint).
//! - Colors are stored as CSS hex strings (e.g. `"#6366f1"`).
//! - Entity tags are physically deleted (no soft-delete) to keep the join table
//!   small and free of accumulated history.

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

/// A taxonomy label with a color, applicable to any CRM entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// UUID v4 primary key.
    pub id: String,

    /// Unique human-readable label (e.g. `"Hot Lead"`, `"VIP"`).
    pub name: String,

    /// CSS color string (e.g. `"#6366f1"`, `"#ef4444"`).
    pub color: String,

    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tag CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new tag.
///
/// Tag names are case-sensitive and must be unique.
///
/// # Errors
///
/// - [`CrmError::Database`] — Duplicate tag name or SQL failure.
pub fn create_tag(conn: &Connection, name: &str, color: &str) -> CrmResult<Tag> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, color, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            CrmError::InvalidInput(format!("Tag '{}' already exists", name))
        } else {
            CrmError::Database(e.to_string())
        }
    })?;

    log::debug!("Created tag id={} name={}", id, name);
    get_tag(conn, &id)
}

/// Retrieves a single tag by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Tag does not exist.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_tag(conn: &Connection, id: &str) -> CrmResult<Tag> {
    conn.query_row(
        "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
        params![id],
        row_to_tag,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Tag '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Returns all tags ordered alphabetically by name.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_tags(conn: &Connection) -> CrmResult<Vec<Tag>> {
    let mut stmt =
        conn.prepare("SELECT id, name, color, created_at FROM tags ORDER BY name ASC")?;

    let rows = stmt.query_map([], |row| row_to_tag(row))?;
    let tags: Vec<Tag> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_tags: {} results", tags.len());
    Ok(tags)
}

/// Permanently deletes a tag and all its `entity_tags` references.
///
/// The `ON DELETE CASCADE` on `entity_tags.tag_id` handles the join rows.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Tag does not exist.
/// - [`CrmError::Database`] — SQL failure.
pub fn delete_tag(conn: &Connection, id: &str) -> CrmResult<()> {
    let changed = conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Tag '{}' not found", id)));
    }

    log::info!("Deleted tag id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Entity tagging
// ─────────────────────────────────────────────────────────────────────────────

/// Applies a tag to an entity.
///
/// Idempotent: if the tag is already applied, returns `Ok(())` silently.
///
/// # Parameters
///
/// - `entity_type` — `"contact"`, `"deal"`, or `"activity"`.
/// - `entity_id` — UUID of the entity.
/// - `tag_id` — UUID of the tag to apply.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Tag does not exist.
/// - [`CrmError::Database`] — SQL failure.
pub fn add_tag_to_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    tag_id: &str,
) -> CrmResult<()> {
    conn.execute(
        r#"
        INSERT OR IGNORE INTO entity_tags (entity_type, entity_id, tag_id)
        VALUES (?1, ?2, ?3)
        "#,
        params![entity_type, entity_id, tag_id],
    )
    .map_err(|e| {
        if e.to_string().contains("FOREIGN KEY") {
            CrmError::NotFound(format!("Tag '{}' does not exist", tag_id))
        } else {
            CrmError::Database(e.to_string())
        }
    })?;

    log::debug!("Added tag {} to {}:{}", tag_id, entity_type, entity_id);
    Ok(())
}

/// Removes a tag from an entity.
///
/// Idempotent: if the tag is not applied, returns `Ok(())` silently.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn remove_tag_from_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    tag_id: &str,
) -> CrmResult<()> {
    conn.execute(
        "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2 AND tag_id = ?3",
        params![entity_type, entity_id, tag_id],
    )?;

    log::debug!("Removed tag {} from {}:{}", tag_id, entity_type, entity_id);
    Ok(())
}

/// Returns all tags applied to the specified entity.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_tags_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT t.id, t.name, t.color, t.created_at
        FROM tags t
        INNER JOIN entity_tags et ON t.id = et.tag_id
        WHERE et.entity_type = ?1 AND et.entity_id = ?2
        ORDER BY t.name ASC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], |row| row_to_tag(row))?;
    let tags: Vec<Tag> = rows.filter_map(|r| r.ok()).collect();

    Ok(tags)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to a [`Tag`].
fn row_to_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
    })
}
