/**
 * src/lib/api/sync.ts — Tauri IPC wrappers for data synchronization.
 *
 * @module api/sync
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Sync operation status. */
export type SyncState = 'idle' | 'syncing' | 'error' | 'success';

/** Current sync status. */
export interface SyncStatus {
  state: SyncState;
  lastSyncAt: string | null;
  errorMessage: string | null;
  pendingChanges: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Trigger a manual sync with the configured server.
 *
 * @returns SyncStatus after the sync completes
 */
export async function triggerSync(): Promise<SyncStatus> {
  return invoke<SyncStatus>('trigger_sync');
}

/**
 * Get the current sync status without triggering a new sync.
 *
 * @returns Current SyncStatus
 */
export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke<SyncStatus>('get_sync_status');
}
