//! Offline MCP readiness catalog for reviewed SDK-backed read tools.
//!
//! This crate does not start an MCP server, bind a listener, execute tools,
//! manage tokens, expose prompts/resources, or integrate with model providers.
//! It only publishes deterministic local catalog metadata for future runtime
//! work.

use serde::Serialize;

/// CLI flag that prints the offline tool catalog as deterministic JSON.
pub const PRINT_TOOL_CATALOG_FLAG: &str = "--print-tool-catalog";
/// Shorter alias for printing the offline tool catalog.
pub const LIST_TOOLS_FLAG: &str = "--list-tools";

/// Honest default status for running the placeholder binary without a catalog flag.
pub const DEFAULT_STATUS_MESSAGE: &str = "900CRM MCP server/runtime is not implemented. This binary does not start a server, listener, tools, prompts, resources, network binding, token handling, or model integration. Use --print-tool-catalog to print the offline SDK-backed read-only catalog.";

/// One offline catalog entry for a future MCP tool boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolCatalogEntry {
    /// SDK-backed tool name.
    pub name: &'static str,
    /// Current reviewed access kind for the tool.
    pub access_kind: &'static str,
    /// Whether callers must pass external-client permission checks before use.
    pub requires_external_client_permission: bool,
    /// Whether this catalog entry is backed by the reviewed `crm-sdk` constants.
    pub sdk_backed: bool,
    /// Whether this crate currently serves or executes the tool at runtime.
    pub runtime_enabled: bool,
}

impl ToolCatalogEntry {
    fn sdk_read_tool(name: &'static str) -> Self {
        Self {
            name,
            access_kind: "read",
            requires_external_client_permission: true,
            sdk_backed: true,
            runtime_enabled: false,
        }
    }
}

/// Returns the initial offline read-only tool catalog in SDK-defined order.
pub fn read_only_tool_catalog() -> Vec<ToolCatalogEntry> {
    crm_sdk::INITIAL_READ_TOOL_NAMES
        .iter()
        .copied()
        .map(ToolCatalogEntry::sdk_read_tool)
        .collect()
}

/// Serializes the offline read-only tool catalog as deterministic pretty JSON.
pub fn read_only_tool_catalog_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&read_only_tool_catalog())
}

/// Returns the CLI help text without implying an implemented MCP runtime.
pub fn help_message(program_name: &str) -> String {
    format!(
        "Usage: {program_name} [{PRINT_TOOL_CATALOG_FLAG}|{LIST_TOOLS_FLAG}]\n\n\
Default: print the current MCP readiness status. No server, listener, tool execution, prompt/resource serving, token handling, or network binding is implemented.\n\
{PRINT_TOOL_CATALOG_FLAG}, {LIST_TOOLS_FLAG}: print the offline SDK-backed read-only tool catalog as JSON."
    )
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        read_only_tool_catalog, read_only_tool_catalog_json, DEFAULT_STATUS_MESSAGE,
        PRINT_TOOL_CATALOG_FLAG,
    };

    #[test]
    fn catalog_contains_exact_sdk_initial_read_tools_in_stable_order() {
        let catalog = read_only_tool_catalog();
        let names: Vec<&str> = catalog.iter().map(|entry| entry.name).collect();

        assert_eq!(names, crm_sdk::INITIAL_READ_TOOL_NAMES);
    }

    #[test]
    fn catalog_entries_are_read_only_permission_gated_and_not_runtime_enabled() {
        let catalog = read_only_tool_catalog();

        assert!(!catalog.is_empty());
        for entry in catalog {
            assert_eq!(entry.access_kind, "read");
            assert!(entry.requires_external_client_permission);
            assert!(entry.sdk_backed);
            assert!(!entry.runtime_enabled);
        }
    }

    #[test]
    fn catalog_json_is_deterministic_and_parseable() {
        let first = read_only_tool_catalog_json().expect("catalog JSON should serialize");
        let second = read_only_tool_catalog_json().expect("catalog JSON should serialize");
        assert_eq!(first, second);

        let parsed: Value = serde_json::from_str(&first).expect("catalog JSON should parse");
        assert!(parsed.is_array());
        assert_eq!(first, expected_catalog_json());
    }

    #[test]
    fn default_status_is_explicitly_non_runtime() {
        assert!(DEFAULT_STATUS_MESSAGE.contains("not implemented"));
        assert!(DEFAULT_STATUS_MESSAGE.contains("does not start a server"));
        assert!(DEFAULT_STATUS_MESSAGE.contains(PRINT_TOOL_CATALOG_FLAG));
        assert!(!DEFAULT_STATUS_MESSAGE.contains("server is running"));
        assert!(!DEFAULT_STATUS_MESSAGE.contains("listening"));
    }

    fn expected_catalog_json() -> String {
        let tools = crm_sdk::INITIAL_READ_TOOL_NAMES;
        assert_eq!(tools.len(), 5);
        format!(
            r#"[
  {{
    "name": "{}",
    "access_kind": "read",
    "requires_external_client_permission": true,
    "sdk_backed": true,
    "runtime_enabled": false
  }},
  {{
    "name": "{}",
    "access_kind": "read",
    "requires_external_client_permission": true,
    "sdk_backed": true,
    "runtime_enabled": false
  }},
  {{
    "name": "{}",
    "access_kind": "read",
    "requires_external_client_permission": true,
    "sdk_backed": true,
    "runtime_enabled": false
  }},
  {{
    "name": "{}",
    "access_kind": "read",
    "requires_external_client_permission": true,
    "sdk_backed": true,
    "runtime_enabled": false
  }},
  {{
    "name": "{}",
    "access_kind": "read",
    "requires_external_client_permission": true,
    "sdk_backed": true,
    "runtime_enabled": false
  }}
]"#,
            tools[0], tools[1], tools[2], tools[3], tools[4]
        )
    }
}
