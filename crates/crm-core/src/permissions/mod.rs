//! Permission primitives for optional external clients.

pub const DEFAULT_EXTERNAL_CLIENT_PERMISSION_MODE: &str = "disabled";

pub fn is_supported_initial_permission_mode(mode: &str) -> bool {
    matches!(mode, "disabled" | "read_only" | "draft_only")
}
