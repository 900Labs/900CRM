use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionMode {
    Disabled,
    ReadOnly,
    DraftOnly,
    WriteWithConfirmation,
    WriteAllowed,
}
