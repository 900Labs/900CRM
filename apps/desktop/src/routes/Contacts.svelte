<script lang="ts">
  /**
   * Contacts.svelte — Contacts list view for 900CRM.
   *
   * Features: DataTable with search, type filter, sort, pagination.
   * Toolbar with Add Contact, Import/Export buttons.
   * Click row to navigate to ContactDetail.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { contactStore } from '$lib/stores/contacts';
  import { uiStore } from '$lib/stores/ui';
  import type { Contact, ContactDuplicateCandidate } from '$lib/api/contacts';
  import type { Column } from '$lib/components/DataTable.svelte';
  import { formatFullName, formatDate } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';
  import {
    listCustomFieldDefinitions,
    type CustomFieldDefinition,
  } from '$lib/api/customFields';
  import DataTable from '$lib/components/DataTable.svelte';
  import ImportExport from '$lib/components/ImportExport.svelte';

  // ── State ───────────────────────────────────────────────────────────────────

  let searchQuery = $state('');
  let typeFilter = $state<'' | 'person' | 'org'>('');
  let showImportExport = $state(false);
  let selectedContact = $state<Contact | null>(null);
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let customFieldFilterTimer: ReturnType<typeof setTimeout> | undefined;
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let selectedCustomFieldDefId = $state('');
  let customFieldQuery = $state('');
  let customFieldsLoading = $state(true);
  let selectedDuplicateKey = $state('');
  let mergeDirection = $state<'suggested' | 'swapped'>('suggested');

  const selectedDuplicateCandidate = $derived(
    contactStore.duplicateCandidates.find((candidate) => duplicateCandidateKey(candidate) === selectedDuplicateKey)
      ?? contactStore.duplicateCandidates[0]
      ?? null,
  );
  const duplicateMergePreview = $derived(
    selectedDuplicateCandidate
      ? mergeDirection === 'swapped'
        ? {
            sourceId: selectedDuplicateCandidate.targetId,
            sourceLabel: selectedDuplicateCandidate.targetDisplayLabel,
            targetId: selectedDuplicateCandidate.sourceId,
            targetLabel: selectedDuplicateCandidate.sourceDisplayLabel,
          }
        : {
            sourceId: selectedDuplicateCandidate.sourceId,
            sourceLabel: selectedDuplicateCandidate.sourceDisplayLabel,
            targetId: selectedDuplicateCandidate.targetId,
            targetLabel: selectedDuplicateCandidate.targetDisplayLabel,
          }
      : null,
  );
  const duplicateControlsDisabled = $derived(
    contactStore.duplicateCandidatesLoading || contactStore.isMergingDuplicates,
  );

  $effect(() => {
    const candidates = contactStore.duplicateCandidates;
    if (candidates.length === 0) {
      selectedDuplicateKey = '';
      mergeDirection = 'suggested';
      return;
    }
    if (!candidates.some((candidate) => duplicateCandidateKey(candidate) === selectedDuplicateKey)) {
      selectedDuplicateKey = duplicateCandidateKey(candidates[0]);
      mergeDirection = 'suggested';
    }
  });

  // ── Column definitions ─────────────────────────────────────────────────────

  const columns: Column<Contact>[] = [
    {
      key: 'name',
      label: t('contacts.name'),
      sortable: true,
      render: (c) => formatFullName((c as Contact).firstName, (c as Contact).lastName),
    },
    {
      key: 'organization',
      label: t('contacts.organization'),
      sortable: true,
      render: (c) => (c as Contact).organization ?? '—',
    },
    {
      key: 'email',
      label: t('contacts.email'),
      render: (c) => (c as Contact).email ?? '—',
    },
    {
      key: 'phone',
      label: t('contacts.phone'),
      render: (c) => (c as Contact).phone ?? '—',
    },
    {
      key: 'type',
      label: t('contacts.type'),
      sortable: true,
      render: (c) => t(`contacts.${(c as Contact).type}`),
    },
    {
      key: 'createdAt',
      label: t('contacts.created'),
      sortable: true,
      render: (c) => formatDate((c as Contact).createdAt, settingsStore.dateFormat as 'MMM D, YYYY'),
    },
  ];

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(async () => {
    await Promise.all([
      contactStore.loadContacts(),
      contactStore.loadDuplicateCandidates().catch(() => undefined),
      loadCustomFieldDefinitions(),
    ]);
  });

  // ── Handlers ───────────────────────────────────────────────────────────────

  function handleSearchInput(e: Event) {
    searchQuery = (e.target as HTMLInputElement).value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      contactStore.setFilters({
        search: searchQuery,
        type: typeFilter || undefined,
        customFieldDefId: selectedCustomFieldDefId || undefined,
        customFieldQuery: customFieldQuery.trim() || undefined,
        page: 1,
      });
    }, 300);
  }

  async function handleTypeFilter(type: '' | 'person' | 'org') {
    typeFilter = type;
    await contactStore.setFilters({
      type: type || undefined,
      customFieldDefId: selectedCustomFieldDefId || undefined,
      customFieldQuery: customFieldQuery.trim() || undefined,
      page: 1,
    });
  }

  function handleRowClick(row: unknown) {
    const contact = row as Contact;
    contactStore.selectContact(contact);
    selectedContact = contact;
    // Navigate to detail view (hash routing)
    window.location.hash = `/contacts/${contact.id}`;
  }

  async function handleSort(key: string, dir: 'asc' | 'desc') {
    await contactStore.setFilters({
      sortBy: key as 'name' | 'createdAt' | 'updatedAt',
      sortDir: dir,
      customFieldDefId: selectedCustomFieldDefId || undefined,
      customFieldQuery: customFieldQuery.trim() || undefined,
    });
  }

  async function loadCustomFieldDefinitions() {
    customFieldsLoading = true;
    try {
      customFieldDefinitions = await listCustomFieldDefinitions('contact');
    } finally {
      customFieldsLoading = false;
    }
  }

  async function applyCustomFieldFilter() {
    await contactStore.setFilters({
      search: searchQuery,
      type: typeFilter || undefined,
      customFieldDefId: selectedCustomFieldDefId || undefined,
      customFieldQuery: customFieldQuery.trim() || undefined,
      page: 1,
    });
  }

  function handleCustomFieldDefinitionChange(event: Event) {
    selectedCustomFieldDefId = (event.target as HTMLSelectElement).value;
    void applyCustomFieldFilter();
  }

  function handleCustomFieldQueryInput(event: Event) {
    customFieldQuery = (event.target as HTMLInputElement).value;
    clearTimeout(customFieldFilterTimer);
    customFieldFilterTimer = setTimeout(() => {
      void applyCustomFieldFilter();
    }, 250);
  }

  async function clearCustomFieldFilter() {
    selectedCustomFieldDefId = '';
    customFieldQuery = '';
    await applyCustomFieldFilter();
  }

  function duplicateCandidateKey(candidate: ContactDuplicateCandidate): string {
    return [
      candidate.matchType,
      candidate.matchedValue,
      candidate.sourceId,
      candidate.targetId,
    ].join(':');
  }

  function duplicateCandidateLabel(candidate: ContactDuplicateCandidate): string {
    const matchLabel = candidate.matchType === 'email' ? t('contacts.email') : t('contacts.phone');
    return t('contacts.duplicatesCandidateOption', {
      source: candidate.sourceDisplayLabel,
      target: candidate.targetDisplayLabel,
      type: matchLabel,
      value: candidate.matchedValue,
    });
  }

  function handleDuplicateCandidateChange(event: Event) {
    selectedDuplicateKey = (event.target as HTMLSelectElement).value;
    mergeDirection = 'suggested';
  }

  async function refreshDuplicateCandidates() {
    await contactStore.loadDuplicateCandidates();
  }

  async function handleMergeDuplicateCandidate() {
    if (!duplicateMergePreview) {
      return;
    }

    const confirmed = window.confirm(t('contacts.duplicatesConfirmMerge', {
      source: duplicateMergePreview.sourceLabel,
      target: duplicateMergePreview.targetLabel,
    }));
    if (!confirmed) {
      return;
    }

    await contactStore.mergeDuplicateContacts(
      duplicateMergePreview.sourceId,
      duplicateMergePreview.targetId,
    );
    mergeDirection = 'suggested';
  }
</script>

<div class="page-content contacts-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('contacts.title')}</h1>
    <div class="toolbar">
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addContact')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('contacts.addContact')}
      </button>
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => showImportExport = true}
        type="button"
      >
        {t('common.import')} / {t('common.export')}
      </button>
      <button
        class="btn btn-secondary btn-sm"
        onclick={refreshDuplicateCandidates}
        type="button"
        disabled={duplicateControlsDisabled}
      >
        {contactStore.duplicateCandidatesLoading ? t('contacts.duplicatesChecking') : t('contacts.duplicatesCheck')}
      </button>
    </div>
  </div>

  <!-- Filters -->
  <div class="contacts-filters">
    <div class="search-wrap">
      <svg class="filter-search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        class="input filter-search selectable"
        type="search"
        placeholder={t('contacts.search')}
        value={searchQuery}
        oninput={handleSearchInput}
        aria-label={t('contacts.search')}
      />
    </div>

    <div class="type-filters" role="group" aria-label={t('contacts.type')}>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'person', label: t('contacts.person') },
        { value: 'org', label: t('contacts.org') },
      ] as option (option.value)}
        <button
          class="filter-chip"
          class:active={typeFilter === option.value}
          onclick={() => handleTypeFilter(option.value as '' | 'person' | 'org')}
          type="button"
        >
          {option.label}
        </button>
      {/each}
    </div>

    <div class="custom-field-filter" role="group" aria-label={t('common.customFieldFilter')}>
      <select
        class="input custom-field-select"
        value={selectedCustomFieldDefId}
        onchange={handleCustomFieldDefinitionChange}
        aria-label={t('common.customField')}
      >
        <option value="">
          {customFieldsLoading ? t('common.loading') : t('common.selectCustomField')}
        </option>
        {#each customFieldDefinitions as definition (definition.id)}
          <option value={definition.id}>{definition.field_name}</option>
        {/each}
      </select>
      <input
        class="input custom-field-input selectable"
        type="search"
        value={customFieldQuery}
        oninput={handleCustomFieldQueryInput}
        placeholder={t('common.filterValue')}
        aria-label={t('common.filterValue')}
        disabled={!selectedCustomFieldDefId}
      />
      <button
        class="btn btn-ghost btn-sm"
        type="button"
        onclick={clearCustomFieldFilter}
        disabled={!selectedCustomFieldDefId && !customFieldQuery}
      >
        {t('common.clear')}
      </button>
    </div>
  </div>

  {#if contactStore.duplicateCandidatesLoading || contactStore.duplicateCandidatesError || contactStore.duplicateCandidates.length > 0}
    <section class="duplicate-warning" aria-live="polite">
      {#if contactStore.duplicateCandidatesLoading}
        <div class="duplicate-status">
          <span class="duplicate-dot" aria-hidden="true"></span>
          {t('contacts.duplicatesChecking')}
        </div>
      {:else if contactStore.duplicateCandidatesError}
        <div class="duplicate-status duplicate-error">
          <strong>{t('contacts.duplicatesCheckFailed')}</strong>
          <span>{contactStore.duplicateCandidatesError}</span>
          <button
            class="btn btn-secondary btn-sm"
            type="button"
            onclick={refreshDuplicateCandidates}
            disabled={duplicateControlsDisabled}
          >
            {t('contacts.duplicatesRetry')}
          </button>
        </div>
      {:else if selectedDuplicateCandidate && duplicateMergePreview}
        <div class="duplicate-summary">
          <div>
            <h2>{t('contacts.duplicatesFound', { count: contactStore.duplicateCandidates.length })}</h2>
            <p>{selectedDuplicateCandidate.reason}</p>
          </div>
          <span class="duplicate-match">
            {selectedDuplicateCandidate.matchType === 'email' ? t('contacts.email') : t('contacts.phone')}: {selectedDuplicateCandidate.matchedValue}
          </span>
        </div>

        <div class="duplicate-controls">
          <label class="duplicate-select-label" for="duplicate-candidate-select">
            {t('contacts.duplicatesReviewLabel')}
          </label>
          <select
            id="duplicate-candidate-select"
            class="input duplicate-select"
            value={selectedDuplicateKey}
            onchange={handleDuplicateCandidateChange}
            disabled={duplicateControlsDisabled}
          >
            {#each contactStore.duplicateCandidates as candidate (duplicateCandidateKey(candidate))}
              <option value={duplicateCandidateKey(candidate)}>
                {duplicateCandidateLabel(candidate)}
              </option>
            {/each}
          </select>
        </div>

        <div class="duplicate-merge-preview">
          <div>
            <span class="preview-label">{t('contacts.duplicatesSource')}</span>
            <strong>{duplicateMergePreview.sourceLabel}</strong>
          </div>
          <div>
            <span class="preview-label">{t('contacts.duplicatesTarget')}</span>
            <strong>{duplicateMergePreview.targetLabel}</strong>
          </div>
          <button
            class="btn btn-ghost btn-sm"
            type="button"
            onclick={() => mergeDirection = mergeDirection === 'suggested' ? 'swapped' : 'suggested'}
            disabled={duplicateControlsDisabled}
          >
            {t('contacts.duplicatesSwap')}
          </button>
        </div>

        <p class="duplicate-explanation">{t('contacts.duplicatesExplanation')}</p>

        <div class="duplicate-actions">
          <button
            class="btn btn-primary btn-sm"
            type="button"
            onclick={handleMergeDuplicateCandidate}
            disabled={duplicateControlsDisabled}
          >
            {contactStore.isMergingDuplicates ? t('contacts.duplicatesMerging') : t('contacts.duplicatesMergeAction')}
          </button>
        </div>
      {/if}
    </section>
  {/if}

  <!-- Table -->
  <div class="contacts-table card">
    <DataTable
      columns={columns as Column[]}
      rows={contactStore.contacts}
      loading={contactStore.isLoading}
      total={contactStore.total}
      page={contactStore.page}
      pageSize={contactStore.pageSize}
      emptyTitle={t('contacts.noContacts')}
      emptyDescription={t('contacts.noContactsDesc')}
      emptyIcon="contacts"
      emptyActionLabel={t('contacts.addContact')}
      onemptyaction={() => uiStore.openModal('addContact')}
      onrowclick={handleRowClick}
      onnextpage={() => contactStore.nextPage()}
      onprevpage={() => contactStore.prevPage()}
    />
  </div>
</div>

<!-- Import/Export Modal -->
<ImportExport bind:open={showImportExport} />

<style>
  .contacts-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .contacts-filters {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    flex-wrap: wrap;
  }

  .search-wrap {
    position: relative;
    flex: 1;
    min-width: 200px;
    max-width: 360px;
  }

  .filter-search-icon {
    position: absolute;
    inset-inline-start: var(--space-4);
    top: 50%;
    transform: translateY(-50%);
    color: var(--icon-muted);
    pointer-events: none;
  }

  .filter-search {
    padding-inline-start: var(--space-10);
    height: 34px;
  }

  .filter-search::-webkit-search-cancel-button {
    display: none;
  }

  .type-filters {
    display: flex;
    gap: var(--space-2);
  }

  .custom-field-filter {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .custom-field-select {
    min-width: 180px;
    height: 34px;
  }

  .custom-field-input {
    min-width: 200px;
    max-width: 320px;
    height: 34px;
  }

  .duplicate-warning {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--color-warning-500);
    border-radius: var(--border-radius-lg);
    background: var(--color-warning-50);
  }

  .duplicate-status {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .duplicate-error {
    color: var(--text-danger);
    flex-wrap: wrap;
  }

  .duplicate-dot {
    width: 8px;
    height: 8px;
    border-radius: 9999px;
    background: var(--color-warning-500);
  }

  .duplicate-summary {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .duplicate-summary h2 {
    margin: 0 0 var(--space-1);
    font-size: var(--text-base);
    line-height: 1.4;
  }

  .duplicate-summary p,
  .duplicate-explanation {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .duplicate-match {
    flex-shrink: 0;
    padding: var(--space-1) var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    background: var(--surface-card);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .duplicate-controls {
    display: grid;
    grid-template-columns: minmax(140px, max-content) minmax(240px, 1fr);
    align-items: center;
    gap: var(--space-3);
  }

  .duplicate-select-label,
  .preview-label {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
  }

  .duplicate-select {
    min-width: 0;
    height: 34px;
  }

  .duplicate-merge-preview {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
  }

  .duplicate-merge-preview > div {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .duplicate-merge-preview strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .duplicate-actions {
    display: flex;
    justify-content: flex-end;
  }

  .filter-chip {
    padding: var(--space-2) var(--space-4);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background-color: transparent;
    border: var(--border-width) solid var(--border-default);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .filter-chip.active {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .contacts-table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  @media (max-width: 760px) {
    .duplicate-summary,
    .duplicate-merge-preview {
      grid-template-columns: 1fr;
    }

    .duplicate-summary {
      display: grid;
    }

    .duplicate-controls {
      grid-template-columns: 1fr;
    }

    .duplicate-actions {
      justify-content: stretch;
    }

    .duplicate-actions .btn {
      width: 100%;
    }
  }
</style>
