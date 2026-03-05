<script lang="ts">
  /**
   * Pipeline.svelte — Kanban pipeline board for 900CRM.
   *
   * Renders deal cards grouped by pipeline stage using the KanbanBoard
   * component. Each column shows deal count and total value. Add deal
   * button per column triggers addDeal modal with the stage pre-selected.
   *
   * Drag-and-drop is handled inside KanbanBoard via native HTML5 drag API.
   * Stage moves call dealStore.moveDealStage() with optimistic updates.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { dealStore } from '$lib/stores/deals';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import type { DealStage, Deal } from '$lib/api/deals';
  import { DEAL_STAGES } from '$lib/api/deals';
  import {
    listCustomFieldDefinitions,
    listCustomFieldValuesForEntityType,
    type CustomFieldDefinition,
    type EntityTypeCustomFieldValue,
  } from '$lib/api/customFields';
  import { formatCurrency } from '$lib/utils/formatters';
  import DealCard from '$lib/components/DealCard.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';

  // ── State ────────────────────────────────────────────────────────────────────

  /** ID of the deal being dragged. */
  let draggingId = $state<string | null>(null);

  /** Stage column currently being dragged over. */
  let dragOverStage = $state<DealStage | null>(null);
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let selectedCustomFieldDefId = $state('');
  let customFieldQuery = $state('');
  let customFieldsLoading = $state(true);
  let customFieldValuesLoading = $state(false);
  let customFieldFilterError = $state<string | null>(null);
  let customFieldValueIndex = $state<Record<string, Record<string, string>>>({});

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(async () => {
    await Promise.all([
      dealStore.loadPipelineBoard(),
      loadCustomFieldDefinitions(),
    ]);
  });

  // ── Drag & drop handlers ────────────────────────────────────────────────────

  function handleDragStart(dealId: string, _stage: DealStage) {
    draggingId = dealId;
  }

  function handleDragEnd() {
    draggingId = null;
    dragOverStage = null;
  }

  function handleDragOver(event: DragEvent, stage: DealStage) {
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
    dragOverStage = stage;
  }

  function handleDragLeave() {
    dragOverStage = null;
  }

  async function handleDrop(event: DragEvent, toStage: DealStage) {
    event.preventDefault();
    dragOverStage = null;
    if (!draggingId) return;
    const id = draggingId;
    draggingId = null;
    await dealStore.moveDealStage(id, toStage);
  }

  // ── Add deal ─────────────────────────────────────────────────────────────────

  /**
   * Open the add deal modal pre-selecting a stage.
   * The modal is responsible for reading uiStore.modalContext.stage.
   */
  function openAddDeal(stage: DealStage) {
    uiStore.openModal('addDeal', { stage });
  }

  async function loadCustomFieldDefinitions() {
    customFieldsLoading = true;
    try {
      customFieldDefinitions = await listCustomFieldDefinitions('deal');
    } finally {
      customFieldsLoading = false;
    }
  }

  async function ensureCustomFieldValueIndex() {
    if (Object.keys(customFieldValueIndex).length > 0) return;

    customFieldValuesLoading = true;
    customFieldFilterError = null;
    try {
      const values = await listCustomFieldValuesForEntityType('deal');
      customFieldValueIndex = indexCustomFieldValues(values);
    } catch (err) {
      customFieldFilterError = t('common.filterLoadFailed');
      console.error('[Pipeline] Failed to load custom-field filter values:', err);
    } finally {
      customFieldValuesLoading = false;
    }
  }

  function indexCustomFieldValues(values: EntityTypeCustomFieldValue[]): Record<string, Record<string, string>> {
    const index: Record<string, Record<string, string>> = {};
    for (const value of values) {
      if (!index[value.field_def_id]) {
        index[value.field_def_id] = {};
      }
      index[value.field_def_id][value.entity_id] = value.value;
    }
    return index;
  }

  async function handleCustomFieldDefinitionChange(event: Event) {
    selectedCustomFieldDefId = (event.target as HTMLSelectElement).value;
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
  }

  async function handleCustomFieldQueryInput(event: Event) {
    customFieldQuery = (event.target as HTMLInputElement).value;
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
  }

  function clearCustomFieldFilter() {
    selectedCustomFieldDefId = '';
    customFieldQuery = '';
  }

  function matchesCustomField(dealId: string): boolean {
    const query = customFieldQuery.trim().toLowerCase();
    if (!selectedCustomFieldDefId || !query) {
      return true;
    }
    if (customFieldValuesLoading || customFieldFilterError) {
      return true;
    }

    const rawValue = customFieldValueIndex[selectedCustomFieldDefId]?.[dealId] ?? '';
    return rawValue.toLowerCase().includes(query);
  }

  // ── Derived ─────────────────────────────────────────────────────────────────

  /** Column metadata derived from dealsByStage. */
  const columns = $derived(
    DEAL_STAGES.map((stage) => {
      const deals = (dealStore.dealsByStage[stage] ?? []).filter((deal) => matchesCustomField(deal.id));
      const totalValue = deals.reduce((sum, d) => sum + (d.value ?? 0), 0);
      return { stage, deals, totalValue };
    })
  );

  // ── Stage label colors ───────────────────────────────────────────────────────

  const stageColors: Record<DealStage, string> = {
    lead:        '#20808D',
    qualified:   '#2D8659',
    proposal:    '#D4A017',
    negotiation: '#A84B2F',
    closedWon:   '#2D8659',
    closedLost:  '#C0392B',
  };
