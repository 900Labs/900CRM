<script lang="ts">
  /**
   * ImportExport.svelte — Import/Export modal for contacts and deals CSV.
   */

  import { t } from '$lib/i18n';
  import { parseCSV } from '$lib/utils/csv';
  import type { ParseCSVResult } from '$lib/utils/csv';
  import { uiStore } from '$lib/stores/ui';
  import { invoke } from '@tauri-apps/api/core';

  let {
    open = $bindable(false),
    onclose,
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  let activeTab = $state<'import' | 'export'>('import');

  let csvText = $state('');
  let parseResult = $state<ParseCSVResult | null>(null);
  let selectedImportPath = $state<string | null>(null);
  let isImporting = $state(false);
  let importEntity = $state<'contacts' | 'deals'>('contacts');

  let exportEntity = $state<'contacts' | 'deals'>('contacts');
  let exportFormat = $state<'csv' | 'json'>('csv');
  let isExporting = $state(false);

  const previewRows = $derived(parseResult?.rows.slice(0, 5) ?? []);

  async function handleFilePick() {
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = await openDialog({
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        multiple: false,
      });

      if (typeof selected !== 'string') {
        return;
      }

      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      selectedImportPath = selected;
      csvText = await readTextFile(selected);
      parseResult = parseCSV(csvText);
    } catch {
      document.getElementById('csv-file-input')?.click();
    }
  }

  function handleFileInputChange(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) {
      return;
    }

    selectedImportPath = file.name;
    const reader = new FileReader();
    reader.onload = (ev) => {
      csvText = (ev.target?.result as string) ?? '';
      parseResult = parseCSV(csvText);
    };
    reader.readAsText(file);
  }

  async function handleImport() {
    if (!selectedImportPath) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }

    isImporting = true;
    try {
      const command = importEntity === 'contacts' ? 'import_contacts_csv' : 'import_deals_csv';
      const result = await invoke<{ created: number; skipped: number; errors: string[] }>(command, {
        file_path: selectedImportPath,
      });

      if (result.skipped > 0) {
        uiStore.toastWarning(`${t('import.success')} (${result.created} created, ${result.skipped} skipped)`);
      } else {
        uiStore.toastSuccess(`${t('import.success')} (${result.created})`);
      }

      close();
    } catch {
      uiStore.toastError(t('import.failed'));
    } finally {
      isImporting = false;
    }
  }

  async function handleExport() {
    if (exportFormat !== 'csv') {
      uiStore.toastWarning('JSON export is not available yet.');
      return;
    }

    isExporting = true;
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const savePath = await save({
        defaultPath: `${exportEntity}-export.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });

      if (typeof savePath !== 'string') {
        return;
      }

      const command = exportEntity === 'contacts' ? 'export_contacts_csv' : 'export_deals_csv';
      const rows = await invoke<number>(command, { file_path: savePath });
      uiStore.toastSuccess(`${t('export.success')} (${rows})`);
    } catch {
      uiStore.toastError(t('export.failed'));
    } finally {
      isExporting = false;
    }
  }

  function close() {
    open = false;
    onclose?.();
  }
</script>

{#if open}
  <div class="modal-backdrop" role="button" tabindex="0" onclick={(e) => { if (e.target === e.currentTarget) close(); }} onkeydown={(e) => { if (e.key === 'Escape') close(); }}>
    <div class="modal" style="width: 600px;">
      <div class="modal-header">
        <span class="modal-title">{t('common.import')} / {t('common.export')}</span>
        <button class="icon-btn" onclick={close} type="button" aria-label={t('common.close')}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
            <path d="M12 4L4 12M4 4l8 8"/>
          </svg>
        </button>
      </div>

      <div class="tab-bar">
        <button
          class="tab"
          class:active={activeTab === 'import'}
          onclick={() => activeTab = 'import'}
          type="button"
        >
          {t('common.import')}
        </button>
        <button
          class="tab"
          class:active={activeTab === 'export'}
          onclick={() => activeTab = 'export'}
          type="button"
        >
          {t('common.export')}
        </button>
      </div>

      <div class="modal-body">
        {#if activeTab === 'import'}
          <div class="import-tab">
            <div class="form-group">
              <label class="form-label" for="import-entity">{t('common.type')}</label>
              <select id="import-entity" class="select" bind:value={importEntity}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="import-file-button">{t('import.chooseFile')}</label>
              <button id="import-file-button" class="btn btn-secondary" onclick={handleFilePick} type="button">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
                </svg>
                {t('import.chooseFile')}
              </button>
              <input
                id="csv-file-input"
                type="file"
                accept=".csv"
                style="display: none;"
                onchange={handleFileInputChange}
              />
              {#if selectedImportPath}
                <p class="import-stats">{selectedImportPath}</p>
              {/if}
            </div>

            {#if parseResult}
              <p class="import-stats">
                {t('import.rowCount', { count: parseResult.count })}
                {#if parseResult.warnings.length > 0}
                  <span class="import-warnings"> · {parseResult.warnings.length} warnings</span>
                {/if}
              </p>

              {#if previewRows.length > 0}
                <details class="preview-details">
                  <summary class="preview-summary">{t('import.preview')}</summary>
                  <div class="preview-table-wrap">
                    <table class="data-table preview-table">
                      <thead>
                        <tr>
                          {#each parseResult.headers as h (h)}
                            <th>{h}</th>
                          {/each}
                        </tr>
                      </thead>
                      <tbody>
                        {#each previewRows as row, i (i)}
                          <tr>
                            {#each parseResult.headers as h (h)}
                              <td>{row[h]}</td>
                            {/each}
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                </details>
              {/if}
            {/if}
          </div>
        {:else}
          <div class="export-tab">
            <div class="form-group">
              <label class="form-label" for="export-entity">{t('export.entity')}</label>
              <select id="export-entity" class="select" bind:value={exportEntity}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="export-format">{t('export.format')}</label>
              <select id="export-format" class="select" bind:value={exportFormat}>
                <option value="csv">CSV</option>
                <option value="json">JSON (soon)</option>
              </select>
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={close} type="button">
          {t('common.cancel')}
        </button>
        {#if activeTab === 'import'}
          <button
            class="btn btn-primary"
            onclick={handleImport}
            disabled={!selectedImportPath || isImporting}
            type="button"
          >
            {isImporting ? t('import.importing') : t('import.importButton')}
          </button>
        {:else}
          <button
            class="btn btn-primary"
            onclick={handleExport}
            disabled={isExporting}
            type="button"
          >
            {isExporting ? t('export.exporting') : t('export.exportButton')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .import-tab,
  .export-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .import-stats {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .import-warnings {
    color: var(--text-warning);
  }

  .preview-details {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    overflow: hidden;
  }

  .preview-summary {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    cursor: pointer;
    background-color: var(--surface-hover);
    user-select: none;
    -webkit-user-select: none;
  }

  .preview-table-wrap {
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
  }

  .preview-table {
    min-width: 100%;
  }

  .preview-table th,
  .preview-table td {
    font-size: var(--text-xs);
    padding: var(--space-2) var(--space-3);
    white-space: nowrap;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
