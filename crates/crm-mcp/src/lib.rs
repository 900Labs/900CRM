//! Offline MCP readiness catalog and runtime guard foundation.
//!
//! This crate does not start an MCP server, bind a listener, execute tools,
//! manage tokens, expose prompts/resources, or integrate with model providers.
//! It only publishes deterministic local catalog/status metadata for future
//! runtime work.

use std::{fmt, fs, io, net::IpAddr, path::Path};

use serde::{Deserialize, Serialize};

/// CLI flag that prints the offline tool catalog as deterministic JSON.
pub const PRINT_TOOL_CATALOG_FLAG: &str = "--print-tool-catalog";
/// Shorter alias for printing the offline tool catalog.
pub const LIST_TOOLS_FLAG: &str = "--list-tools";
/// CLI flag that prints the disabled runtime guard status as deterministic JSON.
pub const PRINT_RUNTIME_STATUS_FLAG: &str = "--print-runtime-status";
/// CLI flag that prints runtime guard status after loading JSON config metadata.
pub const PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG: &str = "--print-runtime-status-from-config";
/// CLI flag that handles one JSON-RPC message and prints at most one response.
pub const HANDLE_JSONRPC_ONCE_FLAG: &str = "--handle-jsonrpc-once";

/// Honest default status for running the placeholder binary without a catalog flag.
pub const DEFAULT_STATUS_MESSAGE: &str = "900CRM MCP server/runtime is not implemented. This binary does not start a server, listener, tools, prompts, resources, network binding, token handling, or model integration. Use --print-tool-catalog to print the offline SDK-backed read-only catalog or --print-runtime-status to print the disabled runtime guard status.";

/// Reason reported when the default runtime guard is disabled.
pub const RUNTIME_DISABLED_REASON: &str = "runtime disabled";
/// Reason reported when configuration is enabled but runtime serving is absent.
pub const RUNTIME_SERVER_NOT_IMPLEMENTED_REASON: &str = "server not implemented";

/// Disabled-by-default runtime guard configuration for future MCP work.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

/// File loading, JSON parsing, and validation failures for MCP runtime config metadata.
#[derive(Debug)]
pub enum McpRuntimeConfigLoadError {
    /// Config file could not be read.
    Read { path: String, source: io::Error },
    /// Config file did not contain valid JSON for `McpRuntimeConfig`.
    Parse {
        path: String,
        source: serde_json::Error,
    },
    /// Config file parsed but failed runtime guard validation.
    Validation {
        path: String,
        source: McpRuntimeConfigError,
    },
}

impl fmt::Display for McpRuntimeConfigLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read MCP runtime config '{path}': {source}"
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "failed to parse MCP runtime config JSON '{path}': {source}"
                )
            }
            Self::Validation { path, source } => {
                write!(formatter, "invalid MCP runtime config '{path}': {source}")
            }
        }
    }
}

impl std::error::Error for McpRuntimeConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Validation { source, .. } => Some(source),
        }
    }
}

/// Reads, parses, and validates JSON runtime config metadata from a required path.
///
/// This is a config/status helper only. It does not start a server, bind
/// sockets, execute tools, call the SDK, issue tokens, or access the network.
pub fn load_runtime_config_from_path(
    path: impl AsRef<Path>,
) -> Result<McpRuntimeConfig, McpRuntimeConfigLoadError> {
    let path = path.as_ref();
    let display_path = path.display().to_string();
    let raw = fs::read_to_string(path).map_err(|source| McpRuntimeConfigLoadError::Read {
        path: display_path.clone(),
        source,
    })?;
    let config: McpRuntimeConfig =
        serde_json::from_str(&raw).map_err(|source| McpRuntimeConfigLoadError::Parse {
            path: display_path.clone(),
            source,
        })?;

    config
        .validate()
        .map_err(|source| McpRuntimeConfigLoadError::Validation {
            path: display_path,
            source,
        })?;

    Ok(config)
}

/// Loads runtime config metadata from JSON, falling back to disabled defaults if absent.
///
/// A missing file is treated as an optional config not being present. Other
/// read errors, invalid JSON, and validation failures are returned explicitly.
pub fn load_runtime_config_from_optional_path(
    path: impl AsRef<Path>,
) -> Result<McpRuntimeConfig, McpRuntimeConfigLoadError> {
    let path = path.as_ref();
    match load_runtime_config_from_path(path) {
        Ok(config) => Ok(config),
        Err(McpRuntimeConfigLoadError::Read { source, .. })
            if source.kind() == io::ErrorKind::NotFound =>
        {
            Ok(McpRuntimeConfig::default())
        }
        Err(error) => Err(error),
    }
}

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

/// Serializes runtime guard status for a config as deterministic pretty JSON.
pub fn runtime_status_json(config: &McpRuntimeConfig) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&runtime_status(config))
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

