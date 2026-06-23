import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createLocalBackup,
  restoreLocalBackupToAppData,
  validateLocalBackup,
} from './backup';

const backendMetadata = {
  backup_format_version: 1,
  created_at: '2026-06-23T08:00:00Z',
  app_version: '1.0.0',
  schema_version: 3,
  device_id: 'device-1',
  database_file: '900crm.db',
};

const backendBackup = {
  backup_dir: '/tmp/backup',
  database_path: '/tmp/backup/900crm.db',
  metadata_path: '/tmp/backup/metadata.json',
  metadata: backendMetadata,
};

describe('backup api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('creates a local backup through the Tauri command', async () => {
    invokeMock.mockResolvedValue(backendBackup);

    const backup = await createLocalBackup('/tmp/backup');

    expect(invokeMock).toHaveBeenCalledWith('create_local_backup', {
      backup_dir: '/tmp/backup',
    });
    expect(backup).toEqual({
      backupDir: '/tmp/backup',
      databasePath: '/tmp/backup/900crm.db',
      metadataPath: '/tmp/backup/metadata.json',
      metadata: {
        backupFormatVersion: 1,
        createdAt: '2026-06-23T08:00:00Z',
        appVersion: '1.0.0',
        schemaVersion: 3,
        deviceId: 'device-1',
        databaseFile: '900crm.db',
      },
    });
  });

  it('validates a local backup through the Tauri command', async () => {
    invokeMock.mockResolvedValue(backendBackup);

    const validation = await validateLocalBackup('/tmp/backup');

    expect(invokeMock).toHaveBeenCalledWith('validate_local_backup', {
      backup_dir: '/tmp/backup',
    });
    expect(validation.metadata.schemaVersion).toBe(3);
    expect(validation.databasePath).toBe('/tmp/backup/900crm.db');
  });

  it('passes explicit restore confirmation to the Tauri command', async () => {
    invokeMock.mockResolvedValue({
      restored_at: '2026-06-23T08:01:00Z',
      database_path: '/tmp/app/900crm.db',
      metadata: backendMetadata,
    });

    const result = await restoreLocalBackupToAppData('/tmp/backup', true);

    expect(invokeMock).toHaveBeenCalledWith('restore_local_backup_to_app_data', {
      backup_dir: '/tmp/backup',
      confirm_destructive_restore: true,
    });
    expect(result).toEqual({
      restoredAt: '2026-06-23T08:01:00Z',
      databasePath: '/tmp/app/900crm.db',
      metadata: {
        backupFormatVersion: 1,
        createdAt: '2026-06-23T08:00:00Z',
        appVersion: '1.0.0',
        schemaVersion: 3,
        deviceId: 'device-1',
        databaseFile: '900crm.db',
      },
    });
  });
});
