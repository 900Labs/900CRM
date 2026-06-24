//! Tag CRUD and entity-tagging operations for 900CRM.
//!
//! Tags are reusable color-labeled taxonomy labels that can be applied to any
//! entity type (`contact`, `organization`, `deal`, `activity`). The legacy
//! many-to-many relationship is stored in `entity_tags`; the target schema also
//! stores mirrored active links in `tag_links`.
//!
//! # Tag Design
//!
//! - Tag names are unique (enforced by `UNIQUE` constraint).
//! - Colors are stored as CSS hex strings (e.g. `"#6366f1"`).
//! - The legacy `tags.color` column is `NOT NULL`; explicit color clears
//!   reset to the default tag color.
//! - Tags are soft-deleted where the target compatibility columns are present.
//! - Legacy `entity_tags` links are physically deleted on remove; target
//!   `tag_links` rows are soft-deleted.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

pub const DEFAULT_TAG_COLOR: &str = "#6366f1";

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

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
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
pub fn create_tag(conn: &Connection, name: &str, color: &str, device_id: &str) -> CrmResult<Tag> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO tags (id, name, color, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![id, name, color, now, now, device_id],
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
        r#"
        SELECT id, name, color, created_at, COALESCE(updated_at, created_at),
               deleted_at, COALESCE(device_id, '')
        FROM tags
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
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
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, color, created_at, COALESCE(updated_at, created_at),
               deleted_at, COALESCE(device_id, '')
        FROM tags
        WHERE deleted_at IS NULL
        ORDER BY name ASC
        "#,
    )?;

    let rows = stmt.query_map([], row_to_tag)?;
    let tags: Vec<Tag> = rows.filter_map(|r| r.ok()).collect();

    log::debug!("list_tags: {} results", tags.len());
    Ok(tags)
}

/// Updates a tag's mutable fields.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Tag does not exist or is soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn update_tag(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    color: Option<&str>,
) -> CrmResult<Tag> {
    let current = get_tag(conn, id)?;
    let now = now_iso8601();
    let changed = conn
        .execute(
            r#"
            UPDATE tags
            SET name = ?1, color = ?2, updated_at = ?3
            WHERE id = ?4 AND deleted_at IS NULL
            "#,
            params![
                name.unwrap_or(&current.name),
                color.unwrap_or(&current.color),
                now,
                id
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                CrmError::InvalidInput(format!(
                    "Tag '{}' already exists",
                    name.unwrap_or(&current.name)
                ))
            } else {
                CrmError::Database(e.to_string())
            }
        })?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Tag '{}' not found", id)));
    }

    log::debug!("Updated tag id={}", id);
    get_tag(conn, id)
}

/// Soft-deletes a tag and hides its active target `tag_links`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Tag does not exist.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_tag(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE tags SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Tag '{}' not found", id)));
    }

    conn.execute("DELETE FROM entity_tags WHERE tag_id = ?1", params![id])?;
    conn.execute(
        "UPDATE tag_links SET deleted_at = ?1 WHERE tag_id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    log::info!("Soft-deleted tag id={}", id);
    Ok(())
}

/// Backward-compatible alias for callers still named around deletion.
pub fn delete_tag(conn: &Connection, id: &str) -> CrmResult<()> {
    soft_delete_tag(conn, id)
}

// ─────────────────────────────────────────────────────────────────────────────
// Entity tagging
// ─────────────────────────────────────────────────────────────────────────────

/// Applies a tag to an entity.
///
/// Idempotent: if the tag is already applied, returns `Ok(false)` silently.
///
/// # Parameters
///
/// - `entity_type` — `"contact"`, `"organization"`, `"deal"`, or `"activity"`.
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
    device_id: &str,
) -> CrmResult<bool> {
    let link_id = new_uuid();
    let now = now_iso8601();

    let legacy_changed = conn
        .execute(
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

    let target_changed = conn.execute(
        r#"
        INSERT INTO tag_links (id, tag_id, entity_type, entity_id, created_at, device_id)
        SELECT ?1, ?2, ?3, ?4, ?5, ?6
        WHERE NOT EXISTS (
            SELECT 1
            FROM tag_links
            WHERE tag_id = ?2
              AND entity_type = ?3
              AND entity_id = ?4
              AND deleted_at IS NULL
        )
        "#,
        params![link_id, tag_id, entity_type, entity_id, now, device_id],
    )?;

    log::debug!("Added tag {} to {}:{}", tag_id, entity_type, entity_id);
    Ok(legacy_changed > 0 || target_changed > 0)
}

/// Removes a tag from an entity.
///
/// Idempotent: if the tag is not applied, returns `Ok(false)` silently.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn remove_tag_from_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    tag_id: &str,
) -> CrmResult<bool> {
    let legacy_changed = conn.execute(
        "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2 AND tag_id = ?3",
        params![entity_type, entity_id, tag_id],
    )?;
    let target_changed = conn.execute(
        r#"
        UPDATE tag_links
        SET deleted_at = ?4
        WHERE entity_type = ?1
          AND entity_id = ?2
          AND tag_id = ?3
          AND deleted_at IS NULL
        "#,
        params![entity_type, entity_id, tag_id, now_iso8601()],
    )?;

    log::debug!("Removed tag {} from {}:{}", tag_id, entity_type, entity_id);
    Ok(legacy_changed > 0 || target_changed > 0)
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
        SELECT DISTINCT t.id, t.name, t.color, t.created_at,
               COALESCE(t.updated_at, t.created_at), t.deleted_at, COALESCE(t.device_id, '')
        FROM tags t
        WHERE t.deleted_at IS NULL
          AND (
            EXISTS (
                SELECT 1
                FROM entity_tags et
                WHERE et.tag_id = t.id
                  AND et.entity_type = ?1
                  AND et.entity_id = ?2
            )
            OR EXISTS (
                SELECT 1
                FROM tag_links tl
                WHERE tl.tag_id = t.id
                  AND tl.entity_type = ?1
                  AND tl.entity_id = ?2
                  AND tl.deleted_at IS NULL
            )
          )
        ORDER BY t.name ASC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], row_to_tag)?;
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
        updated_at: row.get(4)?,
        deleted_at: row.get(5)?,
        device_id: row.get(6)?,
    })
}