/// Handles one metadata-only JSON-RPC/MCP request without starting a runtime.
///
/// This one-shot helper does not read from stdio, start a serving loop, bind
/// sockets, execute tools, call the SDK, authenticate clients, or access data.
pub fn handle_jsonrpc_once(raw: &str) -> Result<Option<String>, serde_json::Error> {
    let response = handle_jsonrpc_value(raw);
    response
        .map(|value| serde_json::to_string(&value).map(Some))
        .unwrap_or(Ok(None))
}

fn handle_jsonrpc_value(raw: &str) -> Option<serde_json::Value> {
    let parsed: serde_json::Value = match serde_json::from_str(raw) {
        Ok(value) => value,
        Err(_) => return Some(jsonrpc_error(None, -32700, "Parse error")),
    };

    let Some(request) = parsed.as_object() else {
        return Some(jsonrpc_error(None, -32600, "Invalid Request"));
    };

    let id = request.get("id").cloned();
    let is_notification = id.is_none();

    if request.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        return Some(jsonrpc_error(id, -32600, "Invalid Request"));
    }

    let Some(method) = request.get("method").and_then(serde_json::Value::as_str) else {
        return Some(jsonrpc_error(id, -32600, "Invalid Request"));
    };

    if is_notification {
        return None;
    }

    match method {
        "initialize" => Some(jsonrpc_success(id, initialize_result())),
        "tools/list" => Some(jsonrpc_success(id, tools_list_result())),
        "tools/call" => Some(jsonrpc_error(
            id,
            -32601,
            "Method not found: tools/call execution is not implemented or enabled",
        )),
        _ => Some(jsonrpc_error(id, -32601, "Method not found")),
    }
}

fn initialize_result() -> serde_json::Value {
    serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "900crm-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "status": "metadata-only"
        },
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "instructions": "Metadata-only one-shot probe. No serving loop, transport, listener, authentication, SDK dispatch, database access, or tool execution is enabled."
    })
}

fn tools_list_result() -> serde_json::Value {
    let tools: Vec<serde_json::Value> = read_only_tool_catalog()
        .iter()
        .map(mcp_tool_metadata)
        .collect();

    serde_json::json!({
        "tools": tools
    })
}

fn mcp_tool_metadata(entry: &ToolCatalogEntry) -> serde_json::Value {
    serde_json::json!({
        "name": entry.name,
        "description": tool_description(entry.name),
        "inputSchema": tool_input_schema(entry.name),
        "annotations": {
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false
        },
        "metadata": {
            "accessKind": entry.access_kind,
            "requiresExternalClientPermission": entry.requires_external_client_permission,
            "sdkBacked": entry.sdk_backed,
            "runtimeEnabled": entry.runtime_enabled,
            "executionEnabled": false,
            "readiness": "metadata-only; runtime execution is not enabled"
        }
    })
}

fn tool_description(name: &str) -> &'static str {
    match name {
        crm_sdk::CONTACTS_LIST_TOOL => {
            "List contacts metadata contract. Read-only catalog entry; runtime execution is not enabled."
        }
        crm_sdk::ORGANIZATIONS_LIST_TOOL => {
            "List organizations metadata contract. Read-only catalog entry; runtime execution is not enabled."
        }
        crm_sdk::DEALS_LIST_TOOL => {
            "List deals metadata contract. Read-only catalog entry; runtime execution is not enabled."
        }
        crm_sdk::ACTIVITIES_LIST_TOOL => {
            "List activities metadata contract. Read-only catalog entry; runtime execution is not enabled."
        }
        crm_sdk::SEARCH_GLOBAL_TOOL => {
            "Search CRM records metadata contract. Read-only catalog entry; runtime execution is not enabled."
        }
        _ => "Read-only metadata contract. Runtime execution is not enabled.",
    }
}

fn tool_input_schema(name: &str) -> serde_json::Value {
    match name {
        crm_sdk::CONTACTS_LIST_TOOL => serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional read-only result limit for future execution."
                },
                "offset": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Optional read-only result offset for future execution."
                }
            }
        }),
        crm_sdk::SEARCH_GLOBAL_TOOL => serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["query"],
            "properties": {
                "query": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Read-only search query for future execution."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional read-only result limit for future execution."
                }
            }
        }),
        _ => serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {}
        }),
    }
}

fn jsonrpc_success(id: Option<serde_json::Value>, result: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "result": result
    })
}

fn jsonrpc_error(
    id: Option<serde_json::Value>,
    code: i64,
    message: &'static str,
) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    })
}

