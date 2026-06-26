use std::process::Command;

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
