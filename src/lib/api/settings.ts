/**
 * src/lib/api/settings.ts — Tauri IPC wrappers for application settings.
 *
 * @module api/settings
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** All persisted application settings. */
export interface AppSettings {
  language: string;
  currency: string;
  theme: 'light' | 'dark' | 'system';
  dateFormat: string;
  syncEnabled: boolean;
  syncUrl: string;
}

/** Key-value pair for a single setting. */
export type SettingKey = keyof AppSettings;

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Fetch all settings.
 *
 * @returns Full AppSettings object
 */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

/**
 * Update a single setting by key.
 *
 * @param key    The setting key
 * @param value  The new value
 */
export async function updateSetting<K extends SettingKey>(
  key: K,
  value: AppSettings[K]
): Promise<void> {
  return invoke<void>('update_setting', { key, value });
}

/**
 * Fetch a single setting by key.
 *
 * @param key  The setting key
 * @returns    The current value for that key
 */
export async function getSetting<K extends SettingKey>(key: K): Promise<AppSettings[K]> {
  return invoke<AppSettings[K]>('get_setting', { key });
}
