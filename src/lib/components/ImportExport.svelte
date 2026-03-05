<script lang="ts">
  /**
   * ImportExport.svelte — Import/Export modal for 900CRM.
   *
   * Tabs:
   *   - Import: file picker, CSV preview, column mapping, import button
   *   - Export: entity selector, format selector, export button
   *
   * Uses Tauri dialog plugin for native file dialogs.
   * Client-side CSV parsing for import preview.
   */

  import { t } from '$lib/i18n';
  import { parseCSV, buildCSV, mapColumns } from '$lib/utils/csv';
  import type { ParseCSVResult, ColumnMapping } from '$lib/utils/csv';
  import { uiStore } from '$lib/stores/ui';
  import { invoke } from '@tauri-apps/api/core';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    open = $bindable(false),
    onclose,
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────

  let activeTab = $state<'import' | 'export'>('import');

  // Import state
  let csvText = $state('');
  let parseResult = $state<ParseCSVResult | null>(null);
  let columnMapping = $state<ColumnMapping>({});
  let isImporting = $state(false);
  let importEntity = $state('contacts');

  // Export state
  let exportEntity = $state('contacts');
  let exportFormat = $state('csv');
  let isExporting = $state(false);

  // ── CRM target fields per entity ──────────────────────────────────────────

  const TARGET_FIELDS: Record<string, string[]> = {
    contacts: ['firstName', 'lastName', 'email', 'phone', 'organization', 'type', 'notes'],
    deals:    ['name', 'value', 'stage', 'probability', 'expectedCloseDate', 'description'],
    activities: ['subject', 'type', 'dueDate', 'notes'],
  };

  // ── Derived ────────────────────────────────────────────────────────────────

  const previewRows = $derived(parseResult?.rows.slice(0, 5) ?? []);
  const targetFields = $derived(TARGET_FIELDS[importEntity] ?? []);

  // ── Handlers ───────────────────────────────────────────────────────────────

  async function handleFilePick() {
    try {
      // Try Tauri dialog; fall back to hidden file input
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = await openDialog({ filters: [{ name: 'CSV', extensions: ['csv'] }] });

      if (typeof selected === 'string') {
        const { readTextFile } = await import('@tauri-apps/plugin-fs');
        csvText = await readTextFile(selected);
        parseCsv();
      }
    } catch {
      // Fallback: trigger hidden file input
      document.getElementById('csv-file-input')?.click();
    }
  }

  function handleFileInputChange(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      csvText = ev.target?.result as string ?? '';
      parseCsv();
    };
    reader.readAsText(file);
  }

  function parseCsv() {
    if (!csvText.trim()) return;
    const result = parseCSV(csvText);
    parseResult = result;
    columnMapping = mapColumns(result.headers, targetFields);
  }

  async function handleImport() {
    if (!parseResult || parseResult.count === 0) return;
    isImporting = true;
    try {
      // Apply mapping and send to backend
      const rows = parseResult.rows.map((row) => {
        const mapped: Record<string, string> = {};
        for (const [src, tgt] of Object.entries(columnMapping)) {
          if (tgt && src in row) mapped[tgt] = row[src];
        }
        return mapped;
      });

      await invoke(`import_${importEntity}`, { rows });
      uiStore.toastSuccess(t('import.success'));
      open = false;
    } catch (err) {
      uiStore.toastError(t('import.failed'));
    } finally {
      isImporting = false;
    }
  }

  async function handleExport() {
    isExporting = true;
    try {
      const data = await invoke<unknown[]>(`export_${exportEntity}`);
      const cols = TARGET_FIELDS[exportEntity] ?? [];
      const csv = buildCSV(data as Record<string, unknown>[], cols);

      // Download file
      const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${exportEntity}-export.csv`;
      a.click();
      URL.revokeObjectURL(url);

      uiStore.toastSuccess(t('export.success'));
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
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) close(); }}>
    <div class="modal" style="width: 600px;">
      <!-- Header -->
      <div class="modal-header">
        <span class="modal-title">{t('common.import')} / {t('common.export')}</span>
        <button class="icon-btn" onclick={close} type="button" aria-label={t('common.close')}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
            <path d="M12 4L4 12M4 4l8 8"/>
          </svg>
        </button>
      </div>

      <!-- Tabs -->
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

      <!-- Body -->
      <div class="modal-body">
        {#if activeTab === 'import'}
          <!-- Import tab -->
          <div class="import-tab">
            <!-- Entity selector -->
            <div class="form-group">
              <label class="form-label" for="import-entity">{t('common.type')}</label>
              <select id="import-entity" class="select" bind:value={importEntity} onchange={() => { if (parseResult) columnMapping = mapColumns(parseResult.headers, targetFields); }}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="activities">{t('activities.title')}</option>
              </select>
            </div>

            <!-- File picker -->
            <div class="form-group">
              <label class="form-label">{t('import.chooseFile')}</label>
              <button class="btn btn-secondary" onclick={handleFilePick} type="button">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
                </svg>
                {t('import.chooseFile')}
              </button>
              <!-- Hidden fallback file input -->
              <input
                id="csv-file-input"
                type="file"
                accept=".csv"
                style="display: none;"
                onchange={handleFileInputChange}
              />
            </div>

            {#if parseResult}
              <!-- Stats -->
              <p class="import-stats">
                {t('import.rowCount', { count: parseResult.count })}
                {#if parseResult.warnings.length > 0}
                  <span class="import-warnings"> · {parseResult.warnings.length} warnings</span>
                {/if}
              </p>

              <!-- Column mapping -->
              {#if parseResult.headers.length > 0}
                <div class="form-group">
                  <label class="form-label">{t('import.columnMapping')}</label>
                  <div class="mapping-grid">
                    {#each parseResult.headers as header (header)}
                      <div class="mapping-row">
                        <span class="mapping-source">{header}</span>
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                          <path d="M2 6h8M7 3l3 3-3 3"/>
                        </svg>
                        <select
                          class="select mapping-target"
                          bind:value={columnMapping[header]}
                        >
                          <option value={null}>{t('import.skip')}</option>
                          {#each targetFields as field (field)}
                            <option value={field}>{field}</option>
                          {/each}
                        </select>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Preview -->
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
          <!-- Export tab -->
          <div class="export-tab">
            <div class="form-group">
              <label class="form-label" for="export-entity">{t('export.entity')}</label>
              <select id="export-entity" class="select" bind:value={exportEntity}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="activities">{t('activities.title')}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="export-format">{t('export.format')}</label>
              <select id="export-format" class="select" bind:value={exportFormat}>
                <option value="csv">CSV</option>
                <option value="json">JSON</option>
              </select>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={close} type="button">
          {t('common.cancel')}
        </button>
        {#if activeTab === 'import'}
          <button
            class="btn btn-primary"
            onclick={handleImport}
            disabled={!parseResult || parseResult.count === 0 || isImporting}
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

  .mapping-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .mapping-row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .mapping-source {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    min-width: 140px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mapping-target {
    flex: 1;
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
