/**
 * src/lib/api/settings.ts — Tauri IPC wrappers for app settings.
 */

import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  language: string;
  currency: string;
  theme: 'light' | 'dark' | 'system';
  dateFormat: string;
  syncEnabled: boolean;
  syncUrl: string;
  notificationsEnabled: boolean;
  reminderLeadMinutes: number;
}

export type SettingKey = keyof AppSettings;

interface BackendSetting {
  key: string;
  value: string;
  updated_at: string;
}

type BackendSettingsMap = Record<string, string>;

function toBackendKey(key: SettingKey): string {
  switch (key) {
    case 'dateFormat':
      return 'date_format';
    case 'syncEnabled':
      return 'sync_enabled';
    case 'syncUrl':
      return 'sync_url';
    case 'notificationsEnabled':
      return 'notifications_enabled';
    case 'reminderLeadMinutes':
      return 'reminder_lead_minutes';
    default:
      return key;
  }
}

function parseBoolean(value: string | undefined, fallback = false): boolean {
  if (value == null) {
    return fallback;
  }
  return value === 'true' || value === '1';
}

function parseTheme(value: string | undefined): AppSettings['theme'] {
  if (value === 'light' || value === 'dark' || value === 'system') {
    return value;
  }
  return 'system';
}

function parseInteger(value: string | undefined, fallback: number): number {
  if (value == null || value.trim() === '') {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function mapSettings(map: BackendSettingsMap): AppSettings {
  return {
    language: map.language || 'en',
    currency: map.currency || 'USD',
    theme: parseTheme(map.theme),
    dateFormat: map.date_format || 'MMM D, YYYY',
    syncEnabled: parseBoolean(map.sync_enabled, false),
    syncUrl: map.sync_url || '',
    notificationsEnabled: parseBoolean(map.notifications_enabled, true),
    reminderLeadMinutes: parseInteger(map.reminder_lead_minutes, 30),
  };
}

function serializeValue<K extends SettingKey>(key: K, value: AppSettings[K]): string {
  if (key === 'syncEnabled' || key === 'notificationsEnabled') {
    return value ? 'true' : 'false';
  }
  if (key === 'reminderLeadMinutes') {
    return String(Math.max(1, Number(value) || 30));
  }
  return String(value ?? '');
}

function parseSettingValue<K extends SettingKey>(key: K, value: string | undefined): AppSettings[K] {
  switch (key) {
    case 'syncEnabled':
      return parseBoolean(value, false) as AppSettings[K];
    case 'theme':
      return parseTheme(value) as AppSettings[K];
    case 'dateFormat':
      return (value || 'MMM D, YYYY') as AppSettings[K];
    case 'currency':
      return (value || 'USD') as AppSettings[K];
    case 'language':
      return (value || 'en') as AppSettings[K];
    case 'syncUrl':
      return (value || '') as AppSettings[K];
    case 'notificationsEnabled':
      return parseBoolean(value, true) as AppSettings[K];
    case 'reminderLeadMinutes':
      return parseInteger(value, 30) as AppSettings[K];
    default:
      return value as AppSettings[K];
  }
}

export async function getSettings(): Promise<AppSettings> {
  const settings = await invoke<BackendSettingsMap>('get_settings');
  return mapSettings(settings);
}

export async function updateSetting<K extends SettingKey>(
  key: K,
  value: AppSettings[K]
): Promise<void> {
  await invoke<BackendSetting>('update_setting', {
    key: toBackendKey(key),
    value: serializeValue(key, value),
  });
}

export async function getSetting<K extends SettingKey>(key: K): Promise<AppSettings[K]> {
  const setting = await invoke<BackendSetting | null>('get_setting', {
    key: toBackendKey(key),
  });

  return parseSettingValue(key, setting?.value);
}
