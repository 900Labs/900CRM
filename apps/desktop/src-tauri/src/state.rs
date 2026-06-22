use std::path::PathBuf;
use std::sync::Mutex;

use crm_core::CrmCore;

pub struct AppState {
    pub core: Mutex<CrmCore>,
    pub data_dir: PathBuf,
}
