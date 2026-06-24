use rusqlite::params;

use super::{CrmCore, TagColorUpdate};
use crate::utils::{
    csv::ImportColumnMapping, datetime::now_iso8601, errors::CrmError, uuid::new_uuid,
};

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

fn import_mapping(pairs: &[(&str, Option<&str>)]) -> ImportColumnMapping {
    pairs
        .iter()
        .map(|(source, target)| ((*source).to_string(), target.map(str::to_string)))
        .collect()
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

fn seed_global_search_entities(core: &mut CrmCore) {
    let contact = core
        .create_contact(
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
    core.create_organization(
        "Clinic Partners".to_string(),
        Some("partners@clinic.example".to_string()),
        None,
        None,
        None,
        None,
        Some("Lagos".to_string()),
        None,
        Some("Nigeria".to_string()),
        None,
        Some("Regional clinic group".to_string()),
    )
    .expect("organization should be created");
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
    let tag = core
        .create_tag("Clinic Priority".to_string(), None)
        .expect("tag should be created");
    core.create_note(
        "contact".to_string(),
        contact.id,
        "Clinic onboarding note".to_string(),
    )
    .expect("note should be created");
    assert_eq!(tag.name, "Clinic Priority");
}

#[test]
fn global_search_returns_all_supported_entity_types() {
    let (mut core, path) = open_test_core();
    seed_global_search_entities(&mut core);

    let results = core
        .global_search("Clinic", Some(20))
        .expect("global search should query");
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Contact));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Organization));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Deal));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Activity));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Note));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Tag));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn global_search_respects_blank_query_and_limit_behavior() {
    let (mut core, path) = open_test_core();
    seed_global_search_entities(&mut core);

    let blank = core
        .global_search("   ", Some(20))
        .expect("blank global search should succeed");
    assert!(blank.is_empty());

    let zero = core
        .global_search("Clinic", Some(0))
        .expect("zero-limit global search should succeed");
    assert!(zero.is_empty());

    let limited = core
        .global_search("Clinic", Some(2))
        .expect("limited global search should succeed");
    assert_eq!(limited.len(), 2);

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
fn import_organizations_csv_creates_valid_rows_and_skips_blank_names() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("organizations.csv");
    std::fs::write(
        &csv_path,
        "name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description\n\
         Acme Health,hello@acme.example,+123456,https://acme.example,Dock 4,Suite 9,Lagos,Lagos State,NG,100001,Regional partner\n\
         ,blank@example.com,,,,,,,,,\n",
    )
    .expect("organization CSV fixture should write");

    let result = core
        .import_organizations_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let organizations = core
        .list_organizations()
        .expect("organizations should list after import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Acme Health");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@acme.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+123456"));
    assert_eq!(
        organizations[0].website.as_deref(),
        Some("https://acme.example")
    );
    assert_eq!(organizations[0].address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(organizations[0].address_line2.as_deref(), Some("Suite 9"));
    assert_eq!(organizations[0].city.as_deref(), Some("Lagos"));
    assert_eq!(organizations[0].region.as_deref(), Some("Lagos State"));
    assert_eq!(organizations[0].country.as_deref(), Some("NG"));
    assert_eq!(organizations[0].postal_code.as_deref(), Some("100001"));
    assert_eq!(
        organizations[0].description.as_deref(),
        Some("Regional partner")
    );

    let import_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row' AND entity_type = 'organization'",
            [],
            |row| row.get(0),
        )
        .expect("organization import audit count should query");
    assert_eq!(import_audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_csv_with_mapping_imports_nonstandard_headers() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("contacts-mapped.csv");
    std::fs::write(
        &csv_path,
        "Given,Family,Company,Mail,Tel,Street,Town,Nation,Memo,Skip\n\
         Mina,Okafor,Acme Health,mina@example.com,+234500,Dock 4,Lagos,NG,Primary buyer,ignored\n\
         ,Blank,Skipped,blank@example.com,,,,,,ignored\n",
    )
    .expect("mapped contact CSV fixture should write");
    let mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Family", Some("last_name")),
        ("Company", Some("org_name")),
        ("Mail", Some("email")),
        ("Tel", Some("phone")),
        ("Street", Some("address")),
        ("Town", Some("city")),
        ("Nation", Some("country")),
        ("Memo", Some("notes")),
        ("Skip", None),
    ]);

    let result = core
        .import_contacts_csv_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped contact CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let imported: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = core
        .db
        .conn
        .query_row(
            "SELECT first_name, last_name, org_name, email, phone, address, city, country, notes FROM contacts WHERE email = ?1",
            params!["mina@example.com"],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .expect("mapped contact should query");
    assert_eq!(
        imported,
        (
            "Mina".to_string(),
            "Okafor".to_string(),
            "Acme Health".to_string(),
            "mina@example.com".to_string(),
            "+234500".to_string(),
            "Dock 4".to_string(),
            "Lagos".to_string(),
            "NG".to_string(),
            "Primary buyer".to_string(),
        )
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_csv_with_mapping_imports_nonstandard_headers() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("organizations-mapped.csv");
    std::fs::write(
        &csv_path,
        "Company,Inbox,Telephone,Site,Line One,Line Two,Town,State,Nation,Postcode,About,Skip\n\
         Amani Labs,hello@amani.example,+254700,https://amani.example,Floor 2,Unit B,Nairobi,Nairobi County,KE,00100,Research partner,ignored\n\
         ,blank@example.com,,,,,,,,,,ignored\n",
    )
    .expect("mapped organization CSV fixture should write");
    let mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Inbox", Some("email")),
        ("Telephone", Some("phone")),
        ("Site", Some("website")),
        ("Line One", Some("address_line1")),
        ("Line Two", Some("address_line2")),
        ("Town", Some("city")),
        ("State", Some("region")),
        ("Nation", Some("country")),
        ("Postcode", Some("postal_code")),
        ("About", Some("description")),
        ("Skip", None),
    ]);

    let result = core
        .import_organizations_csv_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped organization CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let organizations = core
        .list_organizations()
        .expect("organizations should list after mapped import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Amani Labs");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@amani.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+254700"));
    assert_eq!(
        organizations[0].website.as_deref(),
        Some("https://amani.example")
    );
    assert_eq!(organizations[0].address_line1.as_deref(), Some("Floor 2"));
    assert_eq!(organizations[0].address_line2.as_deref(), Some("Unit B"));
    assert_eq!(organizations[0].city.as_deref(), Some("Nairobi"));
    assert_eq!(organizations[0].region.as_deref(), Some("Nairobi County"));
    assert_eq!(organizations[0].country.as_deref(), Some("KE"));
    assert_eq!(organizations[0].postal_code.as_deref(), Some("00100"));
    assert_eq!(
        organizations[0].description.as_deref(),
        Some("Research partner")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_organization_preflight_flags_name_email_and_phone_duplicates() {
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
        .expect("organization fixture should be created");

    let csv_path = path.join("organizations-mapped-preflight.csv");
    std::fs::write(
        &csv_path,
        "Company,Inbox,Telephone,Ignored\n\
         acme health,HELLO@acme.example,+123456,ignored\n",
    )
    .expect("mapped organization preflight CSV fixture should write");
    let mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Inbox", Some("email")),
        ("Telephone", Some("phone")),
        ("Ignored", None),
    ]);

    let report = core
        .preflight_organizations_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped organization preflight should succeed");

    assert_eq!(report.entity_type, "organizations");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 3);
    for match_type in ["name", "email", "phone"] {
        let warning = report
            .warnings
            .iter()
            .find(|warning| warning.match_type == match_type)
            .unwrap_or_else(|| panic!("{match_type} duplicate warning should exist"));
        assert_eq!(warning.row_number, 2);
        assert_eq!(warning.existing_entity_type, "organization");
        assert_eq!(warning.existing_entity_id, organization.id);
        assert_eq!(warning.existing_display_label, "Acme Health");
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_import_rejects_invalid_target_fields_and_duplicate_assignments() {
    let (core, path) = open_test_core();
    let csv_path = path.join("invalid-mapping.csv");
    std::fs::write(&csv_path, "A,B,C\none,two,three\n")
        .expect("invalid mapping CSV fixture should write");

    let invalid_target = import_mapping(&[("A", Some("nickname"))]);
    let err = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            invalid_target,
        )
        .expect_err("unknown mapped target should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Unknown import target field 'nickname'"));
        }
        other => panic!("expected InvalidInput for unknown target, got {other:?}"),
    }

    let duplicate_target = import_mapping(&[("A", Some("first_name")), ("B", Some("first_name"))]);
    let err = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            duplicate_target,
        )
        .expect_err("duplicate mapped target should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("mapped more than once"));
            assert!(message.contains("first_name"));
        }
        other => panic!("expected InvalidInput for duplicate target, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_contact_preflight_is_read_only() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact fixture should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("contacts-mapped-preflight.csv");
    std::fs::write(
        &csv_path,
        "Given,Mail,Telephone,Ignored\n\
         Imported,ADA@example.com,+15550100,ignored\n",
    )
    .expect("mapped contact preflight CSV fixture should write");
    let mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Mail", Some("email")),
        ("Telephone", Some("phone")),
        ("Ignored", None),
    ]);

    let report = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped contact preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 2);
    assert!(report
        .warnings
        .iter()
        .all(|warning| warning.existing_entity_id == contact.id));
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_contacts_csv_import_flags_email_and_phone_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let email_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("email fixture contact should be created");
    let phone_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Hopper".to_string()),
            None,
            None,
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("phone fixture contact should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("contacts-preflight.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email,phone\n\
         Imported,Email,ADA@example.com,\n\
         Imported,Phone,,+15550100\n",
    )
    .expect("contact preflight CSV fixture should write");

    let report = core
        .preflight_contacts_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 2);
    assert_eq!(report.duplicate_warning_count, 2);
    assert_eq!(report.warnings.len(), 2);

    let email_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "email")
        .expect("email duplicate warning should exist");
    assert_eq!(email_warning.row_number, 2);
    assert_eq!(email_warning.csv_value, "ADA@example.com");
    assert_eq!(email_warning.existing_entity_type, "contact");
    assert_eq!(email_warning.existing_entity_id, email_contact.id);
    assert_eq!(email_warning.existing_display_label, "Ada Lovelace");

    let phone_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "phone")
        .expect("phone duplicate warning should exist");
    assert_eq!(phone_warning.row_number, 3);
    assert_eq!(phone_warning.csv_value, "+15550100");
    assert_eq!(phone_warning.existing_entity_type, "contact");
    assert_eq!(phone_warning.existing_entity_id, phone_contact.id);
    assert_eq!(phone_warning.existing_display_label, "Grace Hopper");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_organizations_csv_import_flags_name_email_and_phone_duplicates_without_writes() {
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
        .expect("organization fixture should be created");

    let organization_count_before = count(&core, "SELECT COUNT(*) FROM organizations");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("organizations-preflight.csv");
    std::fs::write(
        &csv_path,
        "name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description\n\
         acme health,HELLO@acme.example,+123456,,,,,,,,\n",
    )
    .expect("organization preflight CSV fixture should write");

    let report = core
        .preflight_organizations_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization preflight should succeed");

    assert_eq!(report.entity_type, "organizations");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 3);
    assert_eq!(report.warnings.len(), 3);

    for match_type in ["name", "email", "phone"] {
        let warning = report
            .warnings
            .iter()
            .find(|warning| warning.match_type == match_type)
            .unwrap_or_else(|| panic!("{match_type} duplicate warning should exist"));
        assert_eq!(warning.row_number, 2);
        assert_eq!(warning.existing_entity_type, "organization");
        assert_eq!(warning.existing_entity_id, organization.id);
        assert_eq!(warning.existing_display_label, "Acme Health");
    }

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM organizations"),
        organization_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_organizations_csv_writes_optional_fields() {
    let (mut core, path) = open_test_core();

    core.create_organization(
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
    .expect("organization should be created before export");

    let csv_path = path.join("organizations-export.csv");
    let count = core
        .export_organizations_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization CSV export should succeed");

    assert_eq!(count, 1);
    let csv = std::fs::read_to_string(&csv_path).expect("organization CSV should read");
    assert!(csv.contains("name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description"));
    assert!(csv.contains("Acme Health"));
    assert!(csv.contains("hello@acme.example"));
    assert!(csv.contains("+123456"));
    assert!(csv.contains("https://acme.example"));
    assert!(csv.contains("Dock 4"));
    assert!(csv.contains("Suite 9"));
    assert!(csv.contains("Lagos State"));
    assert!(csv.contains("100001"));
    assert!(csv.contains("Regional partner"));

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
fn create_activity_with_contact_and_deal_creates_activity_links_audit_and_sync() {
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
            None,
        )
        .expect("deal should be created");

    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(contact.id.clone()),
            Some(deal.id.clone()),
        )
        .expect("activity should be created");

    assert_eq!(activity.contact_id.as_deref(), Some(contact.id.as_str()));
    assert_eq!(activity.deal_id.as_deref(), Some(deal.id.as_str()));
    let links = core
        .list_activity_links(&activity.id)
        .expect("activity links should list");
    assert_eq!(links.len(), 2);
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Contact
            && link.entity_id == contact.id
    }));
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Deal
            && link.entity_id == deal.id
    }));

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('link_contact', 'link_deal') AND entity_type = 'activity_link'"
        ),
        2
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__create__'"
        ),
        2
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_contact_and_deal_activity_links_do_not_rewrite_activity_or_audit_sync() {
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
            None,
        )
        .expect("deal should be created");
    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(contact.id.clone()),
            Some(deal.id.clone()),
        )
        .expect("activity should be created");

    let links_before = core
        .list_activity_links(&activity.id)
        .expect("activity links should list before duplicate adds");
    let contact_link_id = links_before
        .iter()
        .find(|link| {
            link.entity_type == crate::storage::activities::ActivityLinkEntityType::Contact
                && link.entity_id == contact.id
        })
        .expect("contact link should exist")
        .id
        .clone();
    let deal_link_id = links_before
        .iter()
        .find(|link| {
            link.entity_type == crate::storage::activities::ActivityLinkEntityType::Deal
                && link.entity_id == deal.id
        })
        .expect("deal link should exist")
        .id
        .clone();

    let sentinel_updated_at = "2026-06-24T09:15:00Z";
    core.db
        .conn
        .execute(
            "UPDATE activities SET updated_at = ?1 WHERE id = ?2",
            params![sentinel_updated_at, activity.id.as_str()],
        )
        .expect("activity updated_at should be pinned");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let duplicate_contact_link = core
        .add_activity_link(&activity.id, "contact", &contact.id)
        .expect("duplicate contact activity link should be a no-op");
    let duplicate_deal_link = core
        .add_activity_link(&activity.id, "deal", &deal.id)
        .expect("duplicate deal activity link should be a no-op");

    assert_eq!(duplicate_contact_link.id, contact_link_id);
    assert_eq!(duplicate_deal_link.id, deal_link_id);
    let activity_after_duplicates = core
        .get_activity(&activity.id)
        .expect("activity should load after duplicate adds");
    assert_eq!(
        activity_after_duplicates.updated_at.as_str(),
        sentinel_updated_at
    );
    assert_eq!(
        activity_after_duplicates.contact_id.as_deref(),
        Some(contact.id.as_str())
    );
    assert_eq!(
        activity_after_duplicates.deal_id.as_deref(),
        Some(deal.id.as_str())
    );
    let active_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = ?1 AND deleted_at IS NULL",
            params![activity.id],
            |row| row.get(0),
        )
        .expect("active activity link count should query");
    assert_eq!(active_link_count, 2);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_activity_changing_and_clearing_contact_deal_keeps_links_and_mirrors_aligned() {
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
    let first_deal = core
        .create_deal(
            "First deal".to_string(),
            Some(1000.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first deal should be created");
    let second_deal = core
        .create_deal(
            "Second deal".to_string(),
            Some(2000.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second deal should be created");
    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(first_contact.id.clone()),
            Some(first_deal.id.clone()),
        )
        .expect("activity should be created");

    let updated = core
        .update_activity(
            &activity.id,
            None,
            None,
            None,
            None,
            None,
            Some(Some(second_contact.id.clone())),
            Some(Some(second_deal.id.clone())),
        )
        .expect("activity relationships should update");
    assert_eq!(
        updated.contact_id.as_deref(),
        Some(second_contact.id.as_str())
    );
    assert_eq!(updated.deal_id.as_deref(), Some(second_deal.id.as_str()));

    let active_links = core
        .list_activity_links(&activity.id)
        .expect("activity links should list after update");
    assert_eq!(active_links.len(), 2);
    assert!(active_links
        .iter()
        .any(|link| link.entity_id == second_contact.id));
    assert!(active_links
        .iter()
        .any(|link| link.entity_id == second_deal.id));

    let cleared = core
        .update_activity(
            &activity.id,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            Some(None),
        )
        .expect("activity relationships should clear");
    assert_eq!(cleared.contact_id, None);
    assert_eq!(cleared.deal_id, None);
    assert!(core
        .list_activity_links(&activity.id)
        .expect("activity links should be empty after clear")
        .is_empty());
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE deleted_at IS NOT NULL"
        ),
        4
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__delete__'"
        ),
        4
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity' AND field_name IN ('contact_id', 'deal_id')"
        ),
        4
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn organization_activity_link_validates_reference_writes_audit_sync_and_skips_legacy_columns() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Regional Clinic".to_string(),
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
    let activity = core
        .create_activity(
            "meeting".to_string(),
            "Planning call".to_string(),
            None,
            None,
            None,
            None,
        )
        .expect("activity should be created");

    let err = core
        .add_activity_link(&activity.id, "organization", "missing-org")
        .expect_err("missing organization should be rejected");
    match err {
        crate::utils::errors::CrmError::NotFound(_) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }

    let link = core
        .add_activity_link(&activity.id, "organization", &organization.id)
        .expect("organization activity link should be created");
    assert_eq!(link.entity_id, organization.id);
    assert_eq!(
        link.entity_type,
        crate::storage::activities::ActivityLinkEntityType::Organization
    );

    let activity_after_link = core
        .get_activity(&activity.id)
        .expect("activity should load after organization link");
    assert_eq!(activity_after_link.contact_id, None);
    assert_eq!(activity_after_link.deal_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'activity_link'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__create__'"
        ),
        1
    );

    let removed = core
        .remove_activity_link(&activity.id, "organization", &organization.id)
        .expect("organization activity link should remove");
    assert!(removed.deleted_at.is_some());
    let activity_after_remove = core
        .get_activity(&activity.id)
        .expect("activity should load after organization unlink");
    assert_eq!(activity_after_remove.contact_id, None);
    assert_eq!(activity_after_remove.deal_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity' AND field_name IN ('contact_id', 'deal_id')"
        ),
        0
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__delete__'"
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
fn migration_v8_creates_activity_relationship_schema_and_backfills_valid_legacy_links() {
    let path = std::env::temp_dir().join(format!("900crm-v8-backfill-test-{}", new_uuid()));
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
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT,
                deal_id       TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, deleted_at) VALUES ('contact-1', NULL);
            INSERT INTO deals (id, deleted_at) VALUES ('deal-1', NULL);
            INSERT INTO activities
                (id, activity_type, title, description, contact_id, deal_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('activity-1', 'task', 'Follow up', '', 'contact-1', 'deal-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 7;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v8");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'activity_links'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_activity_links_active_unique'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = 'activity-1' AND entity_type = 'contact' AND entity_id = 'contact-1' AND deleted_at IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = 'activity-1' AND entity_type = 'deal' AND entity_id = 'deal-1' AND deleted_at IS NULL"
        ),
        1
    );
    assert_eq!(
        core.db
            .schema_version()
            .expect("schema version should read"),
        8
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v8_skips_missing_deleted_and_deleted_activity_legacy_links() {
    let path = std::env::temp_dir().join(format!("900crm-v8-invalid-backfill-test-{}", new_uuid()));
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
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT,
                deal_id       TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, deleted_at)
            VALUES
                ('active-contact', NULL),
                ('deleted-contact', '2026-06-24T09:00:00Z');
            INSERT INTO deals (id, deleted_at)
            VALUES
                ('active-deal', NULL),
                ('deleted-deal', '2026-06-24T09:00:00Z');
            INSERT INTO activities
                (id, activity_type, title, description, contact_id, deal_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('missing-contact-activity', 'task', 'Missing contact', '', 'missing-contact', NULL, '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-contact-activity', 'task', 'Deleted contact', '', 'deleted-contact', NULL, '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('missing-deal-activity', 'task', 'Missing deal', '', NULL, 'missing-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-deal-activity', 'task', 'Deleted deal', '', NULL, 'deleted-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-activity', 'task', 'Deleted activity', '', 'active-contact', 'active-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', '2026-06-24T09:30:00Z', 'device-a');
            PRAGMA user_version = 7;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v8");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE deleted_at IS NULL"
        ),
        0
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
    assert_eq!(report.contacts_with_invalid_legacy_org_id_links, 0);
    assert_eq!(
        report.contacts_with_invalid_normalized_organization_id_links,
        0
    );
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
    assert_eq!(report.contacts_with_invalid_legacy_org_id_links, 1);
    assert_eq!(
        report.contacts_with_invalid_normalized_organization_id_links,
        1
    );
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
