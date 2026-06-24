use rusqlite::params;

use super::{CrmCore, TagColorUpdate};
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
fn corrupt_backup_database_is_rejected_by_integrity_validation() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    std::fs::write(&backup.database_path, b"not a sqlite database")
        .expect("backup database should be overwritten with corrupt bytes");

    let err = core
        .validate_local_backup(&backup_dir)
        .expect_err("corrupt backup should be rejected");
    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(
                message.contains("invalid") || message.contains("integrity check failed"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let leftover_validation_files = std::fs::read_dir(&backup_dir)
        .expect("backup dir should be readable")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("integrity-check")
        })
        .count();
    assert_eq!(leftover_validation_files, 0);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn failed_confirmed_restore_removes_temporary_restore_file() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    core.create_local_backup(&backup_dir)
        .expect("backup should be created");

    let restore_dir = path.join("restore-target");
    let conflicting_database_dir = restore_dir.join("900crm.db");
    std::fs::create_dir_all(&conflicting_database_dir)
        .expect("conflicting database directory should be created");

    let err = CrmCore::restore_local_backup_to_app_data(&restore_dir, &backup_dir, true)
        .expect_err("restore should fail when target database path is a directory");
    match err {
        crate::utils::errors::CrmError::Io(_) => {}
        other => panic!("expected Io restore failure, got {other:?}"),
    }

    assert!(
        !restore_dir.join("900crm.db.restore_tmp").exists(),
        "failed restore should remove temporary database copy"
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
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
fn deal_contact_link_unlink_writes_audit_sync_and_updates_legacy_mirror() {
    let (mut core, path) = open_test_core();

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
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    let deal_contact = core
        .add_deal_contact(
            &deal.id,
            &contact.id,
            Some(" Decision maker ".to_string()),
            true,
        )
        .expect("deal contact should link");
    assert_eq!(deal_contact.deal_id, deal.id);
    assert_eq!(deal_contact.contact_id, contact.id);
    assert_eq!(deal_contact.role.as_deref(), Some("Decision maker"));
    assert!(deal_contact.is_primary);
    assert_eq!(
        core.get_deal(&deal.id)
            .expect("deal should load")
            .contact_id
            .as_deref(),
        Some(contact.id.as_str())
    );

    let active_links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(active_links.len(), 1);

    let link_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_contact' AND entity_type = 'deal_contact' AND entity_id = ?1",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("link audit count should query");
    assert_eq!(link_audit_count, 1);

    let link_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND entity_id = ?1 AND field_name = '__create__'",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("link sync count should query");
    assert_eq!(link_sync_count, 1);

    let deal_contact_mirror_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'contact_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal mirror sync count should query");
    assert_eq!(deal_contact_mirror_sync_count, 1);

    let removed = core
        .remove_deal_contact(&deal.id, &contact.id)
        .expect("deal contact should unlink");
    assert_eq!(removed.id, deal_contact.id);
    assert!(removed.deleted_at.is_some());
    assert_eq!(
        core.get_deal(&deal.id)
            .expect("deal should load after unlink")
            .contact_id,
        None
    );
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list after unlink")
        .is_empty());

    let unlink_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_contact' AND entity_type = 'deal_contact' AND entity_id = ?1",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("unlink audit count should query");
    assert_eq!(unlink_audit_count, 1);

    let unlink_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND entity_id = ?1 AND field_name = '__delete__'",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("unlink sync count should query");
    assert_eq!(unlink_sync_count, 1);

    let deal_contact_mirror_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'contact_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal mirror sync count should query after unlink");
    assert_eq!(deal_contact_mirror_sync_count, 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_deal_with_contact_id_creates_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

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

    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(contact.id.clone()),
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    assert_eq!(deal.contact_id.as_deref(), Some(contact.id.as_str()));
    let links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].contact_id, contact.id);
    assert!(links[0].is_primary);

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_contact' AND entity_type = 'deal_contact'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__create__'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_changing_contact_id_updates_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let first_contact = core
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
        .expect("first contact should be created");
    let second_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Luis".to_string()),
            Some("Rivera".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(first_contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let updated = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some(second_contact.id.clone())),
            None,
            None,
        )
        .expect("deal contact should update");

    assert_eq!(
        updated.contact_id.as_deref(),
        Some(second_contact.id.as_str())
    );
    let links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(
        links
            .iter()
            .filter(|link| link.is_primary && link.contact_id == second_contact.id)
            .count(),
        1
    );
    assert!(links
        .iter()
        .any(|link| !link.is_primary && link.contact_id == first_contact.id));
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND field_name = 'contact_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__create__'"
        ),
        2
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = 'is_primary'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_clearing_contact_id_removes_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

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
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let updated = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            None,
            None,
        )
        .expect("deal contact should clear");

    assert_eq!(updated.contact_id, None);
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list")
        .is_empty());
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_contact' AND entity_type = 'deal_contact'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__delete__'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND field_name = 'contact_id'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_distinguishes_omitted_and_explicit_clears_for_legacy_fields() {
    let (mut core, path) = open_test_core();

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
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            Some("2026-07-15".to_string()),
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let unchanged = core
        .update_deal(
            &deal.id,
            Some("Renamed expansion".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should update without clearing omitted fields");
    assert_eq!(unchanged.expected_close.as_deref(), Some("2026-07-15"));
    assert_eq!(unchanged.contact_id.as_deref(), Some(contact.id.as_str()));
    assert_eq!(
        core.list_deal_contacts(&deal.id)
            .expect("deal contacts should list")
            .len(),
        1
    );

    let cleared_expected_close = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            None,
            None,
            None,
        )
        .expect("expected close should clear");
    assert_eq!(cleared_expected_close.expected_close, None);
    assert_eq!(
        cleared_expected_close.contact_id.as_deref(),
        Some(contact.id.as_str())
    );

    let cleared_contact = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some("   ".to_string())),
            None,
            None,
        )
        .expect("blank contact id should clear");
    assert_eq!(cleared_contact.contact_id, None);
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list after clear")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn deal_organization_link_unlink_writes_audit_and_sync() {
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
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    let linked = core
        .link_deal_to_organization(&deal.id, Some(organization.id.clone()))
        .expect("deal should link to organization");
    assert_eq!(
        linked.organization_id.as_deref(),
        Some(organization.id.as_str())
    );

    let unlinked = core
        .link_deal_to_organization(&deal.id, None)
        .expect("deal should unlink from organization");
    assert_eq!(unlinked.organization_id, None);

    let link_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'deal' AND entity_id = ?1",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("link organization audit count should query");
    assert_eq!(link_audit_count, 1);

    let unlink_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_organization' AND entity_type = 'deal' AND entity_id = ?1",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("unlink organization audit count should query");
    assert_eq!(unlink_audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'organization_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal organization sync count should query");
    assert_eq!(sync_count, 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn note_lifecycle_writes_note_audit_sync_and_soft_deletes() {
    let (mut core, path) = open_test_core();

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

    let note = core
        .create_note(
            "contact".to_string(),
            contact.id.clone(),
            "Initial note".to_string(),
        )
        .expect("note should be created");
    assert_eq!(note.content, "Initial note");

    let created_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'note' AND entity_id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note create audit count should query");
    assert_eq!(created_audit_count, 1);

    let created_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'note' AND entity_id = ?1 AND field_name = '__create__'",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note create sync count should query");
    assert_eq!(created_sync_count, 1);

    let updated = core
        .update_note(&note.id, "Updated note".to_string())
        .expect("note should update");
    assert_eq!(updated.content, "Updated note");

    let body_content_pair: (String, String) = core
        .db
        .conn
        .query_row(
            "SELECT content, body FROM notes WHERE id = ?1",
            params![note.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("note content/body should query");
    assert_eq!(
        body_content_pair,
        ("Updated note".to_string(), "Updated note".to_string())
    );

    core.delete_note(&note.id).expect("note should soft delete");

    assert!(core.get_note(&note.id).is_err());
    assert!(core
        .list_notes_for_entity("contact".to_string(), contact.id.clone())
        .expect("contact notes should list")
        .is_empty());

    let deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note deleted count should query");
    assert_eq!(deleted_count, 1);

    let delete_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'delete' AND entity_type = 'note' AND entity_id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note delete audit count should query");
    assert_eq!(delete_audit_count, 1);

    let delete_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'note' AND entity_id = ?1 AND field_name = '__delete__'",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note delete sync count should query");
    assert_eq!(delete_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn notes_read_legacy_content_and_target_body_without_loss() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Compat".to_string()),
            Some("Tester".to_string()),
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

    let now = now_iso8601();
    core.db
        .conn
        .execute(
            r#"
            INSERT INTO notes
                (id, content, body, entity_type, entity_id, created_at, updated_at, device_id)
            VALUES
                ('legacy-content-note', 'Legacy content', NULL, 'contact', ?1, ?2, ?2, 'device-a')
            "#,
            params![contact.id, now],
        )
        .expect("legacy content note should insert");
    core.db
        .conn
        .execute(
            r#"
            INSERT INTO notes
                (id, content, body, entity_type, entity_id, created_at, updated_at, device_id)
            VALUES
                ('target-body-note', '', 'Target body', 'contact', ?1, ?2, ?2, 'device-a')
            "#,
            params![contact.id, now],
        )
        .expect("target body note should insert");

    let legacy = core
        .get_note("legacy-content-note")
        .expect("legacy content note should read");
    assert_eq!(legacy.content, "Legacy content");

    let target = core
        .get_note("target-body-note")
        .expect("target body note should read");
    assert_eq!(target.content, "Target body");

    let updated = core
        .update_note("target-body-note", "Unified body".to_string())
        .expect("target body note should update");
    assert_eq!(updated.content, "Unified body");

    let body_content_pair: (String, String) = core
        .db
        .conn
        .query_row(
            "SELECT content, body FROM notes WHERE id = 'target-body-note'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("updated content/body should query");
    assert_eq!(
        body_content_pair,
        ("Unified body".to_string(), "Unified body".to_string())
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_lifecycle_writes_tag_audit_sync_and_soft_deletes() {
    let (mut core, path) = open_test_core();

    let tag = core
        .create_tag("VIP".to_string(), Some("#ef4444".to_string()))
        .expect("tag should be created");
    assert_eq!(tag.name, "VIP");
    assert_eq!(tag.color, "#ef4444");

    let create_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'tag' AND entity_id = ?1",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag create audit count should query");
    assert_eq!(create_audit_count, 1);

    let create_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'tag' AND entity_id = ?1 AND field_name = '__create__'",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag create sync count should query");
    assert_eq!(create_sync_count, 1);

    let updated = core
        .update_tag(
            &tag.id,
            Some("Priority".to_string()),
            Some(TagColorUpdate::Set("#0f766e".to_string())),
        )
        .expect("tag should update");
    assert_eq!(updated.name, "Priority");
    assert_eq!(updated.color, "#0f766e");

    core.delete_tag(&tag.id).expect("tag should soft delete");

    assert!(core.get_tag(&tag.id).is_err());
    assert!(core
        .list_tags()
        .expect("tags should list")
        .iter()
        .all(|listed| listed.id != tag.id));

    let deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag deleted count should query");
    assert_eq!(deleted_count, 1);

    let delete_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'tag' AND entity_id = ?1 AND field_name = '__delete__'",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag delete sync count should query");
    assert_eq!(delete_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_tag_distinguishes_omitted_color_from_explicit_reset() {
    let (mut core, path) = open_test_core();

    let tag = core
        .create_tag("Warm".to_string(), Some("#ef4444".to_string()))
        .expect("tag should be created");

    let renamed = core
        .update_tag(&tag.id, Some("Hot".to_string()), None)
        .expect("tag name should update without changing color");
    assert_eq!(renamed.name, "Hot");
    assert_eq!(renamed.color, "#ef4444");

    let reset_from_flag = core
        .update_tag(&tag.id, None, Some(TagColorUpdate::Reset))
        .expect("explicit reset should reset color");
    assert_eq!(
        reset_from_flag.color,
        crate::storage::tags::DEFAULT_TAG_COLOR
    );

    let recolored = core
        .update_tag(
            &tag.id,
            None,
            Some(TagColorUpdate::Set(" #0f766e ".to_string())),
        )
        .expect("explicit color should trim and update");
    assert_eq!(recolored.color, "#0f766e");

    let reset_from_blank = core
        .update_tag(&tag.id, None, Some(TagColorUpdate::Set("   ".to_string())))
        .expect("blank color should reset");
    assert_eq!(
        reset_from_blank.color,
        crate::storage::tags::DEFAULT_TAG_COLOR
    );

    let existing = core
        .create_tag("Existing".to_string(), None)
        .expect("second tag should be created");
    let err = core
        .update_tag(&tag.id, Some(existing.name), None)
        .expect_err("duplicate tag name should be rejected");
    assert!(err.to_string().contains("already exists"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_apply_remove_mirrors_legacy_and_target_links_with_audit_sync() {
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
    let tag = core
        .create_tag("Partner".to_string(), None)
        .expect("tag should be created");

    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("tag should apply to organization");

    let tags = core
        .list_tags_for_entity("organization".to_string(), organization.id.clone())
        .expect("organization tags should list");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, tag.id);

    let legacy_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags count should query");
    assert_eq!(legacy_link_count, 1);

    let target_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target tag_links count should query");
    assert_eq!(target_link_count, 1);

    let apply_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'apply_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("apply tag audit count should query");
    assert_eq!(apply_audit_count, 1);

    let apply_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND new_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("apply tag sync count should query");
    assert_eq!(apply_sync_count, 1);

    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("tag should remove from organization");

    assert!(core
        .list_tags_for_entity("organization".to_string(), organization.id.clone())
        .expect("organization tags should list after remove")
        .is_empty());

    let legacy_removed_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags removed count should query");
    assert_eq!(legacy_removed_count, 0);

    let target_active_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target active tag_links count should query");
    assert_eq!(target_active_count, 0);

    let target_soft_deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NOT NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target soft-deleted tag_links count should query");
    assert_eq!(target_soft_deleted_count, 1);

    let remove_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'remove_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("remove tag audit count should query");
    assert_eq!(remove_audit_count, 1);

    let remove_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND old_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("remove tag sync count should query");
    assert_eq!(remove_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_tag_apply_and_remove_do_not_record_extra_audit_or_sync_entries() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Noiseless Links".to_string(),
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
    let tag = core
        .create_tag("Quiet".to_string(), None)
        .expect("tag should be created");

    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("first apply should create link");
    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("duplicate apply should be idempotent");

    let legacy_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags count should query");
    assert_eq!(legacy_link_count, 1);

    let target_active_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target active tag_links count should query");
    assert_eq!(target_active_count, 1);

    let apply_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'apply_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("apply tag audit count should query");
    assert_eq!(apply_audit_count, 1);

    let apply_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND new_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("apply tag sync count should query");
    assert_eq!(apply_sync_count, 1);

    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("first remove should remove link");
    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("duplicate remove should be idempotent");

    let target_soft_deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NOT NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target soft-deleted tag_links count should query");
    assert_eq!(target_soft_deleted_count, 1);

    let remove_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'remove_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("remove tag audit count should query");
    assert_eq!(remove_audit_count, 1);

    let remove_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND old_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("remove tag sync count should query");
    assert_eq!(remove_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn notes_and_tags_reject_unknown_entity_references() {
    let (mut core, path) = open_test_core();

    assert!(core
        .create_note(
            "invoice".to_string(),
            "entity-1".to_string(),
            "Unsupported".to_string(),
        )
        .is_err());

    assert!(core
        .create_note(
            "contact".to_string(),
            "missing-contact".to_string(),
            "Missing".to_string(),
        )
        .is_err());

    let tag = core
        .create_tag("Follow-up".to_string(), None)
        .expect("tag should be created");
    assert!(core
        .apply_tag_to_entity(
            "deal".to_string(),
            "missing-deal".to_string(),
            tag.id.clone(),
        )
        .is_err());

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
fn migration_v7_creates_deal_relationship_schema() {
    let path = std::env::temp_dir().join(format!("900crm-v7-schema-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v7");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('deals') WHERE name = 'organization_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'deal_contacts'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_deal_contacts_active_primary'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_backfills_legacy_deal_contact_and_organization_link() {
    let path = std::env::temp_dir().join(format!("900crm-v7-backfill-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO organizations (id, deleted_at)
            VALUES ('org-1', NULL);
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES ('contact-1', 'org-1', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES ('deal-1', 'contact-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and backfill v7");

    let backfilled_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM deal_contacts WHERE deal_id = 'deal-1' AND contact_id = 'contact-1' AND is_primary = 1 AND deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("deal contact backfill count should query");
    assert_eq!(backfilled_link_count, 1);

    let organization_id: Option<String> = core
        .db
        .conn
        .query_row(
            "SELECT organization_id FROM deals WHERE id = 'deal-1'",
            [],
            |row| row.get(0),
        )
        .expect("deal organization backfill should query");
    assert_eq!(organization_id.as_deref(), Some("org-1"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_skips_deal_organization_backfill_for_missing_or_deleted_organizations() {
    let path = std::env::temp_dir().join(format!("900crm-v7-invalid-org-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO organizations (id, deleted_at)
            VALUES ('deleted-org', '2026-06-24T08:30:00Z');
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES
                ('missing-org-contact', 'missing-org', NULL),
                ('deleted-org-contact', 'deleted-org', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('missing-org-deal', 'missing-org-contact', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-org-deal', 'deleted-org-contact', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v7");

    let dangling_deal_org_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM deals WHERE organization_id IS NOT NULL AND organization_id <> ''",
            [],
            |row| row.get(0),
        )
        .expect("deal organization count should query");
    assert_eq!(dangling_deal_org_count, 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM deal_contacts WHERE is_primary = 1 AND deleted_at IS NULL"
        ),
        2
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_skips_deal_organization_backfill_when_organizations_table_is_missing() {
    let path = std::env::temp_dir().join(format!("900crm-v7-no-org-table-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES ('contact-1', 'missing-org', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES ('deal-1', 'contact-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and skip org backfill safely");

    let organization_id: Option<String> = core
        .db
        .conn
        .query_row(
            "SELECT organization_id FROM deals WHERE id = 'deal-1'",
            [],
            |row| row.get(0),
        )
        .expect("deal organization should query");
    assert_eq!(organization_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM deal_contacts WHERE deal_id = 'deal-1' AND contact_id = 'contact-1' AND is_primary = 1 AND deleted_at IS NULL"
        ),
        1
    );

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
fn normalization_migration_preflight_reports_clean_migrated_database() {
    let (core, path) = open_test_core();

    let report = core
        .normalization_migration_preflight()
        .expect("preflight report should be generated");

    assert_eq!(report.legacy_organization_contacts, 0);
    assert_eq!(report.contacts_with_org_id_missing_organization_id, 0);
    assert_eq!(report.contacts_with_invalid_organization_links, 0);
    assert!(report.backup_restore_baseline_available);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn normalization_migration_preflight_counts_legacy_and_invalid_links() {
    let (core, path) = open_test_core();

    let now = now_iso8601();
    core.db
        .conn
        .execute_batch(&format!(
            r#"
            INSERT INTO contacts
                (id, contact_type, first_name, last_name, org_name, email, phone,
                 address, city, country, org_id, organization_id, notes,
                 created_at, updated_at, device_id)
            VALUES
                ('legacy-org', 'organization', '', '', 'Legacy Org', '', '',
                 '', '', '', NULL, NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-missing-normalized', 'person', 'Amina', 'Diallo', 'Legacy Org', '', '',
                 '', '', '', 'legacy-org', NULL, '', '{now}', '{now}', 'device-a'),
                ('person-as-org', 'person', 'Wrong', 'Type', '', '', '',
                 '', '', '', NULL, NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-invalid-org-id', 'person', 'Invalid', 'Legacy Link', '', '', '',
                 '', '', '', 'person-as-org', NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-invalid-organization-id', 'person', 'Invalid', 'Normalized Link', '', '', '',
                 '', '', '', NULL, 'missing-organization', '', '{now}', '{now}', 'device-a');
            "#
        ))
        .expect("test contacts should be inserted");

    let report = core
        .normalization_migration_preflight()
        .expect("preflight report should be generated");

    assert_eq!(report.legacy_organization_contacts, 1);
    assert_eq!(report.contacts_with_org_id_missing_organization_id, 2);
    assert_eq!(report.contacts_with_invalid_organization_links, 2);
    assert!(report.backup_restore_baseline_available);

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
