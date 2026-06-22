use rusqlite::params;

use super::CrmCore;
use crate::utils::uuid::new_uuid;

fn open_test_core() -> (CrmCore, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("900crm-core-test-{}", new_uuid()));
    let core = CrmCore::open(&path).expect("test core should open");
    (core, path)
}

fn count(core: &CrmCore, sql: &str) -> i64 {
    core.db
        .conn
        .query_row(sql, [], |row| row.get(0))
        .expect("count query should succeed")
}

#[test]
fn create_contact_writes_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            Some("amina@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            Some("Imported from field office".to_string()),
        )
        .expect("contact should be created");

    let contact_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1 AND deleted_at IS NULL",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("contact count should query");
    assert_eq!(contact_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'contact' AND entity_id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'contact' AND entity_id = ?1 AND field_name = '__create__'",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_proposed_action_writes_proposed_action_and_audit() {
    let (mut core, path) = open_test_core();

    let proposed_action = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Follow up"}"#.to_string(),
            None,
        )
        .expect("proposed action should be created");

    let proposed_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM proposed_actions WHERE id = ?1 AND status = 'pending'",
            params![proposed_action.id],
            |row| row.get(0),
        )
        .expect("proposed action count should query");
    assert_eq!(proposed_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'propose_action' AND entity_type = 'proposed_action' AND entity_id = ?1",
            params![proposed_action.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_organization_writes_organization_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            None,
            None,
            None,
            Some("Lagos".to_string()),
            None,
            Some("NG".to_string()),
            None,
            Some("Regional partner".to_string()),
        )
        .expect("organization should be created");

    let organization_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM organizations WHERE id = ?1 AND deleted_at IS NULL",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("organization count should query");
    assert_eq!(organization_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = '__create__'",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_organization_can_clear_optional_fields() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            Some("https://acme.example".to_string()),
            Some("Dock 4".to_string()),
            Some("Suite 9".to_string()),
            Some("Lagos".to_string()),
            Some("Lagos State".to_string()),
            Some("NG".to_string()),
            Some("100001".to_string()),
            Some("Regional partner".to_string()),
        )
        .expect("organization should be created");

    let updated = core
        .update_organization(
            &organization.id,
            None,
            Some(None),
            Some(Some("".to_string())),
            Some(Some("   ".to_string())),
            Some(None),
            Some(Some("".to_string())),
            Some(Some(" ".to_string())),
            Some(None),
            Some(Some("".to_string())),
            Some(Some(" ".to_string())),
            Some(Some("".to_string())),
        )
        .expect("organization optional fields should clear");

    assert_eq!(updated.name, "Acme Health");
    assert_eq!(updated.email, None);
    assert_eq!(updated.phone, None);
    assert_eq!(updated.website, None);
    assert_eq!(updated.address_line1, None);
    assert_eq!(updated.address_line2, None);
    assert_eq!(updated.city, None);
    assert_eq!(updated.region, None);
    assert_eq!(updated.country, None);
    assert_eq!(updated.postal_code, None);
    assert_eq!(updated.description, None);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'update' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = '__update__'",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn link_contact_to_organization_writes_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let linked = core
        .link_contact_to_organization(&contact.id, Some(organization.id.clone()))
        .expect("contact should link to organization");
    assert_eq!(
        linked.organization_id.as_deref(),
        Some(organization.id.as_str())
    );
    assert_eq!(linked.org_id, None);
    assert_eq!(linked.org_name, "Acme Health");

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'contact' AND entity_id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'contact' AND entity_id = ?1 AND field_name = 'organization_id'",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_clients_default_to_disabled() {
    let (mut core, path) = open_test_core();

    let client = core
        .create_external_client_placeholder("Claude Desktop", "mcp")
        .expect("external client placeholder should be created");

    assert_eq!(client.permission_mode, "disabled");
    assert!(!client.enabled);

    let disabled_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM external_clients WHERE id = ?1 AND enabled = 0 AND permission_mode = 'disabled'",
            params![client.id],
            |row| row.get(0),
        )
        .expect("external client count should query");
    assert_eq!(disabled_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v3_bridges_legacy_organization_contacts_without_deleting_contacts() {
    let path = std::env::temp_dir().join(format!("900crm-bridge-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                contact_type    TEXT NOT NULL DEFAULT 'person',
                first_name      TEXT NOT NULL DEFAULT '',
                last_name       TEXT NOT NULL DEFAULT '',
                org_name        TEXT NOT NULL DEFAULT '',
                email           TEXT NOT NULL DEFAULT '',
                phone           TEXT NOT NULL DEFAULT '',
                address         TEXT NOT NULL DEFAULT '',
                city            TEXT NOT NULL DEFAULT '',
                country         TEXT NOT NULL DEFAULT '',
                org_id          TEXT,
                notes           TEXT NOT NULL DEFAULT '',
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL,
                deleted_at      TEXT,
                device_id       TEXT NOT NULL DEFAULT '',
                organization_id TEXT
            );
            CREATE TABLE organizations (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                email         TEXT,
                phone         TEXT,
                website       TEXT,
                address_line1 TEXT,
                address_line2 TEXT,
                city          TEXT,
                region        TEXT,
                country       TEXT,
                postal_code   TEXT,
                source        TEXT,
                description   TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL
            );
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT INTO contacts
                (id, contact_type, first_name, last_name, org_name, email, phone,
                 address, city, country, org_id, notes, created_at, updated_at, device_id)
            VALUES
                ('legacy-org', 'organization', '', '', 'Legacy Org', 'org@example.com', '',
                 '1 Main', 'Paris', 'FR', NULL, 'Legacy note', '2026-01-01', '2026-01-02', 'device-a'),
                ('person-1', 'person', 'Amina', 'Diallo', 'Legacy Org', '', '',
                 '', '', '', 'legacy-org', '', '2026-01-03', '2026-01-04', 'device-a');
            PRAGMA user_version = 2;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v3 bridge");

    let bridged_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM organizations WHERE id = 'legacy-org' AND name = 'Legacy Org' AND source = 'legacy_contact'",
            [],
            |row| row.get(0),
        )
        .expect("bridged organization count should query");
    assert_eq!(bridged_count, 1);

    let linked_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = 'person-1' AND org_id = 'legacy-org' AND organization_id = 'legacy-org'",
            [],
            |row| row.get(0),
        )
        .expect("linked contact count should query");
    assert_eq!(linked_count, 1);

    let legacy_contact_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = 'legacy-org' AND contact_type = 'organization'",
            [],
            |row| row.get(0),
        )
        .expect("legacy contact count should query");
    assert_eq!(legacy_contact_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v2_creates_required_readiness_tables() {
    let (core, path) = open_test_core();

    for table in [
        "organizations",
        "audit_log",
        "sync_changelog",
        "external_clients",
        "external_client_permissions",
        "proposed_actions",
    ] {
        let sql = format!(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{}'",
            table
        );
        assert_eq!(count(&core, &sql), 1, "missing table {table}");
    }

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('contacts') WHERE name = 'organization_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('sync_changelog') WHERE name = 'operation'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}
