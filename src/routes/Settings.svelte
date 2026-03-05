<script lang="ts">
  /**
   * Settings.svelte — Application settings page for 900CRM.
   *
   * Sections:
   *   1. Language — pick from 6 locales (shows nativeName)
   *   2. Currency — ISO 4217 selector
   *   3. Theme — light / dark / system
   *   4. Date format — 4 preset formats
   *   5. Sync — enable/disable toggle + server URL
   *   6. About — 900 Labs mission statement
   *   7. Data management — export all, import data
   *
   * Each setting auto-saves via settingsStore.updateSetting().
   * No separate Save button needed — changes apply immediately.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { settingsStore } from '$lib/stores/settings';
  import { availableLocales } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';
  import type { AppSettings } from '$lib/api/settings';

  // ── Types ────────────────────────────────────────────────────────────────────

  type ThemeOption = 'light' | 'dark' | 'system';
  type DateFormat  = 'YYYY-MM-DD' | 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'MMM D, YYYY';

  // ── Constants ────────────────────────────────────────────────────────────────

  const CURRENCIES = [
    { code: 'USD', label: 'USD — US Dollar' },
    { code: 'EUR', label: 'EUR — Euro' },
    { code: 'GBP', label: 'GBP — British Pound' },
    { code: 'JPY', label: 'JPY — Japanese Yen' },
    { code: 'KES', label: 'KES — Kenyan Shilling' },
    { code: 'NGN', label: 'NGN — Nigerian Naira' },
    { code: 'ZAR', label: 'ZAR — South African Rand' },
    { code: 'INR', label: 'INR — Indian Rupee' },
    { code: 'BRL', label: 'BRL — Brazilian Real' },
    { code: 'MXN', label: 'MXN — Mexican Peso' },
    { code: 'EGP', label: 'EGP — Egyptian Pound' },
    { code: 'SAR', label: 'SAR — Saudi Riyal' },
    { code: 'AED', label: 'AED — UAE Dirham' },
    { code: 'CAD', label: 'CAD — Canadian Dollar' },
    { code: 'AUD', label: 'AUD — Australian Dollar' },
  ];

  const THEME_OPTIONS: { value: ThemeOption; labelKey: string }[] = [
    { value: 'light',  labelKey: 'settings.themeLight' },
    { value: 'dark',   labelKey: 'settings.themeDark' },
    { value: 'system', labelKey: 'settings.themeSystem' },
  ];

  const DATE_FORMATS: { value: DateFormat; example: string }[] = [
    { value: 'MMM D, YYYY',  example: 'Mar 5, 2026' },
    { value: 'MM/DD/YYYY',   example: '03/05/2026' },
    { value: 'DD/MM/YYYY',   example: '05/03/2026' },
    { value: 'YYYY-MM-DD',   example: '2026-03-05' },
  ];

  // ── State ────────────────────────────────────────────────────────────────────

  /** Tracks which setting key is currently being saved (for per-row spinner). */
  let savingKey = $state<keyof AppSettings | null>(null);

  /** URL field local state — only committed on blur or Enter. */
  let syncUrlLocal = $state('');
  let syncUrlDirty = $state(false);

  let exportLoading = $state(false);

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(() => {
    syncUrlLocal = settingsStore.syncUrl;
  });

  // ── Handlers ─────────────────────────────────────────────────────────────────

  /**
   * Generic setting updater — shows per-key saving indicator.
   */
  async function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    savingKey = key;
    try {
      await settingsStore.updateSetting(key, value);
      uiStore.toastSuccess(t('settings.settingsSaved'));
    } catch {
      // already toasted by store
    } finally {
      savingKey = null;
    }
  }

  async function handleLanguageChange(code: string) {
    await updateSetting('language', code);
  }

  async function handleCurrencyChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    await updateSetting('currency', select.value);
  }

  async function handleThemeChange(theme: ThemeOption) {
    await updateSetting('theme', theme);
  }

  async function handleDateFormatChange(format: string) {
    await updateSetting('dateFormat', format);
  }

  async function handleSyncToggle() {
    await updateSetting('syncEnabled', !settingsStore.syncEnabled);
  }

  function handleSyncUrlInput(e: Event) {
    syncUrlLocal = (e.target as HTMLInputElement).value;
    syncUrlDirty = true;
  }

  async function handleSyncUrlCommit() {
    if (!syncUrlDirty) return;
    syncUrlDirty = false;
    await updateSetting('syncUrl', syncUrlLocal);
  }

  async function handleSyncUrlKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      (e.target as HTMLInputElement).blur();
      await handleSyncUrlCommit();
    }
  }

  // ── Data management ──────────────────────────────────────────────────────────

  async function handleExportAll() {
    exportLoading = true;
    try {
      // Trigger export via Tauri (backend handles file dialog)
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('export_all_data');
      uiStore.toastSuccess(t('export.success'));
    } catch {
      uiStore.toastError(t('export.failed'));
    } finally {
      exportLoading = false;
    }
  }

  function handleImportData() {
    uiStore.openModal('importExport');
  }
