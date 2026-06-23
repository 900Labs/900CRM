use crate::result::CrmResult;
use crate::storage::{self, audit::AuditLogEntry};

use super::CrmCore;

impl CrmCore {
    pub fn list_recent_audit_log(&self, limit: u32) -> CrmResult<Vec<AuditLogEntry>> {
        storage::audit::list_recent_audit_log(&self.db.conn, limit)
    }
}
