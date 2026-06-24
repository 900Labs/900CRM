/**
 * src/lib/api/externalClients.ts - Tauri IPC wrappers for disabled external client placeholders.
 */

import { invoke } from '@tauri-apps/api/core';

export interface ExternalClient {
  id: string;
  name: string;
  clientType: string;
  permissionMode: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

interface BackendExternalClient {
  id: string;
  name: string;
  client_type: string;
  permission_mode: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

function mapExternalClient(client: BackendExternalClient): ExternalClient {
  return {
    id: client.id,
    name: client.name,
    clientType: client.client_type,
    permissionMode: client.permission_mode,
    enabled: client.enabled,
    createdAt: client.created_at,
    updatedAt: client.updated_at,
    deletedAt: client.deleted_at,
    deviceId: client.device_id,
  };
}

export async function listExternalClients(): Promise<ExternalClient[]> {
  const clients = await invoke<BackendExternalClient[]>('list_external_clients');
  return clients.map(mapExternalClient);
}

export async function createExternalClientPlaceholder(
  name: string,
  clientType: string,
): Promise<ExternalClient> {
  const client = await invoke<BackendExternalClient>('create_external_client_placeholder', {
    name,
    client_type: clientType,
  });

  return mapExternalClient(client);
}
