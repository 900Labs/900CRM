use rusqlite::params;

use super::CrmCore;
use crate::utils::{datetime::now_iso8601, uuid::new_uuid};

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
fn create_local_backup_writes_snapshot_and_metadata() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Backup".to_string()),
            Some("Tester".to_string()),
            None,
            Some("backup@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created before backup");

    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    assert_eq!(backup.metadata.backup_format_version, 1);
    assert_eq!(backup.metadata.app_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(
        backup.metadata.schema_version,
        crate::storage::Database::current_schema_version()
    );
    assert_eq!(backup.metadata.device_id, core.device_id());
    assert_eq!(backup.metadata.database_file, "900crm.db");
    assert!(std::path::Path::new(&backup.database_path).is_file());
    assert!(std::path::Path::new(&backup.metadata_path).is_file());

    let backup_conn =
        rusqlite::Connection::open(&backup.database_path).expect("backup database should open");
    let backed_up_contact_count: i64 = backup_conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("backup contact count should query");
    assert_eq!(backed_up_contact_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn backup_metadata_validation_accepts_current_schema() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    let validation = core
        .validate_local_backup(&backup_dir)
        .expect("backup should validate");

    assert_eq!(validation.metadata, backup.metadata);
    assert_eq!(validation.database_path, backup.database_path);
    assert_eq!(validation.metadata_path, backup.metadata_path);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn restore_requires_explicit_confirmation() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    core.create_local_backup(&backup_dir)
        .expect("backup should be created");

    let restore_dir = path.join("restore-target");
    let err = CrmCore::restore_local_backup_to_app_data(&restore_dir, &backup_dir, false)
        .expect_err("restore should require explicit confirmation");

    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(message.contains("requires explicit confirmation"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
    assert!(!restore_dir.join("900crm.db").exists());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn invalid_restore_is_rejected_before_applying() {
    let (source_core, source_path) = open_test_core();
    let backup_dir = source_path.join("backup");
    let backup = source_core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    let mut metadata: super::backup::LocalBackupMetadata = serde_json::from_slice(
        &std::fs::read(&backup.metadata_path).expect("metadata should read"),
    )
    .expect("metadata should parse");
    metadata.schema_version = crate::storage::Database::current_schema_version() + 1;
    std::fs::write(
        &backup.metadata_path,
        serde_json::to_vec_pretty(&metadata).expect("metadata should serialize"),
    )
    .expect("metadata should be overwritten");

    let (mut target_core, target_path) = open_test_core();
    let existing = target_core
        .create_contact(
            Some("person".to_string()),
            Some("Existing".to_string()),
            Some("Target".to_string()),
            None,
            Some("existing@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("target contact should be created");
    drop(target_core);

    let err = CrmCore::restore_local_backup_to_app_data(&target_path, &backup_dir, true)
        .expect_err("future-schema backup should be rejected");
    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(message.contains("newer than supported schema version"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let target_core = CrmCore::open(&target_path).expect("target core should still open");
    let preserved_count: i64 = target_core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1",
            params![existing.id],
            |row| row.get(0),
        )
        .expect("preserved target contact count should query");
    assert_eq!(preserved_count, 1);

    drop(source_core);
    drop(target_core);
    let _ = std::fs::remove_dir_all(source_path);
    let _ = std::fs::remove_dir_all(target_path);
}

#[test]
fn activity_stats_query_counts_completed_overdue_and_due_today() {
    let (mut core, path) = open_test_core();
    let today = now_iso8601()[..10].to_string();

    core.create_activity(
        "task".to_string(),
        "Due today".to_string(),
        None,
        Some(format!("{today}T23:59:59Z")),
        None,
        None,
    )
    .expect("due-today activity should be created");
    core.create_activity(
        "task".to_string(),
        "Past due".to_string(),
        None,
        Some("2000-01-01T00:00:00Z".to_string()),
        None,
        None,
    )
    .expect("overdue activity should be created");
    let completed = core
        .create_activity(
            "task".to_string(),
            "Completed".to_string(),
            None,
            Some("2000-01-01T00:00:00Z".to_string()),
            None,
            None,
        )
        .expect("completed activity should be created");
    core.mark_activity_complete(&completed.id)
        .expect("activity should be marked complete");

    let stats = crate::crm_engine::activities::get_activity_stats(&core.db.conn)
        .expect("activity stats should query");
    assert_eq!(stats.total, 3);
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.overdue, 1);
    assert_eq!(stats.due_today, 1);
    assert_eq!(stats.pending, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_detection_uses_contact_repository_queries() {
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
            None,
        )
        .expect("contact should be created");

    let email_input = crate::crm_engine::contacts::ContactInput {
        contact_type: Some("person".to_string()),
        first_name: Some("Other".to_string()),
        last_name: Some("Person".to_string()),
        org_name: None,
        email: Some("AMINA@example.com".to_string()),
        phone: None,
        address: None,
        city: None,
        country: None,
        org_id: None,
        notes: None,
    };
    let email_matches =
        crate::crm_engine::contacts::find_duplicate_candidates(&core.db.conn, &email_input)
            .expect("email duplicate detection should query");
    assert_eq!(email_matches.len(), 1);
    assert_eq!(email_matches[0].contact.id, contact.id);
    assert_eq!(email_matches[0].score, 1.0);

    let name_input = crate::crm_engine::contacts::ContactInput {
        contact_type: Some("person".to_string()),
        first_name: Some("Amina".to_string()),
        last_name: Some("Diallo".to_string()),
        org_name: None,
        email: None,
        phone: None,
        address: None,
        city: None,
        country: None,
        org_id: None,
        notes: None,
    };
    let name_matches =
        crate::crm_engine::contacts::find_duplicate_candidates(&core.db.conn, &name_input)
            .expect("name duplicate detection should query");
    assert_eq!(name_matches.len(), 1);
    assert_eq!(name_matches[0].contact.id, contact.id);
    assert_eq!(name_matches[0].score, 0.9);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn pipeline_age_query_ignores_closed_deals() {
    let (mut core, path) = open_test_core();

    let active = core
        .create_deal(
            "Current active deal".to_string(),
            Some(100.0),
            None,
            Some("Lead".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("active deal should be created");
    let closed = core
        .create_deal(
            "Old closed deal".to_string(),
            Some(100.0),
            None,
            Some("Closed Won".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("closed deal should be created");

    core.db
        .conn
        .execute(
            "UPDATE deals SET created_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            params![closed.id],
        )
        .expect("closed deal timestamp should update");

    let age = crate::storage::deals::get_average_active_deal_age_days(&core.db.conn)
        .expect("average active deal age should query");
    assert!(
        age < 1.0,
        "closed deal age should not affect active average, got {age}"
    );

    let active_lookup = core
        .get_deal(&active.id)
        .expect("active deal should remain");
    assert_eq!(active_lookup.stage, "Lead");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn unified_search_uses_storage_repositories_for_all_entities() {
    let (mut core, path) = open_test_core();

    core.create_contact(
        Some("person".to_string()),
        Some("Clinic".to_string()),
        Some("Lead".to_string()),
        Some("Clinic Partners".to_string()),
        Some("clinic@example.com".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("contact should be created");
    core.create_deal(
        "Clinic expansion".to_string(),
        Some(2500.0),
        None,
        Some("Proposal".to_string()),
        None,
        None,
        None,
        Some("Expansion project".to_string()),
    )
    .expect("deal should be created");
    core.create_activity(
        "call".to_string(),
        "Call clinic".to_string(),
        Some("Discuss rollout".to_string()),
        None,
        None,
        None,
    )
    .expect("activity should be created");

    let results = crate::crm_engine::search::unified_search(&core.db.conn, "Clinic", 10)
        .expect("unified search should query");
    assert!(results.iter().any(|r| r.entity_type == "contact"));
    assert!(results.iter().any(|r| r.entity_type == "deal"));
    assert!(results.iter().any(|r| r.entity_type == "activity"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
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
