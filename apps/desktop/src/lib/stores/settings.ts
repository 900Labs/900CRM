/**
 * src/lib/stores/settings.ts — Application settings store for 900CRM.
 *
 * Manages language, currency, theme, date format, and sync configuration.
 * Persists settings via the Tauri backend (SQLite).
 *
 * @module stores/settings
 */

import { getSettings, updateSetting } from '$lib/api/settings';
import type { AppSettings } from '$lib/api/settings';
import { chooseLocale, initI18n } from '$lib/i18n/bootstrap';
import { uiStore } from './ui';

// ─────────────────────────────────────────────────────────────────────────────
// SettingsStore
// ─────────────────────────────────────────────────────────────────────────────

class SettingsStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** ISO 639-1 locale code. */
  language = $state<string>('en');

  /** ISO 4217 currency code. */
  currency = $state<string>('USD');

  /** UI theme. */
  theme = $state<'light' | 'dark' | 'system'>('system');

  /** Date display format. */
  dateFormat = $state<string>('MMM D, YYYY');

  /** Whether background sync is enabled. */
  syncEnabled = $state<boolean>(false);

  /** URL of the sync server. */
  syncUrl = $state<string>('');

  /** Whether settings have been loaded from the backend. */
  loaded = $state<boolean>(false);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load all settings from the backend and apply them to the UI.
   * Call once on app startup.
   */
  async loadSettings(): Promise<void> {
    try {
      await initI18n();
      const data: AppSettings = await getSettings();
      this.language   = data.language;
      this.currency   = data.currency;
      this.theme      = data.theme;
      this.dateFormat = data.dateFormat;
      this.syncEnabled = data.syncEnabled;
      this.syncUrl    = data.syncUrl;
      this.loaded     = true;

      // Apply immediately
      this.applyTheme(data.theme);
      await chooseLocale(data.language);
    } catch (err) {
      uiStore.toastError('Failed to load settings');
      // Apply defaults
      this.applyTheme(this.theme);
      await chooseLocale(this.language);
      this.loaded = true;
    }
  }

  /**
   * Update a single setting. Persists to backend and applies immediately.
   *
   * @param key    Setting key
   * @param value  New value
   */
  async updateSetting<K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ): Promise<void> {
    const old = this[key as keyof SettingsStore];

    try {
      // Optimistic update
      (this as Record<string, unknown>)[key] = value;

      await updateSetting(key, value);

      // Apply side effects
      if (key === 'theme') {
        this.applyTheme(value as AppSettings['theme']);
      }
      if (key === 'language') {
        await chooseLocale(value as string);
      }
    } catch (err) {
      // Rollback on failure
      (this as Record<string, unknown>)[key] = old;
      uiStore.toastError(`Failed to save setting: ${key}`);
      throw err;
    }
  }

  /**
   * Apply the theme to the document element.
   * @param theme  'light' | 'dark' | 'system'
   */
  applyTheme(theme: 'light' | 'dark' | 'system'): void {
    if (typeof document === 'undefined') return;
    document.documentElement.setAttribute('data-theme', theme);
  }
}

/** Singleton settings store. */
export const settingsStore = new SettingsStore();
