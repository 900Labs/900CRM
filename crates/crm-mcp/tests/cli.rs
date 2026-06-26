use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use crm_mcp::{
    HANDLE_JSONRPC_ONCE_FLAG, PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG, SERVE_STDIO_FROM_CONFIG_FLAG,
};
use crm_sdk::INITIAL_READ_TOOL_NAMES;
use serde_json::Value;

#[test]
fn default_cli_output_does_not_imply_running_server() {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("not implemented"));
    assert!(stdout.contains("does not start a server"));
    assert!(stdout.contains("--print-runtime-status"));
    assert!(!stdout.contains("server is running"));
    assert!(!stdout.contains("listening"));
}

#[test]
fn default_cli_does_not_start_stdio_loop() {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .stdin(Stdio::piped())
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("not implemented"));
    assert!(!stdout.trim_end().ends_with(r#""jsonrpc":"2.0"}"#));
}

#[test]
fn print_tool_catalog_cli_outputs_parseable_deterministic_json() {
    let first = run_catalog_flag("--print-tool-catalog");
    let second = run_catalog_flag("--print-tool-catalog");

    assert_eq!(first, second);
    assert_catalog_json(&first);
}

#[test]
fn list_tools_alias_outputs_same_catalog_json() {
    let canonical = run_catalog_flag("--print-tool-catalog");
    let alias = run_catalog_flag("--list-tools");

    assert_eq!(alias, canonical);
}

#[test]
fn print_runtime_status_cli_outputs_parseable_deterministic_json() {
    let first = run_runtime_status_flag();
    let second = run_runtime_status_flag();

    assert_eq!(first, second);
    assert_runtime_status_json(&first);
}

#[test]
fn print_runtime_status_from_config_cli_outputs_parseable_deterministic_disabled_json() {
    let path = write_temp_config(
        "cli-disabled",
        r#"{
  "enabled": false,
  "bind_host": "127.0.0.1",
  "bind_port": 0
}"#,
    );
    let first = run_runtime_status_from_config(&path);
    let second = run_runtime_status_from_config(&path);

    assert_eq!(first, second);
    assert_runtime_status_json(&first);
    fs::remove_file(path).ok();
}

