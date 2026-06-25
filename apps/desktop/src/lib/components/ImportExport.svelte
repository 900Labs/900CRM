<script lang="ts">
  /**
   * ImportExport.svelte — Import/Export modal for CRM CSV data.
   */

  import { t } from '$lib/i18n';
  import { parseCSV, applyMapping } from '$lib/utils/csv';
  import type { ParseCSVResult, ColumnMapping } from '$lib/utils/csv';
  import {
    getImportFieldOptions,
    suggestImportMapping,
    toBackendMapping,
    validateImportMapping,
    type MappedImportEntity,
  } from '$lib/utils/importWizard';
  import { uiStore } from '$lib/stores/ui';
  import {
    exportData,
    importCsv,
    importData,
    importContactsCsvWithMapping,
    importDealsCsvWithMapping,
    importOrganizationsCsvWithMapping,
    preflightJson,
    preflightContactsCsvImportWithMapping,
    preflightDealsCsvImportWithMapping,
    preflightOrganizationsCsvImportWithMapping,
    type ExportFormat,
    type ContactImportTargetField,
    type DealImportTargetField,
    type ImportFormat,
    type ImportExportEntity,
    type ImportPreflightReport,
    type ImportResult,
    type ImportWithBackupResult,
    type OrganizationImportTargetField,
  } from '$lib/api/importExport';

  let {
    open = $bindable(false),
    onclose,
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  type ImportStep = 'select' | 'preview' | 'mapping' | 'duplicates' | 'confirm' | 'summary';
  type FileSource = 'desktop' | 'browser';

  let activeTab = $state<'import' | 'export'>('import');

  let csvText = $state('');
  let parseResult = $state<ParseCSVResult | null>(null);
  let selectedImportPath = $state<string | null>(null);
  let selectedImportLabel = $state<string | null>(null);
  let fileSource = $state<FileSource | null>(null);
  let isImporting = $state(false);
  let isPreflighting = $state(false);
  let importEntity = $state<ImportExportEntity>('contacts');
  let importFormat = $state<ImportFormat>('csv');
  let importStep = $state<ImportStep>('select');
  let columnMapping = $state<ColumnMapping>({});
  let validationErrors = $state<string[]>([]);
  let preflightReport = $state<ImportPreflightReport | null>(null);
  let importSummary = $state<ImportResult | null>(null);
  let importBackupPath = $state<string | null>(null);

  let exportEntity = $state<ImportExportEntity>('contacts');
  let exportFormat = $state<ExportFormat>('csv');
  let isExporting = $state(false);

  const importDialogMetadata: Record<ImportFormat, { extension: string; filterName: string }> = {
    csv: { extension: 'csv', filterName: 'CSV' },
    json: { extension: 'json', filterName: 'JSON' },
  };

  const exportDialogMetadata: Record<ExportFormat, { defaultExtension: string; filterName: string }> = {
    csv: { defaultExtension: 'csv', filterName: 'CSV' },
    json: { defaultExtension: 'json', filterName: 'JSON' },
  };

  const previewRows = $derived(parseResult?.rows.slice(0, 5) ?? []);
  const mappedPreviewRows = $derived(applyMapping(previewRows, columnMapping));
  const isMappedImport = $derived(
    importEntity === 'contacts' || importEntity === 'deals' || importEntity === 'organizations',
  );
  const isCsvImport = $derived(importFormat === 'csv');
  const isJsonImport = $derived(importFormat === 'json');
  const showCsvWizard = $derived(isCsvImport && isMappedImport);
  const showDuplicateReview = $derived(showCsvWizard || isJsonImport);
  const mappedEntity = $derived(isMappedImport ? (importEntity as MappedImportEntity) : null);
  const importFieldOptions = $derived(mappedEntity ? getImportFieldOptions(mappedEntity) : []);
  const canUseMappedCommands = $derived(Boolean(isCsvImport && fileSource === 'desktop' && selectedImportPath));
  const fallbackImportBlocked = $derived(showCsvWizard && fileSource === 'browser');
  const duplicateWarnings = $derived(preflightReport?.warnings ?? []);

  async function handleFilePick() {
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const dialogMetadata = importDialogMetadata[importFormat];
      const selected = await openDialog({
        filters: [{ name: dialogMetadata.filterName, extensions: [dialogMetadata.extension] }],
        multiple: false,
      });

      if (typeof selected !== 'string') {
        return;
      }

      if (isJsonImport) {
        loadSelectedJson(selected, selected);
        await handleJsonPreflight();
        return;
      }

      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const text = await readTextFile(selected);
      loadSelectedCsv(text, selected, selected, 'desktop');
    } catch {
      if (isCsvImport) {
        document.getElementById('csv-file-input')?.click();
      } else {
        uiStore.toastError(t('import.failed'));
      }
    }
  }

  function handleFileInputChange(e: Event) {
    if (!isCsvImport) {
      return;
    }

    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = (ev) => {
      const text = (ev.target?.result as string) ?? '';
      loadSelectedCsv(text, file.name, file.name, 'browser');
    };
    reader.readAsText(file);
  }

  function handleImportEntityChange(e: Event) {
    importEntity = (e.target as HTMLSelectElement).value as ImportExportEntity;
    resetImportState({ keepEntity: true, keepFormat: true });
  }

  function handleImportFormatChange(e: Event) {
    importFormat = (e.target as HTMLSelectElement).value as ImportFormat;
    resetImportState({ keepEntity: true, keepFormat: true });
  }

  function loadSelectedCsv(text: string, label: string, path: string | null, source: FileSource) {
    csvText = text;
    selectedImportLabel = label;
    selectedImportPath = path;
    fileSource = source;
    parseResult = parseCSV(csvText);
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    validationErrors = [];
    importStep = parseResult.headers.length > 0 ? 'preview' : 'select';

    if (isMappedImport && mappedEntity) {
      columnMapping = suggestImportMapping(mappedEntity, parseResult.headers);
    } else {
      columnMapping = {};
    }
  }

  function loadSelectedJson(label: string, path: string) {
    csvText = '';
    parseResult = null;
    selectedImportLabel = label;
    selectedImportPath = path;
    fileSource = 'desktop';
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    validationErrors = [];
    columnMapping = {};
    importStep = 'select';
  }

  function resetImportState(options: { keepEntity?: boolean; keepFormat?: boolean } = {}) {
    csvText = '';
    parseResult = null;
    selectedImportPath = null;
    selectedImportLabel = null;
    fileSource = null;
    isImporting = false;
    isPreflighting = false;
    importStep = 'select';
    columnMapping = {};
    validationErrors = [];
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;

    if (!options.keepEntity) {
      importEntity = 'contacts';
    }

    if (!options.keepFormat) {
      importFormat = 'csv';
    }
  }

  function updateMapping(sourceHeader: string, target: string | null) {
    columnMapping = { ...columnMapping, [sourceHeader]: target };
    validationErrors = [];
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
  }

  function validateCurrentMapping(): boolean {
    if (!mappedEntity) {
      return true;
    }

    const result = validateImportMapping(mappedEntity, columnMapping);
    validationErrors = result.errors;
    return result.valid;
  }

  function goToPreview() {
    if (!parseResult) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }
    importStep = 'preview';
  }

  function goToMapping() {
    if (!parseResult) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }
    importStep = 'mapping';
  }

  async function handleLegacyImport() {
    if (!selectedImportPath) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }

    isImporting = true;
    try {
      const result = await importCsv(importEntity, selectedImportPath);
      applyImportResult(result);

      if (result.import.skipped > 0) {
        uiStore.toastWarning(`${t('import.success')} (${result.import.created} created, ${result.import.skipped} skipped)`);
      } else {
        uiStore.toastSuccess(`${t('import.success')} (${result.import.created})`);
      }

      close();
    } catch {
      uiStore.toastError(t('import.failed'));
    } finally {
      isImporting = false;
    }
  }

  async function handleJsonImport() {
    if (!selectedImportPath) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }

    isImporting = true;
    validationErrors = [];
    try {
      const result = await importData(importEntity, 'json', selectedImportPath);
      applyImportResult(result);
      importStep = 'summary';
      const summary = result.import;

      if (summary.skipped > 0) {
        uiStore.toastWarning(`${t('import.success')} (${summary.created} created, ${summary.skipped} skipped)`);
      } else {
        uiStore.toastSuccess(`${t('import.success')} (${summary.created})`);
      }
    } catch {
      validationErrors = [t('import.failed')];
      uiStore.toastError(t('import.failed'));
    } finally {
      isImporting = false;
    }
  }

  async function handleJsonPreflight() {
    if (!selectedImportPath) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }

    isPreflighting = true;
    validationErrors = [];
    preflightReport = null;
    try {
      preflightReport = await preflightJson(importEntity, selectedImportPath);
      importStep = preflightReport.duplicate_warning_count > 0 ? 'duplicates' : 'confirm';
    } catch {
      validationErrors = [t('import.preflightFailed')];
      uiStore.toastError(t('import.preflightFailed'));
    } finally {
      isPreflighting = false;
    }
  }

  async function handlePreflight() {
    if (!mappedEntity || !validateCurrentMapping()) {
      return;
    }

    if (!canUseMappedCommands || !selectedImportPath) {
      validationErrors = [t('import.desktopPickerRequired')];
      return;
    }

    isPreflighting = true;
    validationErrors = [];
    try {
      preflightReport = await runMappedPreflight(mappedEntity, selectedImportPath);
      importStep = 'duplicates';
    } catch {
      validationErrors = [t('import.preflightFailed')];
    } finally {
      isPreflighting = false;
    }
  }

  async function handleMappedImport() {
    if (!mappedEntity || !validateCurrentMapping()) {
      return;
    }

    if (!canUseMappedCommands || !selectedImportPath) {
      validationErrors = [t('import.desktopPickerRequired')];
      return;
    }

    isImporting = true;
    validationErrors = [];
    try {
      applyImportResult(await runMappedImport(mappedEntity, selectedImportPath));
      importStep = 'summary';
      uiStore.toastSuccess(t('import.success'));
    } catch {
      validationErrors = [t('import.failed')];
    } finally {
      isImporting = false;
    }
  }

  async function runMappedPreflight(
    entity: MappedImportEntity,
    filePath: string,
  ): Promise<ImportPreflightReport> {
    if (entity === 'contacts') {
      return preflightContactsCsvImportWithMapping(
        filePath,
        toBackendMapping<ContactImportTargetField>(columnMapping),
      );
    }

    if (entity === 'deals') {
      return preflightDealsCsvImportWithMapping(
        filePath,
        toBackendMapping<DealImportTargetField>(columnMapping),
      );
    }

    return preflightOrganizationsCsvImportWithMapping(
      filePath,
      toBackendMapping<OrganizationImportTargetField>(columnMapping),
    );
  }

  async function runMappedImport(
    entity: MappedImportEntity,
    filePath: string,
  ): Promise<ImportWithBackupResult> {
    if (entity === 'contacts') {
      return importContactsCsvWithMapping(
        filePath,
        toBackendMapping<ContactImportTargetField>(columnMapping),
      );
    }

    if (entity === 'deals') {
      return importDealsCsvWithMapping(
        filePath,
        toBackendMapping<DealImportTargetField>(columnMapping),
      );
    }

    return importOrganizationsCsvWithMapping(
      filePath,
      toBackendMapping<OrganizationImportTargetField>(columnMapping),
    );
  }

  function isTargetAssigned(target: string, currentHeader: string): boolean {
    return Object.entries(columnMapping).some(
      ([source, assignedTarget]) => source !== currentHeader && assignedTarget === target,
    );
  }

  function backFromCurrentStep() {
    if (isJsonImport) {
      if (importStep === 'summary') {
        importStep = 'confirm';
      } else if (importStep === 'confirm') {
        importStep = (preflightReport?.duplicate_warning_count ?? 0) > 0 ? 'duplicates' : 'select';
      } else {
        importStep = 'select';
      }
    } else if (importStep === 'summary') {
      importStep = 'confirm';
    } else if (importStep === 'confirm') {
      importStep = 'duplicates';
    } else if (importStep === 'duplicates') {
      importStep = 'mapping';
    } else if (importStep === 'mapping') {
      importStep = 'preview';
    } else {
      importStep = 'select';
    }
  }

  function doneImport() {
    close();
    resetImportState({ keepEntity: true });
  }

  function applyImportResult(result: ImportWithBackupResult) {
    importSummary = result.import;
    importBackupPath = result.backup.backup_dir;
  }

  async function handleExport() {
    isExporting = true;
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const dialogMetadata = exportDialogMetadata[exportFormat];
      const savePath = await save({
        defaultPath: `${exportEntity}-export.${dialogMetadata.defaultExtension}`,
        filters: [
          {
            name: dialogMetadata.filterName,
            extensions: [dialogMetadata.defaultExtension],
          },
        ],
      });

      if (typeof savePath !== 'string') {
        return;
      }

      const rows = await exportData(exportEntity, exportFormat, savePath);
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
    <div class="modal import-export-modal">
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
              <select id="import-entity" class="select" value={importEntity} onchange={handleImportEntityChange}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="organizations">{t('organizations.title')}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="import-format">{t('export.format')}</label>
              <select id="import-format" class="select" value={importFormat} onchange={handleImportFormatChange}>
                <option value="csv">CSV</option>
                <option value="json">JSON</option>
              </select>
            </div>

            {#if showCsvWizard}
              <ol class="wizard-steps" aria-label={t('import.wizardProgress')}>
                <li class:active={importStep === 'select'} class:complete={importStep !== 'select'}>{t('import.stepSelect')}</li>
                <li class:active={importStep === 'preview'} class:complete={['mapping', 'duplicates', 'confirm', 'summary'].includes(importStep)}>{t('import.stepPreview')}</li>
                <li class:active={importStep === 'mapping'} class:complete={['duplicates', 'confirm', 'summary'].includes(importStep)}>{t('import.stepMap')}</li>
                <li class:active={importStep === 'duplicates'} class:complete={['confirm', 'summary'].includes(importStep)}>{t('import.stepDuplicates')}</li>
                <li class:active={importStep === 'confirm'} class:complete={importStep === 'summary'}>{t('import.stepConfirm')}</li>
                <li class:active={importStep === 'summary'}>{t('import.stepSummary')}</li>
              </ol>
            {/if}

            {#if !showCsvWizard || importStep === 'select'}
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
                {#if selectedImportLabel}
                  <p class="import-stats">{selectedImportLabel}</p>
                {/if}
                {#if fallbackImportBlocked}
                  <p class="validation-message">{t('import.desktopPickerRequired')}</p>
                {/if}
                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            {#if parseResult && (!showCsvWizard || importStep === 'preview')}
              <p class="import-stats">
                {t('import.rowCount', { count: parseResult.count })}
                {#if parseResult.warnings.length > 0}
                  <span class="import-warnings"> · {parseResult.warnings.length} {t('import.parseWarnings')}</span>
                {/if}
              </p>

              {#if previewRows.length > 0}
                <div class="preview-panel">
                  <div class="preview-title">{t('import.previewRows')}</div>
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
                </div>
              {/if}
            {/if}

            {#if showCsvWizard && parseResult && importStep === 'mapping'}
              <div class="mapping-panel">
                <div class="mapping-header">
                  <span>{t('import.columnMapping')}</span>
                  <span>{t('import.targetField')}</span>
                </div>
                {#each parseResult.headers as h (h)}
                  <div class="mapping-row">
                    <div class="source-column">
                      <span class="source-label">{h}</span>
                      {#if previewRows[0]?.[h]}
                        <span class="source-sample">{previewRows[0][h]}</span>
                      {/if}
                    </div>
                    <select
                      class="select"
                      aria-label={`${t('import.mapColumn')}: ${h}`}
                      value={columnMapping[h] ?? ''}
                      onchange={(e) => updateMapping(h, (e.target as HTMLSelectElement).value || null)}
                    >
                      <option value="">{t('import.skip')}</option>
                      {#each importFieldOptions as field (field.value)}
                        <option
                          value={field.value}
                          disabled={isTargetAssigned(field.value, h)}
                        >
                          {field.label}{field.required ? ` (${t('common.required')})` : ''}
                        </option>
                      {/each}
                    </select>
                  </div>
                {/each}

                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}

                {#if mappedPreviewRows.length > 0}
                  <div class="mapped-preview">
                    <div class="preview-title">{t('import.mappedPreview')}</div>
                    <div class="mapped-preview-list">
                      {#each mappedPreviewRows.slice(0, 3) as row, i (i)}
                        <div class="mapped-preview-row">
                          <span>{t('import.rowNumber', { number: i + 1 })}</span>
                          <code>{JSON.stringify(row)}</code>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}

            {#if showDuplicateReview && importStep === 'duplicates'}
              <div class="duplicate-panel">
                <p class="import-stats">
                  {t('import.duplicateWarningCount', { count: preflightReport?.duplicate_warning_count ?? 0 })}
                </p>
                {#if duplicateWarnings.length > 0}
                  <div class="preview-table-wrap duplicate-table-wrap">
                    <table class="data-table preview-table">
                      <thead>
                        <tr>
                          <th>{t('import.row')}</th>
                          <th>{t('import.matchType')}</th>
                          <th>{t('import.csvValue')}</th>
                          <th>{t('import.existingRecord')}</th>
                          <th>{t('import.reason')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each duplicateWarnings as warning (warning.row_number + warning.match_type + warning.csv_value)}
                          <tr>
                            <td>{warning.row_number}</td>
                            <td>{warning.match_type}</td>
                            <td>{warning.csv_value}</td>
                            <td>{warning.existing_display_label}</td>
                            <td>{warning.reason}</td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                {:else}
                  <p class="empty-message">{t('import.noDuplicateWarnings')}</p>
                {/if}
              </div>
            {/if}

            {#if showDuplicateReview && importStep === 'confirm'}
              <div class="confirm-panel">
                <p class="import-stats">
                  {t('import.confirmRows', { count: preflightReport?.total_rows ?? parseResult?.count ?? 0 })}
                </p>
                {#if (preflightReport?.duplicate_warning_count ?? 0) > 0}
                  <p class="import-warnings">
                    {t('import.confirmDuplicateWarnings', { count: preflightReport?.duplicate_warning_count ?? 0 })}
                  </p>
                {/if}
                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            {#if importStep === 'summary' && importSummary}
              <div class="summary-panel">
                <div class="summary-grid">
                  <div>
                    <span>{t('import.created')}</span>
                    <strong>{importSummary.created}</strong>
                  </div>
                  <div>
                    <span>{t('import.skipped')}</span>
                    <strong>{importSummary.skipped}</strong>
                  </div>
                  <div>
                    <span>{t('import.errors')}</span>
                    <strong>{importSummary.errors.length}</strong>
                  </div>
                </div>
                {#if importSummary.errors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each importSummary.errors.slice(0, 8) as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
                {#if importBackupPath}
                  <p class="backup-summary">
                    {t('import.preImportBackupCreated', { path: importBackupPath })}
                  </p>
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <div class="export-tab">
            <div class="form-group">
              <label class="form-label" for="export-entity">{t('export.entity')}</label>
              <select id="export-entity" class="select" bind:value={exportEntity}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="organizations">{t('organizations.title')}</option>
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

      <div class="modal-footer">
        {#if activeTab === 'import' && showDuplicateReview && importStep !== 'select' && importStep !== 'summary'}
          <button class="btn btn-secondary" onclick={backFromCurrentStep} type="button" disabled={isImporting || isPreflighting}>
            {t('common.back')}
          </button>
        {:else if activeTab === 'import' && importStep === 'summary'}
          <button class="btn btn-secondary" onclick={doneImport} type="button">
            {t('common.close')}
          </button>
        {:else}
          <button class="btn btn-secondary" onclick={close} type="button">
            {t('common.cancel')}
          </button>
        {/if}

        {#if activeTab === 'import'}
          {#if isJsonImport}
            {#if importStep === 'select'}
              <button
                class="btn btn-primary"
                onclick={handleJsonPreflight}
                disabled={!selectedImportPath || isPreflighting}
                type="button"
              >
                {isPreflighting ? t('import.checking') : t('import.detectDuplicates')}
              </button>
            {:else if importStep === 'duplicates'}
              <button class="btn btn-primary" onclick={() => importStep = 'confirm'} type="button">
                {t('import.continueDespiteWarnings')}
              </button>
            {:else if importStep === 'confirm'}
              <button
                class="btn btn-primary"
                onclick={handleJsonImport}
                disabled={isImporting}
                type="button"
              >
                {isImporting ? t('import.importing') : t('import.confirmImport')}
              </button>
            {/if}
          {:else if !isMappedImport}
            <button
              class="btn btn-primary"
              onclick={handleLegacyImport}
              disabled={!selectedImportPath || isImporting}
              type="button"
            >
              {isImporting ? t('import.importing') : t('import.importButton')}
            </button>
          {:else if importStep === 'select'}
            <button
              class="btn btn-primary"
              onclick={goToPreview}
              disabled={!parseResult}
              type="button"
            >
              {t('common.next')}
            </button>
          {:else if importStep === 'preview'}
            <button class="btn btn-primary" onclick={goToMapping} type="button">
              {t('common.next')}
            </button>
          {:else if importStep === 'mapping'}
            <button
              class="btn btn-primary"
              onclick={handlePreflight}
              disabled={isPreflighting}
              type="button"
            >
              {isPreflighting ? t('import.checking') : t('import.detectDuplicates')}
            </button>
          {:else if importStep === 'duplicates'}
            <button class="btn btn-primary" onclick={() => importStep = 'confirm'} type="button">
              {t('import.continueDespiteWarnings')}
            </button>
          {:else if importStep === 'confirm'}
            <button
              class="btn btn-primary"
              onclick={handleMappedImport}
              disabled={isImporting}
              type="button"
            >
              {isImporting ? t('import.importing') : t('import.confirmImport')}
            </button>
          {/if}
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
  .import-export-modal {
    width: min(760px, calc(100vw - 32px));
  }

  .import-tab,
  .export-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .wizard-steps {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: var(--space-2);
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .wizard-steps li {
    border-bottom: 2px solid var(--border-default);
    color: var(--text-muted);
    font-size: var(--text-xs);
    line-height: 1.3;
    min-height: 34px;
    padding-bottom: var(--space-2);
  }

  .wizard-steps li.active {
    border-color: var(--border-focus);
    color: var(--text-primary);
    font-weight: var(--weight-medium);
  }

  .wizard-steps li.complete {
    border-color: var(--text-success);
    color: var(--text-secondary);
  }

  .import-stats,
  .empty-message {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .import-warnings {
    color: var(--text-warning);
    font-size: var(--text-sm);
  }

  .backup-summary {
    background-color: var(--surface-hover);
    border-left: 3px solid var(--color-primary);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    margin: 0;
    overflow-wrap: anywhere;
    padding: var(--space-3);
  }

  .validation-message {
    color: var(--text-danger);
    font-size: var(--text-sm);
  }

  .preview-panel,
  .mapping-panel,
  .duplicate-panel,
  .confirm-panel,
  .summary-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .preview-title {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .preview-table-wrap {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    max-height: 220px;
    overflow: auto;
  }

  .duplicate-table-wrap {
    max-height: 260px;
  }

  .preview-table {
    min-width: 100%;
  }

  .preview-table th,
  .preview-table td {
    font-size: var(--text-xs);
    padding: var(--space-2) var(--space-3);
    white-space: nowrap;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mapping-header,
  .mapping-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(180px, 240px);
    gap: var(--space-4);
    align-items: center;
  }

  .mapping-header {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
  }

  .mapping-row {
    border-top: var(--border-width) solid var(--border-default);
    padding-top: var(--space-3);
  }

  .source-column {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .source-label {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-sample {
    color: var(--text-muted);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .validation-list {
    background-color: var(--surface-hover);
    border-left: 3px solid var(--text-danger);
    color: var(--text-danger);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
  }

  .validation-list p {
    font-size: var(--text-sm);
    margin: 0;
  }

  .mapped-preview {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .mapped-preview-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .mapped-preview-row {
    align-items: center;
    display: grid;
    gap: var(--space-3);
    grid-template-columns: 64px minmax(0, 1fr);
  }

  .mapped-preview-row span,
  .mapped-preview-row code {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .mapped-preview-row code {
    background-color: var(--surface-hover);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
    padding: var(--space-2);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .summary-grid {
    display: grid;
    gap: var(--space-3);
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .summary-grid div {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
  }

  .summary-grid span {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .summary-grid strong {
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  @media (max-width: 720px) {
    .wizard-steps {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .mapping-header,
    .mapping-row {
      grid-template-columns: 1fr;
      gap: var(--space-2);
    }

    .summary-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
