import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createExternalClientPlaceholder,
  listExternalClients,
  type ExternalClient,
} from './externalClients';

const backendClient = {
  id: 'client-1',
  name: 'Claude Desktop',
  client_type: 'mcp',
  permission_mode: 'disabled',
  enabled: false,
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T08:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

const client: ExternalClient = {
  id: 'client-1',
  name: 'Claude Desktop',
  clientType: 'mcp',
  permissionMode: 'disabled',
  enabled: false,
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

describe('external clients API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps listExternalClients to list_external_clients', async () => {
    invokeMock.mockResolvedValueOnce([backendClient]);

    await expect(listExternalClients()).resolves.toEqual([client]);

    expect(invokeMock).toHaveBeenCalledWith('list_external_clients');
  });

  it('maps createExternalClientPlaceholder to create_external_client_placeholder', async () => {
    invokeMock.mockResolvedValueOnce(backendClient);

    const created = await createExternalClientPlaceholder('Claude Desktop', 'mcp');

    expect(created).toEqual(client);

    expect(invokeMock).toHaveBeenCalledWith('create_external_client_placeholder', {
      name: 'Claude Desktop',
      client_type: 'mcp',
    });
    expect(created.enabled).toBe(false);
    expect(created.permissionMode).toBe('disabled');
  });
});
