import { invoke } from '@tauri-apps/api/core';

export interface LocalBackupMetadata {
  backupFormatVersion: number;
  createdAt: string;
  appVersion: string;
  schemaVersion: number;
  deviceId: string;
  databaseFile: string;
}

export interface LocalBackup {
  backupDir: string;
  databasePath: string;
  metadataPath: string;
  metadata: LocalBackupMetadata;
}

export interface LocalBackupValidation {
  backupDir: string;
  databasePath: string;
  metadataPath: string;
  metadata: LocalBackupMetadata;
}

export interface LocalRestoreResult {
  restoredAt: string;
  databasePath: string;
  metadata: LocalBackupMetadata;
}

interface BackendLocalBackupMetadata {
  backup_format_version: number;
  created_at: string;
  app_version: string;
  schema_version: number;
  device_id: string;
  database_file: string;
}

interface BackendLocalBackup {
  backup_dir: string;
  database_path: string;
  metadata_path: string;
  metadata: BackendLocalBackupMetadata;
}

type BackendLocalBackupValidation = BackendLocalBackup;

interface BackendLocalRestoreResult {
  restored_at: string;
  database_path: string;
  metadata: BackendLocalBackupMetadata;
}

function mapMetadata(metadata: BackendLocalBackupMetadata): LocalBackupMetadata {
  return {
    backupFormatVersion: metadata.backup_format_version,
    createdAt: metadata.created_at,
    appVersion: metadata.app_version,
    schemaVersion: metadata.schema_version,
    deviceId: metadata.device_id,
    databaseFile: metadata.database_file,
  };
}

function mapBackup(backup: BackendLocalBackup): LocalBackup {
  return {
    backupDir: backup.backup_dir,
    databasePath: backup.database_path,
    metadataPath: backup.metadata_path,
    metadata: mapMetadata(backup.metadata),
  };
}

function mapValidation(validation: BackendLocalBackupValidation): LocalBackupValidation {
  return {
    backupDir: validation.backup_dir,
    databasePath: validation.database_path,
    metadataPath: validation.metadata_path,
    metadata: mapMetadata(validation.metadata),
  };
}

function mapRestoreResult(result: BackendLocalRestoreResult): LocalRestoreResult {
  return {
    restoredAt: result.restored_at,
    databasePath: result.database_path,
    metadata: mapMetadata(result.metadata),
  };
}

export async function createLocalBackup(backupDir: string): Promise<LocalBackup> {
  const backup = await invoke<BackendLocalBackup>('create_local_backup', {
    backup_dir: backupDir,
  });
  return mapBackup(backup);
}

export async function validateLocalBackup(backupDir: string): Promise<LocalBackupValidation> {
  const validation = await invoke<BackendLocalBackupValidation>('validate_local_backup', {
    backup_dir: backupDir,
  });
  return mapValidation(validation);
}

export async function restoreLocalBackupToAppData(
  backupDir: string,
  confirmDestructiveRestore: boolean
): Promise<LocalRestoreResult> {
  const result = await invoke<BackendLocalRestoreResult>('restore_local_backup_to_app_data', {
    backup_dir: backupDir,
    confirm_destructive_restore: confirmDestructiveRestore,
  });
  return mapRestoreResult(result);
}
