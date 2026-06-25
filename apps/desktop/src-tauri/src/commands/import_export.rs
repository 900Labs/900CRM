use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crm_core::utils::csv::ImportColumnMapping;
use crm_core::{
    services::{ImportPreflightReport, ImportResult, LocalBackup},
    utils::csv::JsonImportPreview,
    CrmCore,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

static PRE_IMPORT_BACKUP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWithBackupResult {
    pub import: ImportResult,
    pub backup: LocalBackup,
}

#[tauri::command]
pub async fn import_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_csv(&file_path)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_contacts_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_csv_with_mapping(&file_path, mapping)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_json(&file_path)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn preview_contacts_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let core = super::lock_core(&state)?;
    core.preview_contacts_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_contacts_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_contacts_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_contacts_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_csv(&file_path).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_deals_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_csv_with_mapping(&file_path, mapping)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_deals_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_json(&file_path)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn preview_deals_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let core = super::lock_core(&state)?;
    core.preview_deals_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_deals_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_deals_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_deals_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_deals_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_deals_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_csv(&file_path)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_organizations_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_csv_with_mapping(&file_path, mapping)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn import_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportWithBackupResult, String> {
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_json(&file_path)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn preview_organizations_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let core = super::lock_core(&state)?;
    core.preview_organizations_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_organizations_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let core = super::lock_core(&state)?;
    core.preflight_organizations_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_organizations_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let core = super::lock_core(&state)?;
    core.export_organizations_json(&file_path)
        .map_err(|e| e.to_string())
}

fn import_with_pre_import_backup<F>(
    state: &State<'_, AppState>,
    import: F,
) -> Result<ImportWithBackupResult, String>
where
    F: FnOnce(&mut CrmCore) -> Result<ImportResult, String>,
{
    let mut core = super::lock_core(state)?;
    let backup_dir = next_pre_import_backup_dir(&state.data_dir)?;
    create_backup_then_import(&mut core, &backup_dir, import)
}

fn create_backup_then_import<F>(
    core: &mut CrmCore,
    backup_dir: &Path,
    import: F,
) -> Result<ImportWithBackupResult, String>
where
    F: FnOnce(&mut CrmCore) -> Result<ImportResult, String>,
{
    let backup = core
        .create_local_backup(backup_dir)
        .map_err(|e| e.to_string())?;
    let import = import(core)?;
    Ok(ImportWithBackupResult { import, backup })
}

fn next_pre_import_backup_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock is before UNIX_EPOCH: {}", e))?
        .as_millis();
    let sequence = PRE_IMPORT_BACKUP_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(app_data_dir
        .join("pre-import-backups")
        .join(format!("{}-{}", timestamp, sequence)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, panic, path::PathBuf};

    fn unique_test_dir(name: &str) -> PathBuf {
        let sequence = PRE_IMPORT_BACKUP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "900crm-pre-import-backup-{}-{}-{}",
            name,
            std::process::id(),
            sequence
        ))
    }

    fn write_csv(path: &Path, contents: &str) {
        fs::write(path, contents).expect("test CSV should be writable");
    }

    #[test]
    fn pre_import_backup_path_is_unique_under_app_data() {
        let app_data_dir = PathBuf::from("/tmp/900crm-app-data");
        let first = next_pre_import_backup_dir(&app_data_dir).expect("first path");
        let second = next_pre_import_backup_dir(&app_data_dir).expect("second path");

        assert_ne!(first, second);
        assert_eq!(
            first.parent().unwrap(),
            app_data_dir.join("pre-import-backups")
        );
        assert_eq!(
            second.parent().unwrap(),
            app_data_dir.join("pre-import-backups")
        );
    }

    #[test]
    fn backup_is_created_before_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-write");
        let csv_path = app_data_dir.join("contacts.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");
        write_csv(
            &csv_path,
            "first_name,last_name,email\nAda,Lovelace,ada@example.com\n",
        );

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let backup_dir = app_data_dir.join("pre-import-backups").join("before");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_contacts_csv(csv_path.to_str().expect("utf8 path"))
                .map_err(|e| e.to_string())
        })
        .expect("backup and import should succeed");

        assert_eq!(result.import.created, 1);
        assert!(backup_dir.join("900crm.db").is_file());
        assert_eq!(result.backup.backup_dir, backup_dir.to_string_lossy());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_contacts = backup_core
            .list_contacts(None)
            .expect("backup contacts list");
        assert_eq!(backup_contacts.contacts.len(), 0);

        let active_contacts = core.list_contacts(None).expect("active contacts list");
        assert_eq!(active_contacts.contacts.len(), 1);

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn failed_backup_prevents_import() {
        let app_data_dir = unique_test_dir("failed-backup");
        let backup_dir = app_data_dir.join("pre-import-backups").join("blocked");
        fs::create_dir_all(&backup_dir).expect("backup dir");
        fs::write(backup_dir.join("900crm.db"), b"existing").expect("existing db marker");

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            create_backup_then_import(&mut core, &backup_dir, |_core| {
                panic!("import must not run when backup fails")
            })
        }))
        .expect("helper should return an error, not panic");

        assert!(result
            .expect_err("backup should fail")
            .contains("already exists"));

        fs::remove_dir_all(&app_data_dir).ok();
    }
}
