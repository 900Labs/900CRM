//! Offline MCP readiness catalog and runtime guard foundation.
//!
//! This crate does not start an MCP server, bind a listener, execute tools,
//! manage tokens, expose prompts/resources, or integrate with model providers.
//! It only publishes deterministic local catalog/status metadata for future
//! runtime work.

use std::{fmt, net::IpAddr};

use serde::Serialize;

/// CLI flag that prints the offline tool catalog as deterministic JSON.
pub const PRINT_TOOL_CATALOG_FLAG: &str = "--print-tool-catalog";
/// Shorter alias for printing the offline tool catalog.
pub const LIST_TOOLS_FLAG: &str = "--list-tools";
/// CLI flag that prints the disabled runtime guard status as deterministic JSON.
pub const PRINT_RUNTIME_STATUS_FLAG: &str = "--print-runtime-status";

/// Honest default status for running the placeholder binary without a catalog flag.
pub const DEFAULT_STATUS_MESSAGE: &str = "900CRM MCP server/runtime is not implemented. This binary does not start a server, listener, tools, prompts, resources, network binding, token handling, or model integration. Use --print-tool-catalog to print the offline SDK-backed read-only catalog or --print-runtime-status to print the disabled runtime guard status.";

/// Reason reported when the default runtime guard is disabled.
pub const RUNTIME_DISABLED_REASON: &str = "runtime disabled";
/// Reason reported when configuration is enabled but runtime serving is absent.
pub const RUNTIME_SERVER_NOT_IMPLEMENTED_REASON: &str = "server not implemented";

/// Disabled-by-default runtime guard configuration for future MCP work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpRuntimeConfig {
    /// Whether a future runtime would be allowed to start.
    pub enabled: bool,
    /// Local bind host reserved for a future runtime listener.
    pub bind_host: String,
    /// Local bind port reserved for a future runtime listener.
    pub bind_port: u16,
}

impl Default for McpRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_host: "127.0.0.1".to_string(),
            bind_port: 0,
        }
    }
}

impl McpRuntimeConfig {
    /// Validates the guard configuration without binding sockets or starting a server.
    pub fn validate(&self) -> Result<(), McpRuntimeConfigError> {
        if !self.enabled {
            return Ok(());
        }

        let bind_host = self.bind_host.trim();
        if bind_host.is_empty() {
            return Err(McpRuntimeConfigError::EmptyBindHostWhenEnabled);
        }

        if is_loopback_bind_host(bind_host) {
            Ok(())
        } else {
            Err(McpRuntimeConfigError::NonLoopbackBindHostWhenEnabled {
                bind_host: self.bind_host.clone(),
            })
        }
    }
}

/// Validation failures for the disabled runtime guard configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpRuntimeConfigError {
    /// Enabled runtime configuration must name an explicit loopback host.
    EmptyBindHostWhenEnabled,
    /// Enabled runtime configuration must not expose a non-loopback host.
    NonLoopbackBindHostWhenEnabled { bind_host: String },
}

impl fmt::Display for McpRuntimeConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBindHostWhenEnabled => {
                write!(
                    formatter,
                    "enabled MCP runtime config requires a loopback bind_host"
                )
            }
            Self::NonLoopbackBindHostWhenEnabled { bind_host } => write!(
                formatter,
                "enabled MCP runtime config rejects non-loopback bind_host '{bind_host}'"
            ),
        }
    }
}

impl std::error::Error for McpRuntimeConfigError {}

/// Explicit non-serving runtime status for the guard configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpRuntimeStatus {
    /// Whether runtime startup is configured as enabled.
    pub enabled: bool,
    /// Configured bind host for a future runtime listener.
    pub bind_host: String,
    /// Configured bind port for a future runtime listener.
    pub bind_port: u16,
    /// Whether this crate is currently serving MCP requests.
    pub serving: bool,
    /// Whether this crate currently executes MCP tools.
    pub tool_execution_enabled: bool,
    /// Human-readable reason for the disabled/non-serving status.
    pub reason: &'static str,
}

impl McpRuntimeStatus {
    fn from_config(config: &McpRuntimeConfig) -> Self {
        let reason = if config.enabled {
            RUNTIME_SERVER_NOT_IMPLEMENTED_REASON
        } else {
            RUNTIME_DISABLED_REASON
        };

        Self {
            enabled: config.enabled,
            bind_host: config.bind_host.clone(),
            bind_port: config.bind_port,
            serving: false,
            tool_execution_enabled: false,
            reason,
        }
    }
}

/// Returns the runtime guard status without starting a server or binding sockets.
pub fn runtime_status(config: &McpRuntimeConfig) -> McpRuntimeStatus {
    McpRuntimeStatus::from_config(config)
}

