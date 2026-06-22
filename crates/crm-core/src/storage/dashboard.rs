use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::errors::CrmResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCounts {
    pub total_contacts: i64,
    pub total_organizations: i64,
    pub new_contacts_this_month: i64,
    pub new_deals_this_month: i64,
}

pub fn get_dashboard_counts(conn: &Connection, month_prefix: &str) -> CrmResult<DashboardCounts> {
    let total_contacts = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND contact_type = 'person'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_organizations = conn
        .query_row(
            r#"
            SELECT
                (SELECT COUNT(*) FROM contacts
                 WHERE deleted_at IS NULL AND contact_type = 'organization')
                +
                (SELECT COUNT(*) FROM organizations
                 WHERE deleted_at IS NULL)
            "#,
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_contacts_this_month = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND created_at LIKE ?1",
            params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_deals_this_month = conn
        .query_row(
            "SELECT COUNT(*) FROM deals WHERE deleted_at IS NULL AND created_at LIKE ?1",
            params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(DashboardCounts {
        total_contacts,
        total_organizations,
        new_contacts_this_month,
        new_deals_this_month,
    })
}