/// Returns the CLI help text without implying an implemented MCP runtime.
pub fn help_message(program_name: &str) -> String {
    format!(
        "Usage: {program_name} [{PRINT_TOOL_CATALOG_FLAG}|{LIST_TOOLS_FLAG}|{PRINT_RUNTIME_STATUS_FLAG}|{PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG} <path>|{HANDLE_JSONRPC_ONCE_FLAG} <json>]\n\n\
Default: print the current MCP readiness status. No server, listener, tool execution, prompt/resource serving, token handling, or network binding is implemented.\n\
{PRINT_TOOL_CATALOG_FLAG}, {LIST_TOOLS_FLAG}: print the offline SDK-backed read-only tool catalog as JSON.\n\
{PRINT_RUNTIME_STATUS_FLAG}: print the disabled runtime guard status as JSON.\n\
{PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG} <path>: load JSON config metadata from an optional path and print non-serving runtime guard status as JSON.\n\
{HANDLE_JSONRPC_ONCE_FLAG} <json>: handle one metadata-only JSON-RPC request and print one response, or nothing for notifications."
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::Value;

    use super::{
        default_runtime_status, default_runtime_status_json,
        load_runtime_config_from_optional_path, read_only_tool_catalog,
        read_only_tool_catalog_json, runtime_status, runtime_status_json, McpRuntimeConfig,
        McpRuntimeConfigError, McpRuntimeConfigLoadError, DEFAULT_STATUS_MESSAGE,
        PRINT_RUNTIME_STATUS_FLAG, PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG, PRINT_TOOL_CATALOG_FLAG,
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
        assert!(!DEFAULT_STATUS_MESSAGE.contains(PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG));
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
    fn missing_optional_runtime_config_path_uses_disabled_default() {
        let path = temp_config_path("missing");
        let config = load_runtime_config_from_optional_path(&path)
            .expect("missing optional runtime config should use default");

        assert_eq!(config, McpRuntimeConfig::default());
    }

    #[test]
    fn valid_disabled_runtime_config_json_parses_and_reports_disabled_status() {
        let path = write_temp_config(
            "disabled",
            r#"{
  "enabled": false,
  "bind_host": "127.0.0.1",
  "bind_port": 0
}"#,
        );
        let config = load_runtime_config_from_optional_path(&path)
            .expect("disabled runtime config should parse");
        let status = runtime_status(&config);

        assert_eq!(
            config,
            McpRuntimeConfig {
                enabled: false,
                bind_host: "127.0.0.1".to_string(),
                bind_port: 0,
            }
        );
        assert!(!status.serving);
        assert!(!status.tool_execution_enabled);
        assert_eq!(status.reason, RUNTIME_DISABLED_REASON);
        fs::remove_file(path).ok();
    }

    #[test]
    fn valid_enabled_loopback_runtime_config_reports_server_not_implemented() {
        let path = write_temp_config(
            "enabled-loopback",
            r#"{
  "enabled": true,
  "bind_host": "localhost",
  "bind_port": 3987
}"#,
        );
        let config = load_runtime_config_from_optional_path(&path)
            .expect("enabled loopback runtime config should parse");
        let status_json =
            runtime_status_json(&config).expect("runtime status JSON should serialize");

        assert!(config.enabled);
        assert_eq!(config.bind_host, "localhost");
        assert_eq!(config.bind_port, 3987);
        assert_eq!(
            status_json,
            r#"{
  "enabled": true,
  "bind_host": "localhost",
  "bind_port": 3987,
  "serving": false,
  "tool_execution_enabled": false,
  "reason": "server not implemented"
}"#
        );
        fs::remove_file(path).ok();
    }

    #[test]
    fn enabled_non_loopback_runtime_config_json_is_rejected() {
        let path = write_temp_config(
            "non-loopback",
            r#"{
  "enabled": true,
  "bind_host": "0.0.0.0",
  "bind_port": 3987
}"#,
        );
        let error = load_runtime_config_from_optional_path(&path)
            .expect_err("enabled non-loopback runtime config should fail validation");

        assert!(matches!(
            error,
            McpRuntimeConfigLoadError::Validation {
                source: McpRuntimeConfigError::NonLoopbackBindHostWhenEnabled { .. },
                ..
            }
        ));
        assert!(error.to_string().contains("non-loopback"));
        fs::remove_file(path).ok();
    }

    #[test]
    fn invalid_runtime_config_json_is_rejected() {
        let path = write_temp_config("invalid-json", r#"{ "enabled": true, "#);
        let error = load_runtime_config_from_optional_path(&path)
            .expect_err("invalid runtime config JSON should fail parsing");

        assert!(matches!(error, McpRuntimeConfigLoadError::Parse { .. }));
        assert!(error.to_string().contains("failed to parse"));
        fs::remove_file(path).ok();
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

    fn write_temp_config(name: &str, contents: &str) -> PathBuf {
        let path = temp_config_path(name);
        fs::write(&path, contents).expect("temp runtime config should be writable");
        path
    }

    fn temp_config_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "crm-mcp-runtime-config-{name}-{}-{unique}.json",
            std::process::id()
        ))
    }
}
