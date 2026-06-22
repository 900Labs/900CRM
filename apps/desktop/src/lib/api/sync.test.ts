import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { getSyncStatus, triggerSync } from './sync';

describe('sync api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps triggerSync response shape', async () => {
    invokeMock.mockResolvedValue({
      state: 'syncing',
      last_sync_at: '2026-03-05T00:00:00Z',
      error_message: null,
      pending_changes: 7,
    });

    const status = await triggerSync();

    expect(invokeMock).toHaveBeenCalledWith('trigger_sync');
    expect(status).toEqual({
      state: 'syncing',
      lastSyncAt: '2026-03-05T00:00:00Z',
      errorMessage: null,
      pendingChanges: 7,
    });
  });

  it('maps getSyncStatus response shape', async () => {
    invokeMock.mockResolvedValue({
      state: 'error',
      last_sync_at: null,
      error_message: 'Network unavailable',
      pending_changes: 3,
    });

    const status = await getSyncStatus();

    expect(invokeMock).toHaveBeenCalledWith('get_sync_status');
    expect(status).toEqual({
      state: 'error',
      lastSyncAt: null,
      errorMessage: 'Network unavailable',
      pendingChanges: 3,
    });
  });
});
