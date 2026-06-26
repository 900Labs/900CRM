//! Narrow local read-only SDK facade for future 900CRM integrations.
//!
//! This crate intentionally exposes no listener, credentials, network behavior,
//! write methods, or proposed-action creation. It is a local facade over
//! [`crm_core::CrmCore`] so future runtime packages can call reviewed core
//! services without raw SQL.

use std::path::Path;

use crm_core::{
    errors::CrmError,
    result::CrmResult,
    search::SearchResult,
    storage::{
        activities::Activity,
        contacts::{ContactListParams, ContactListResult},
        deals::Deal,
        organizations::Organization,
    },
    CrmCore,
};

/// Tool name for listing contacts through the read-only SDK.
pub const CONTACTS_LIST_TOOL: &str = "contacts.list";
/// Tool name for listing organizations through the read-only SDK.
pub const ORGANIZATIONS_LIST_TOOL: &str = "organizations.list";
/// Tool name for listing deals through the read-only SDK.
pub const DEALS_LIST_TOOL: &str = "deals.list";
/// Tool name for listing activities through the read-only SDK.
pub const ACTIVITIES_LIST_TOOL: &str = "activities.list";
/// Tool name for global search through the read-only SDK.
pub const SEARCH_GLOBAL_TOOL: &str = "search.global";

/// Initial read-only tool names exported for future MCP/runtime reuse.
pub const INITIAL_READ_TOOL_NAMES: &[&str] = &[
    CONTACTS_LIST_TOOL,
    ORGANIZATIONS_LIST_TOOL,
    DEALS_LIST_TOOL,
    ACTIVITIES_LIST_TOOL,
    SEARCH_GLOBAL_TOOL,
];

/// Local read-only SDK facade over [`CrmCore`].
pub struct ReadOnlyCrmSdk {
    core: CrmCore,
    external_client_id: String,
}

/// Product-facing alias for the read-only SDK surface.
pub type CrmSdk = ReadOnlyCrmSdk;

impl ReadOnlyCrmSdk {
    /// Opens the local CRM data store for a previously reviewed external client.
    pub fn open(
        app_data_dir: impl AsRef<Path>,
        external_client_id: impl Into<String>,
    ) -> CrmResult<Self> {
        let external_client_id = normalize_external_client_id(external_client_id.into())?;
        Ok(Self {
            core: CrmCore::open(app_data_dir.as_ref())?,
            external_client_id,
        })
    }

    /// Returns the external client id used for permission checks.
    pub fn external_client_id(&self) -> &str {
        &self.external_client_id
    }

    /// Lists contacts after verifying `contacts.list` read permission.
    pub fn contacts_list(&self, params: Option<ContactListParams>) -> CrmResult<ContactListResult> {
        self.require_read_permission(CONTACTS_LIST_TOOL)?;
        self.core.list_contacts(params)
    }

    /// Lists organizations after verifying `organizations.list` read permission.
    pub fn organizations_list(&self) -> CrmResult<Vec<Organization>> {
        self.require_read_permission(ORGANIZATIONS_LIST_TOOL)?;
        self.core.list_organizations()
    }

    /// Lists deals after verifying `deals.list` read permission.
    pub fn deals_list(&self) -> CrmResult<Vec<Deal>> {
        self.require_read_permission(DEALS_LIST_TOOL)?;
        self.core.list_deals()
    }

    /// Lists activities after verifying `activities.list` read permission.
    pub fn activities_list(&self) -> CrmResult<Vec<Activity>> {
        self.require_read_permission(ACTIVITIES_LIST_TOOL)?;
        self.core.list_activities()
    }

    /// Runs global search after verifying `search.global` read permission.
    pub fn search_global(&self, query: &str, limit: Option<u32>) -> CrmResult<Vec<SearchResult>> {
        self.require_read_permission(SEARCH_GLOBAL_TOOL)?;
        self.core.global_search(query, limit)
    }

    fn require_read_permission(&self, tool_name: &str) -> CrmResult<()> {
        let evaluation = self
            .core
            .evaluate_external_client_tool_read_permission(&self.external_client_id, tool_name)?;
        if evaluation.allowed {
            return Ok(());
        }

        Err(CrmError::InvalidInput(format!(
            "External client '{}' may not read tool '{}': {}",
            self.external_client_id,
            tool_name,
            evaluation.reason.as_str()
        )))
    }
}

/// Reports whether the SDK has real client behavior.
pub fn is_implemented() -> bool {
    true
}