/// Returns the default disabled runtime guard status.
pub fn default_runtime_status() -> McpRuntimeStatus {
    runtime_status(&McpRuntimeConfig::default())
}

/// Serializes the default runtime guard status as deterministic pretty JSON.
pub fn default_runtime_status_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&default_runtime_status())
}

fn is_loopback_bind_host(bind_host: &str) -> bool {
    bind_host == "localhost"
        || bind_host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

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
        "Usage: {program_name} [{PRINT_TOOL_CATALOG_FLAG}|{LIST_TOOLS_FLAG}|{PRINT_RUNTIME_STATUS_FLAG}]\n\n\
Default: print the current MCP readiness status. No server, listener, tool execution, prompt/resource serving, token handling, or network binding is implemented.\n\
{PRINT_TOOL_CATALOG_FLAG}, {LIST_TOOLS_FLAG}: print the offline SDK-backed read-only tool catalog as JSON.\n\
{PRINT_RUNTIME_STATUS_FLAG}: print the disabled runtime guard status as JSON."
    )
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        default_runtime_status, default_runtime_status_json, read_only_tool_catalog,
        read_only_tool_catalog_json, runtime_status, McpRuntimeConfig, McpRuntimeConfigError,
        DEFAULT_STATUS_MESSAGE, PRINT_RUNTIME_STATUS_FLAG, PRINT_TOOL_CATALOG_FLAG,
        RUNTIME_DISABLED_REASON, RUNTIME_SERVER_NOT_IMPLEMENTED_REASON,
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
        assert!(DEFAULT_STATUS_MESSAGE.contains(PRINT_RUNTIME_STATUS_FLAG));
        assert!(!DEFAULT_STATUS_MESSAGE.contains("server is running"));
        assert!(!DEFAULT_STATUS_MESSAGE.contains("listening"));
    }

    #[test]
    fn default_runtime_config_is_disabled_and_localhost_only() {
        let config = McpRuntimeConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.bind_host, "127.0.0.1");
        assert_eq!(config.bind_port, 0);
        config
            .validate()
            .expect("default disabled runtime config should validate");
    }

    #[test]
    fn default_runtime_status_is_not_serving_and_never_executes_tools() {
        let status = default_runtime_status();

        assert!(!status.enabled);
        assert_eq!(status.bind_host, "127.0.0.1");
        assert_eq!(status.bind_port, 0);
        assert!(!status.serving);
        assert!(!status.tool_execution_enabled);
        assert_eq!(status.reason, RUNTIME_DISABLED_REASON);
    }

    #[test]
    fn enabled_runtime_config_accepts_loopback_hosts() {
        for bind_host in ["127.0.0.1", "127.42.0.1", "::1", "localhost"] {
            let config = McpRuntimeConfig {
                enabled: true,
                bind_host: bind_host.to_string(),
                bind_port: 0,
            };

            config
                .validate()
                .expect("enabled loopback runtime config should validate");

            let status = runtime_status(&config);
            assert!(!status.serving);
            assert!(!status.tool_execution_enabled);
            assert_eq!(status.reason, RUNTIME_SERVER_NOT_IMPLEMENTED_REASON);
        }
    }

    #[test]
    fn enabled_runtime_config_rejects_non_loopback_hosts() {
        for bind_host in ["0.0.0.0", "192.168.1.10", "::", "example.com"] {
            let config = McpRuntimeConfig {
                enabled: true,
                bind_host: bind_host.to_string(),
                bind_port: 0,
            };

            assert_eq!(
                config.validate(),
                Err(McpRuntimeConfigError::NonLoopbackBindHostWhenEnabled {
                    bind_host: bind_host.to_string()
                })
            );
        }
    }

    #[test]
    fn default_runtime_status_json_is_deterministic_and_parseable() {
        let first = default_runtime_status_json().expect("runtime status JSON should serialize");
        let second = default_runtime_status_json().expect("runtime status JSON should serialize");

        assert_eq!(first, second);
        let parsed: Value = serde_json::from_str(&first).expect("runtime status JSON should parse");
        assert_eq!(parsed["enabled"], false);
        assert_eq!(parsed["bind_host"], "127.0.0.1");
        assert_eq!(parsed["bind_port"], 0);
        assert_eq!(parsed["serving"], false);
        assert_eq!(parsed["tool_execution_enabled"], false);
        assert_eq!(parsed["reason"], RUNTIME_DISABLED_REASON);
        assert_eq!(first, expected_default_runtime_status_json());
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

    fn expected_default_runtime_status_json() -> String {
        format!(
            r#"{{
  "enabled": false,
  "bind_host": "127.0.0.1",
  "bind_port": 0,
  "serving": false,
  "tool_execution_enabled": false,
  "reason": "{RUNTIME_DISABLED_REASON}"
}}"#
        )
    }
}