</script>

<div class="page-content settings-page">
  <div class="page-header">
    <h1 class="page-title">{t('settings.title')}</h1>
  </div>

  <div class="settings-grid">

    <!-- ── LEFT: app settings ───────────────────────────────────────────────── -->
    <div class="settings-main">

      <!-- Language -->
      <section class="card settings-section" aria-labelledby="lang-heading">
        <div class="card-header">
          <h2 class="section-title" id="lang-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/>
            </svg>
            {t('settings.language')}
          </h2>
          {#if savingKey === 'language'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="locale-grid" role="radiogroup" aria-labelledby="lang-heading">
            {#each availableLocales as locale (locale.code)}
              <button
                class="locale-option"
                class:locale-option--active={settingsStore.language === locale.code}
                onclick={() => handleLanguageChange(locale.code)}
                role="radio"
                aria-checked={settingsStore.language === locale.code}
                type="button"
              >
                <span class="locale-native">{locale.nativeName}</span>
                <span class="locale-name">{locale.name}</span>
                {#if settingsStore.language === locale.code}
                  <span class="locale-check" aria-hidden="true">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                      <polyline points="20 6 9 17 4 12"/>
                    </svg>
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Currency -->
      <section class="card settings-section" aria-labelledby="currency-heading">
        <div class="card-header">
          <h2 class="section-title" id="currency-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/>
            </svg>
            {t('settings.currency')}
          </h2>
          {#if savingKey === 'currency'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="field-row">
            <label class="field-label" for="currency-select">{t('settings.currency')}</label>
            <select
              id="currency-select"
              class="input select-input"
              value={settingsStore.currency}
              onchange={handleCurrencyChange}
            >
              {#each CURRENCIES as c (c.code)}
                <option value={c.code}>{c.label}</option>
              {/each}
            </select>
          </div>
        </div>
      </section>

      <!-- Theme -->
      <section class="card settings-section" aria-labelledby="theme-heading">
        <div class="card-header">
          <h2 class="section-title" id="theme-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
            {t('settings.theme')}
          </h2>
          {#if savingKey === 'theme'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="theme-options" role="radiogroup" aria-labelledby="theme-heading">
            {#each THEME_OPTIONS as opt (opt.value)}
              <button
                class="theme-option"
                class:theme-option--active={settingsStore.theme === opt.value}
                onclick={() => handleThemeChange(opt.value)}
                role="radio"
                aria-checked={settingsStore.theme === opt.value}
                type="button"
              >
                <!-- Theme icon -->
                {#if opt.value === 'light'}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                    <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                  </svg>
                {:else if opt.value === 'dark'}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
                  </svg>
                {:else}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
                  </svg>
                {/if}
                <span>{t(opt.labelKey)}</span>
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Date format -->
      <section class="card settings-section" aria-labelledby="dateformat-heading">
        <div class="card-header">
          <h2 class="section-title" id="dateformat-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            {t('settings.dateFormat')}
          </h2>
          {#if savingKey === 'dateFormat'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="date-format-options" role="radiogroup" aria-labelledby="dateformat-heading">
            {#each DATE_FORMATS as fmt (fmt.value)}
              <button
                class="date-format-option"
                class:date-format-option--active={settingsStore.dateFormat === fmt.value}
                onclick={() => handleDateFormatChange(fmt.value)}
                role="radio"
                aria-checked={settingsStore.dateFormat === fmt.value}
                type="button"
              >
                <code class="format-code">{fmt.value}</code>
                <span class="format-example">{fmt.example}</span>
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Sync configuration -->
      <section class="card settings-section" aria-labelledby="sync-heading">
        <div class="card-header">
          <h2 class="section-title" id="sync-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
            {t('settings.sync')}
          </h2>
          {#if savingKey === 'syncEnabled' || savingKey === 'syncUrl'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body sync-body">
          <!-- Sync enable toggle -->
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">{t('settings.syncEnabled')}</span>
              <span class="toggle-desc">
                {settingsStore.syncEnabled
                  ? t('settings.sync') + ' ' + t('common.success').toLowerCase()
                  : t('settings.sync') + ' ' + t('common.none').toLowerCase()}
              </span>
            </div>
            <button
              class="toggle-switch"
              class:toggle-switch--on={settingsStore.syncEnabled}
              onclick={handleSyncToggle}
              role="switch"
              aria-checked={settingsStore.syncEnabled}
              aria-label={t('settings.syncEnabled')}
              type="button"
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          <!-- Sync URL (shown only when sync is enabled) -->
          {#if settingsStore.syncEnabled}
            <div class="field-row sync-url-row">
              <label class="field-label" for="sync-url">{t('settings.syncUrl')}</label>
              <div class="sync-url-input-wrap">
                <input
                  id="sync-url"
                  class="input"
                  type="url"
                  value={syncUrlLocal}
                  oninput={handleSyncUrlInput}
                  onblur={handleSyncUrlCommit}
                  onkeydown={handleSyncUrlKeydown}
                  placeholder="https://sync.example.com"
                  autocomplete="url"
                  spellcheck={false}
                />
              </div>
            </div>
          {/if}
        </div>
      </section>
    </div>

    <!-- ── RIGHT: about + data management ─────────────────────────────────── -->
    <div class="settings-sidebar">

      <!-- About / Mission -->
      <section class="card settings-section settings-about" aria-labelledby="about-heading">
        <div class="card-header">
          <h2 class="section-title" id="about-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {t('settings.about')}
          </h2>
        </div>
        <div class="card-body about-body">
          <!-- App logo mark -->
          <div class="about-logo" aria-hidden="true">
            <svg width="36" height="36" viewBox="0 0 36 36" fill="none" aria-hidden="true">
              <rect width="36" height="36" rx="8" fill="#20808D"/>
              <path d="M9 18C9 13.03 13.03 9 18 9C22.97 9 27 13.03 27 18" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M18 27C15.24 27 12.76 25.76 11.1 23.8" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
              <circle cx="18" cy="18" r="3" fill="white"/>
            </svg>
          </div>

          <div class="about-app">
            <strong class="about-appname">{t('app.title')}</strong>
            <span class="about-tagline">{t('app.tagline')}</span>
          </div>

          <!-- Mission -->
          <div class="mission-block">
            <h3 class="mission-title">{t('mission.title')}</h3>
            <p class="mission-desc">{t('mission.description')}</p>
          </div>

          <!-- Version -->
          <div class="about-meta">
            <span class="about-meta-item">v1.0.0</span>
            <span class="about-meta-item">900 Labs</span>
            <a
              class="about-meta-link"
              href="https://github.com/900labs/900crm"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
          </div>
        </div>
      </section>

      <!-- Data management -->
      <section class="card settings-section" aria-labelledby="data-heading">
        <div class="card-header">
          <h2 class="section-title" id="data-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
              <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
            </svg>
            {t('settings.dataManagement')}
          </h2>
        </div>
        <div class="card-body data-body">
          <div class="data-action">
            <div class="data-action-info">
              <span class="data-action-label">{t('settings.exportAll')}</span>
              <span class="data-action-desc">{t('export.title')}</span>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              onclick={handleExportAll}
              disabled={exportLoading}
              type="button"
            >
              {#if exportLoading}
                {t('export.exporting')}
              {:else}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                {t('settings.exportAll')}
              {/if}
            </button>
          </div>

          <div class="data-action">
            <div class="data-action-info">
              <span class="data-action-label">{t('settings.importData')}</span>
              <span class="data-action-desc">{t('import.title')}</span>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              onclick={handleImportData}
              type="button"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
              </svg>
              {t('settings.importData')}
            </button>
          </div>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .settings-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  /* ── Two-column grid ─────────────────────────────────────────────────────── */

  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: var(--space-6);
    align-items: start;
  }

  @media (max-width: 900px) {
    .settings-grid {
      grid-template-columns: 1fr;
    }
  }

  /* ── Columns ─────────────────────────────────────────────────────────────── */

  .settings-main,
  .settings-sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  /* ── Sections ────────────────────────────────────────────────────────────── */

  .settings-section .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .saving-indicator {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* ── Language grid ───────────────────────────────────────────────────────── */

  .locale-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: var(--space-3);
  }

  .locale-option {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-4) var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    text-align: start;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .locale-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
  }

  .locale-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
  }

  .locale-native {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .locale-name {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .locale-check {
    position: absolute;
    inset-inline-end: var(--space-3);
    top: var(--space-3);
    color: var(--color-primary);
  }

  /* ── Currency selector ───────────────────────────────────────────────────── */

  .field-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .select-input {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2313343B' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--space-3) center;
    padding-inline-end: var(--space-8);
    cursor: pointer;
  }

  [data-theme="dark"] .select-input {
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23FCFAF6' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
  }

  /* ── Theme options ───────────────────────────────────────────────────────── */

  .theme-options {
    display: flex;
    gap: var(--space-3);
  }

  .theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-5) var(--space-3);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out);
  }

  .theme-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
    color: var(--text-primary);
  }

  .theme-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
    color: var(--text-accent);
  }

  /* ── Date format options ─────────────────────────────────────────────────── */

  .date-format-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .date-format-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    text-align: start;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .date-format-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
  }

  .date-format-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
  }

  .format-code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    background: none;
  }

  .format-example {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* ── Sync section ────────────────────────────────────────────────────────── */

  .sync-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .toggle-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* Toggle switch */
  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 9999px;
    background-color: var(--border-default);
    border: none;
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .toggle-switch--on {
    background-color: var(--color-primary);
  }

  .toggle-thumb {
    position: absolute;
    top: 3px;
    inset-inline-start: 3px;
    width: 18px;
    height: 18px;
    background-color: white;
    border-radius: 50%;
    transition: inset-inline-start var(--duration-fast) var(--ease-out),
                transform var(--duration-fast) var(--ease-out);
    box-shadow: 0 1px 3px rgba(0,0,0,0.15);
  }

  .toggle-switch--on .toggle-thumb {
    inset-inline-start: calc(100% - 21px);
  }

  .sync-url-row {
    padding-block-start: var(--space-2);
  }

  .sync-url-input-wrap {
    width: 100%;
  }

  /* ── About section ───────────────────────────────────────────────────────── */

  .about-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .about-logo {
    display: flex;
    align-items: center;
  }

  .about-app {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .about-appname {
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
  }

  .about-tagline {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .mission-block {
    padding: var(--space-4) var(--space-5);
    background-color: var(--surface-raised);
    border-radius: var(--radius-md);
    border-inline-start: 3px solid var(--color-primary);
  }

  .mission-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    margin: 0 0 var(--space-2) 0;
  }

  .mission-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.6;
  }

  .about-meta {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
    padding-block-start: var(--space-2);
    border-block-start: var(--border-width) solid var(--border-default);
  }

  .about-meta-item {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .about-meta-link {
    font-size: var(--text-xs);
    color: var(--text-accent);
    text-decoration: none;
  }

  .about-meta-link:hover {
    text-decoration: underline;
  }

  /* ── Data management ─────────────────────────────────────────────────────── */

  .data-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .data-action {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
  }

  .data-action-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .data-action-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .data-action-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }
</style>