</script>

<div class="page-content pipeline-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('deals.title')}</h1>
    <div class="toolbar">
      <button
        class="btn btn-primary btn-sm"
        onclick={() => openAddDeal('lead')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('deals.addDeal')}
      </button>
    </div>
  </div>

  <!-- Loading skeleton -->
  <div class="pipeline-filters" role="group" aria-label={t('common.customFieldFilter')}>
    <span class="pipeline-filters-label">{t('common.customField')}:</span>
    <select
      class="input pipeline-filter-select"
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
      class="input pipeline-filter-input selectable"
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
    {#if customFieldValuesLoading}
      <span class="pipeline-filters-label">{t('common.loading')}</span>
    {/if}
  </div>

  {#if customFieldFilterError}
    <div class="filter-error" role="status">{customFieldFilterError}</div>
  {/if}

  {#if dealStore.isLoading}
    <div class="kanban-loading" aria-label={t('common.loading')}>
      {#each DEAL_STAGES as stage (stage)}
        <div class="kanban-column-skeleton">
          <div class="skeleton skeleton-col-header"></div>
          {#each [1, 2, 3] as i (i)}
            <div class="skeleton skeleton-card"></div>
          {/each}
        </div>
      {/each}
    </div>

  {:else}
    <!-- ── Kanban board ─────────────────────────────────────────────────────── -->
    <div class="kanban-board" role="region" aria-label={t('deals.title')}>
      {#each columns as col (col.stage)}
        {@const isOver = dragOverStage === col.stage}
        <div
          class="kanban-column"
          class:kanban-column--drag-over={isOver}
          ondragover={(e) => handleDragOver(e, col.stage)}
          ondragleave={handleDragLeave}
          ondrop={(e) => handleDrop(e, col.stage)}
          role="list"
          aria-label={t(`deals.stages.${col.stage}`)}
        >
          <!-- Column header -->
          <div class="col-header" style="--stage-color: {stageColors[col.stage]}">
            <div class="col-header-left">
              <span class="col-stage-dot" style="background-color: {stageColors[col.stage]}" aria-hidden="true"></span>
              <h2 class="col-title">{t(`deals.stages.${col.stage}`)}</h2>
              <span class="col-count" aria-label="{col.deals.length} {t('deals.addDeal')}">
                {col.deals.length}
              </span>
            </div>
            <button
              class="btn-add-deal"
              onclick={() => openAddDeal(col.stage)}
              type="button"
              aria-label="{t('deals.addDeal')} — {t(`deals.stages.${col.stage}`)}"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M6 1v10M1 6h10"/>
              </svg>
            </button>
          </div>

          <!-- Column total value -->
          {#if col.deals.length > 0}
            <div class="col-total">
              <span class="col-total-label">{t('deals.totalValue')}:</span>
              <span class="col-total-value">
                {formatCurrency(col.totalValue, settingsStore.currency, settingsStore.language)}
              </span>
            </div>
          {/if}

          <!-- Deal cards -->
          <div class="col-cards">
            {#if col.deals.length === 0}
              <div class="col-empty" aria-label={t('deals.noDeals')}>
                <p class="col-empty-text">{t('deals.noDeals')}</p>
              </div>
            {:else}
              {#each col.deals as deal (deal.id)}
                <div
                  class="card-wrapper"
                  class:card-wrapper--dragging={draggingId === deal.id}
                  draggable="true"
                  ondragstart={() => handleDragStart(deal.id, col.stage)}
                  ondragend={handleDragEnd}
                  role="listitem"
                >
                  <DealCard {deal} />
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .pipeline-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
    min-height: 0;
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .pipeline-filters {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .pipeline-filters-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .pipeline-filter-select {
    min-width: 180px;
    height: 32px;
  }

  .pipeline-filter-input {
    min-width: 180px;
    height: 32px;
  }

  .filter-error {
    font-size: var(--text-xs);
    color: var(--text-danger);
  }

  /* ── Board ───────────────────────────────────────────────────────────────── */

  .kanban-board {
    display: flex;
    gap: var(--space-4);
    overflow-x: auto;
    flex: 1;
    min-height: 0;
    padding-block-end: var(--space-4);
    overscroll-behavior-x: contain;
    -webkit-overflow-scrolling: touch;

    /* Custom scrollbar */
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
  }

  .kanban-board::-webkit-scrollbar {
    height: 6px;
  }

  .kanban-board::-webkit-scrollbar-track {
    background: transparent;
  }

  .kanban-board::-webkit-scrollbar-thumb {
    background-color: var(--border-default);
    border-radius: 3px;
  }

  /* ── Column ──────────────────────────────────────────────────────────────── */

  .kanban-column {
    flex: 0 0 268px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    background-color: var(--surface-raised);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    min-height: 300px;
    border: 2px solid transparent;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .kanban-column--drag-over {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
  }

  /* ── Column header ───────────────────────────────────────────────────────── */

  .col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .col-header-left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .col-stage-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .col-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 var(--space-2);
    background-color: var(--surface-default);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  /* ── Add deal button ─────────────────────────────────────────────────────── */

  .btn-add-deal {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .btn-add-deal:hover {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  /* ── Column total ────────────────────────────────────────────────────────── */

  .col-total {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    padding-block-end: var(--space-2);
    border-block-end: var(--border-width) solid var(--border-default);
  }

  .col-total-label {
    color: var(--text-secondary);
  }

  .col-total-value {
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
  }

  /* ── Cards area ──────────────────────────────────────────────────────────── */

  .col-cards {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    flex: 1;
    min-height: 80px;
  }

  .card-wrapper {
    cursor: grab;
    border-radius: var(--radius-md);
    transition: opacity var(--duration-fast) var(--ease-out),
                box-shadow var(--duration-fast) var(--ease-out);
  }

  .card-wrapper:active {
    cursor: grabbing;
  }

  .card-wrapper--dragging {
    opacity: 0.4;
  }

  /* ── Empty column state ──────────────────────────────────────────────────── */

  .col-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 80px;
    border-radius: var(--radius-md);
    border: 1px dashed var(--border-default);
  }

  .col-empty-text {
    font-size: var(--text-xs);
    color: var(--text-muted, var(--text-secondary));
    text-align: center;
    margin: 0;
    opacity: 0.7;
  }

  /* ── Loading skeleton ────────────────────────────────────────────────────── */

  .kanban-loading {
    display: flex;
    gap: var(--space-4);
    overflow-x: hidden;
    padding-block-end: var(--space-4);
  }

  .kanban-column-skeleton {
    flex: 0 0 268px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    background-color: var(--surface-raised);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .skeleton-col-header {
    height: 28px;
    border-radius: var(--radius-sm);
  }

  .skeleton-card {
    height: 88px;
    border-radius: var(--radius-md);
  }
</style>