#[test]
fn print_runtime_status_from_config_cli_outputs_not_serving_for_enabled_loopback() {
    let path = write_temp_config(
        "cli-enabled-loopback",
        r#"{
  "enabled": true,
  "bind_host": "localhost",
  "bind_port": 3987
}"#,
    );
    let raw = run_runtime_status_from_config(&path);
    let parsed: Value = serde_json::from_str(&raw).expect("runtime status JSON should parse");

    assert_eq!(parsed["enabled"], true);
    assert_eq!(parsed["bind_host"], "localhost");
    assert_eq!(parsed["bind_port"], 3987);
    assert_eq!(parsed["serving"], false);
    assert_eq!(parsed["tool_execution_enabled"], false);
    assert_eq!(parsed["reason"], "server not implemented");
    assert_eq!(
        raw,
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
fn print_runtime_status_from_config_cli_exits_nonzero_for_invalid_json() {
    let path = write_temp_config("cli-invalid-json", r#"{ "enabled": true, "#);
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG)
        .arg(&path)
        .output()
        .expect("crm-mcp should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("failed to parse MCP runtime config JSON"));
    fs::remove_file(path).ok();
}

#[test]
fn print_runtime_status_from_config_cli_exits_nonzero_for_non_loopback_enabled_config() {
    let path = write_temp_config(
        "cli-non-loopback",
        r#"{
  "enabled": true,
  "bind_host": "0.0.0.0",
  "bind_port": 3987
}"#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG)
        .arg(&path)
        .output()
        .expect("crm-mcp should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("non-loopback"));
    fs::remove_file(path).ok();
}

#[test]
fn serve_stdio_from_config_cli_exits_nonzero_for_disabled_config_without_output() {
    let path = write_temp_config(
        "cli-stdio-disabled",
        r#"{
  "enabled": false,
  "bind_host": "127.0.0.1",
  "bind_port": 0
}"#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(SERVE_STDIO_FROM_CONFIG_FLAG)
        .arg(&path)
        .output()
        .expect("crm-mcp should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("MCP stdio loop is disabled by config"));
    fs::remove_file(path).ok();
}

#[test]
fn serve_stdio_from_config_cli_processes_piped_loopback_metadata() {
    let path = write_temp_config(
        "cli-stdio-enabled-loopback",
        r#"{
  "enabled": true,
  "bind_host": "127.0.0.1",
  "bind_port": 3987
}"#,
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(SERVE_STDIO_FROM_CONFIG_FLAG)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("crm-mcp should spawn");

    {
        let stdin = child.stdin.as_mut().expect("child stdin should be piped");
        stdin
            .write_all(
                concat!(
                    r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
                    "\n",
                    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                    "\n",
                    r#"{"jsonrpc":"2.0","id":"tools","method":"tools/list"}"#,
                    "\n",
                )
                .as_bytes(),
            )
            .expect("stdin write should succeed");
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("crm-mcp should finish after stdin closes");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        serde_json::from_str::<Value>(lines[0]).expect("initialize should parse")["id"],
        1
    );
    assert_eq!(
        serde_json::from_str::<Value>(lines[1]).expect("tools/list should parse")["id"],
        "tools"
    );
    fs::remove_file(path).ok();
}

#[test]
fn jsonrpc_initialize_probe_outputs_deterministic_non_runtime_capabilities() {
    let request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"test"}}}"#;
    let first = run_jsonrpc_probe(request);
    let second = run_jsonrpc_probe(request);

    assert_eq!(first, second);
    let parsed: Value = serde_json::from_str(&first).expect("initialize response should parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
    assert_eq!(parsed["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(parsed["result"]["serverInfo"]["name"], "900crm-mcp");
    assert_eq!(parsed["result"]["serverInfo"]["status"], "metadata-only");
    assert_eq!(
        parsed["result"]["capabilities"]["tools"]["listChanged"],
        false
    );
    assert!(parsed["result"]["capabilities"]["resources"].is_null());
    assert!(parsed["result"]["capabilities"]["prompts"].is_null());
    assert!(parsed["result"]["instructions"]
        .as_str()
        .expect("instructions should be a string")
        .contains("No serving loop"));
    assert!(parsed["result"]["instructions"]
        .as_str()
        .expect("instructions should be a string")
        .contains("tool execution is enabled"));
}

#[test]
fn jsonrpc_tools_list_probe_maps_catalog_in_order_with_read_only_schemas() {
    let raw = run_jsonrpc_probe(r#"{"jsonrpc":"2.0","id":"tools","method":"tools/list"}"#);
    let parsed: Value = serde_json::from_str(&raw).expect("tools/list response should parse");
    let tools = parsed["result"]["tools"]
        .as_array()
        .expect("tools/list result should include tools array");
    let names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name should be a string"))
        .collect();

    assert_eq!(names, INITIAL_READ_TOOL_NAMES);

    for tool in tools {
        assert!(tool["description"]
            .as_str()
            .expect("tool description should be a string")
            .contains("runtime execution is not enabled"));
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["annotations"]["destructiveHint"], false);
        assert_eq!(tool["annotations"]["openWorldHint"], false);
        assert_eq!(tool["metadata"]["accessKind"], "read");
        assert_eq!(tool["metadata"]["runtimeEnabled"], false);
        assert_eq!(tool["metadata"]["executionEnabled"], false);
        assert_eq!(
            tool["metadata"]["readiness"],
            "metadata-only; runtime execution is not enabled"
        );
    }
}

#[test]
fn jsonrpc_notification_probe_outputs_no_response() {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(HANDLE_JSONRPC_ONCE_FLAG)
        .arg(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn jsonrpc_malformed_json_probe_returns_parse_error_with_null_id() {
    let raw = run_jsonrpc_probe(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","#);
    let parsed: Value = serde_json::from_str(&raw).expect("parse error response should parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert!(parsed["id"].is_null());
    assert_eq!(parsed["error"]["code"], -32700);
    assert_eq!(parsed["error"]["message"], "Parse error");
}

#[test]
fn jsonrpc_invalid_request_probe_returns_invalid_request_error() {
    let raw = run_jsonrpc_probe(r#"{"jsonrpc":"2.0","id":2,"params":{}}"#);
    let parsed: Value = serde_json::from_str(&raw).expect("invalid request response should parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 2);
    assert_eq!(parsed["error"]["code"], -32600);
    assert_eq!(parsed["error"]["message"], "Invalid Request");
}

#[test]
fn jsonrpc_unknown_method_probe_returns_method_not_found_error() {
    let raw = run_jsonrpc_probe(r#"{"jsonrpc":"2.0","id":3,"method":"resources/list"}"#);
    let parsed: Value = serde_json::from_str(&raw).expect("unknown method response should parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 3);
    assert_eq!(parsed["error"]["code"], -32601);
    assert_eq!(parsed["error"]["message"], "Method not found");
}

#[test]
fn jsonrpc_tools_call_probe_is_rejected_without_execution() {
    let raw = run_jsonrpc_probe(
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"contacts.list","arguments":{}}}"#,
    );
    let parsed: Value = serde_json::from_str(&raw).expect("tools/call response should parse");

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 4);
    assert_eq!(parsed["error"]["code"], -32601);
    assert!(parsed["error"]["message"]
        .as_str()
        .expect("error message should be a string")
        .contains("tools/call execution is not implemented or enabled"));
}

fn run_catalog_flag(flag: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(flag)
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("stdout should be UTF-8")
        .trim_end()
        .to_string()
}

fn run_runtime_status_flag() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg("--print-runtime-status")
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("stdout should be UTF-8")
        .trim_end()
        .to_string()
}

fn run_runtime_status_from_config(path: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG)
        .arg(path)
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("stdout should be UTF-8")
        .trim_end()
        .to_string()
}

fn run_jsonrpc_probe(raw_json: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_crm-mcp"))
        .arg(HANDLE_JSONRPC_ONCE_FLAG)
        .arg(raw_json)
        .output()
        .expect("crm-mcp should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout)
        .expect("stdout should be UTF-8")
        .trim_end()
        .to_string()
}

fn assert_catalog_json(raw: &str) {
    let parsed: Value = serde_json::from_str(raw).expect("catalog JSON should parse");
    let tools = parsed.as_array().expect("catalog should be a JSON array");
    let names: Vec<&str> = tools
        .iter()
        .map(|tool| {
            tool.get("name")
                .and_then(Value::as_str)
                .expect("tool name should be a string")
        })
        .collect();

    assert_eq!(names, INITIAL_READ_TOOL_NAMES);

    for tool in tools {
        assert_eq!(
            tool.get("access_kind").and_then(Value::as_str),
            Some("read")
        );
        assert_eq!(
            tool.get("requires_external_client_permission")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(tool.get("sdk_backed").and_then(Value::as_bool), Some(true));
        assert_eq!(
            tool.get("runtime_enabled").and_then(Value::as_bool),
            Some(false)
        );
    }
}

fn assert_runtime_status_json(raw: &str) {
    let parsed: Value = serde_json::from_str(raw).expect("runtime status JSON should parse");
    let status = parsed
        .as_object()
        .expect("runtime status should be a JSON object");

    assert_eq!(status.get("enabled").and_then(Value::as_bool), Some(false));
    assert_eq!(
        status.get("bind_host").and_then(Value::as_str),
        Some("127.0.0.1")
    );
    assert_eq!(status.get("bind_port").and_then(Value::as_u64), Some(0));
    assert_eq!(status.get("serving").and_then(Value::as_bool), Some(false));
    assert_eq!(
        status
            .get("tool_execution_enabled")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        status.get("reason").and_then(Value::as_str),
        Some("runtime disabled")
    );
    assert_eq!(
        raw,
        r#"{
  "enabled": false,
  "bind_host": "127.0.0.1",
  "bind_port": 0,
  "serving": false,
  "tool_execution_enabled": false,
  "reason": "runtime disabled"
}"#
    );
}

fn write_temp_config(name: &str, contents: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "crm-mcp-runtime-config-{name}-{}-{unique}.json",
        std::process::id()
    ));
    fs::write(&path, contents).expect("temp runtime config should be writable");
    path
}