fn normalize_external_client_id(external_client_id: String) -> CrmResult<String> {
    let external_client_id = external_client_id.trim();
    if external_client_id.is_empty() {
        return Err(CrmError::InvalidInput(
            "External client id is required".to_string(),
        ));
    }

    Ok(external_client_id.to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crm_core::{
        errors::CrmError,
        storage::{audit::AuditLogEntry, external_clients::ExternalClient},
        CrmCore,
    };

    use super::{is_implemented, ReadOnlyCrmSdk, CONTACTS_LIST_TOOL, SEARCH_GLOBAL_TOOL};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn sdk_reports_implemented_behavior() {
        assert!(is_implemented());
    }

    #[test]
    fn disabled_external_client_cannot_read_and_records_audit_evidence() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = core
            .create_external_client_placeholder("Disabled SDK Client", "mcp")
            .expect("client placeholder should be created");
        core.create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Hassan".to_string()),
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
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .contacts_list(None)
            .expect_err("disabled client should not read contacts");
        assert_invalid_input_contains(err, "client_disabled");
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            false,
            "denied",
        );
        cleanup(app_data_dir);
    }

    #[test]
    fn read_only_client_without_explicit_permission_cannot_read() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = create_read_only_client(&mut core, "Unpermitted SDK Client");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .contacts_list(None)
            .expect_err("missing tool permission should deny contacts");
        assert_invalid_input_contains(err, "missing_tool_permission");
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            false,
            "denied",
        );
        cleanup(app_data_dir);
    }

    #[test]
    fn read_only_client_with_explicit_permission_can_read_contacts() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let contact = core
            .create_contact(
                Some("person".to_string()),
                Some("Amina".to_string()),
                Some("Hassan".to_string()),
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
        let client = create_read_only_client(&mut core, "Allowed SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            false,
            false,
        )
        .expect("contacts list permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let contacts = sdk
            .contacts_list(None)
            .expect("permitted client should list contacts");
        assert_eq!(contacts.total, 1);
        assert_eq!(contacts.contacts.len(), 1);
        assert_eq!(contacts.contacts[0].id, contact.id);
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            "allowed",
        );
        cleanup(app_data_dir);
    }

    #[test]
    fn global_search_uses_the_sdk_permission_boundary() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        core.create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Searchable".to_string()),
            None,
            Some("searchable@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
        let client = create_read_only_client(&mut core, "Search SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            false,
            false,
        )
        .expect("contacts list permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .search_global("Amina", Some(10))
            .expect_err("contacts permission must not allow global search");
        assert_invalid_input_contains(err, "missing_tool_permission");
        drop(sdk);
        assert_permission_audit(
            &app_data_dir,
            &client.id,
            SEARCH_GLOBAL_TOOL,
            false,
            "denied",
        );

        let mut core = open_core(&app_data_dir);
        core.upsert_external_client_tool_permission(
            &client.id,
            SEARCH_GLOBAL_TOOL,
            true,
            false,
            false,
        )
        .expect("search permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should reopen");
        let results = sdk
            .search_global("Amina", Some(10))
            .expect("permitted client should search globally");
        assert!(
            results.iter().any(|result| result.title.contains("Amina")),
            "global search should return the seeded contact"
        );
        drop(sdk);
        assert_permission_audit(
            &app_data_dir,
            &client.id,
            SEARCH_GLOBAL_TOOL,
            true,
            "allowed",
        );
        cleanup(app_data_dir);
    }

    fn test_app_data_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "900crm-sdk-test-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn open_core(app_data_dir: &Path) -> CrmCore {
        CrmCore::open(app_data_dir).expect("test core should open")
    }

    fn create_read_only_client(core: &mut CrmCore, name: &str) -> ExternalClient {
        let client = core
            .create_external_client_placeholder(name, "mcp")
            .expect("external client placeholder should be created");
        core.update_external_client_activation(&client.id, true, "read_only")
            .expect("external client should enable read-only");
        client
    }

    fn assert_invalid_input_contains(error: CrmError, expected: &str) {
        match error {
            CrmError::InvalidInput(message) => {
                assert!(message.contains(expected), "{message}");
            }
            other => panic!("expected InvalidInput containing {expected}, got {other:?}"),
        }
    }

    fn assert_permission_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
        allowed: bool,
        status: &str,
    ) {
        let entry = latest_permission_audit(app_data_dir, client_id, tool_name);
        assert_eq!(entry.action, "evaluate_external_client_read_permission");
        assert_eq!(entry.entity_type.as_deref(), Some("external_client"));
        assert_eq!(entry.entity_id.as_deref(), Some(client_id));

        let after_json = entry
            .after_json
            .as_deref()
            .expect("permission audit should include after_json context");
        assert!(
            after_json.contains(&format!(r#""client_id":"{client_id}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""tool_name":"{tool_name}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(r#""access_kind":"read""#),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""allowed":{allowed}"#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""status":"{status}""#)),
            "{after_json}"
        );
    }

    fn latest_permission_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
    ) -> AuditLogEntry {
        let core = open_core(app_data_dir);
        core.list_recent_audit_log(50)
            .expect("audit log should list")
            .into_iter()
            .find(|entry| {
                entry.action == "evaluate_external_client_read_permission"
                    && entry.entity_id.as_deref() == Some(client_id)
                    && entry
                        .after_json
                        .as_deref()
                        .is_some_and(|json| json.contains(&format!(r#""tool_name":"{tool_name}""#)))
            })
            .expect("permission audit entry should exist")
    }

    fn cleanup(app_data_dir: PathBuf) {
        let _ = std::fs::remove_dir_all(app_data_dir);
    }
}
