<script lang="ts">
  /**
   * PendingActions.svelte - Read-only proposed-action review surface.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { listPendingProposedActions, type ProposedAction } from '$lib/api/proposedActions';
  import { settingsStore } from '$lib/stores/settings';

  let actions = $state<ProposedAction[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(() => {
    void loadActions();
  });

  async function loadActions() {
    isLoading = true;
    error = null;

    try {
      actions = await listPendingProposedActions();
    } catch (err) {
      console.error('[PendingActions] Failed to load pending actions:', err);
      error = t('pendingActions.loadFailed');
    } finally {
      isLoading = false;
    }
  }

  function formatTimestamp(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value || '-';

    try {
      return new Intl.DateTimeFormat(settingsStore.language, {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(date);
    } catch {
      return date.toLocaleString();
    }
  }

  function compactId(value: string | null): string {
    if (!value) return '-';
    return value.length > 18 ? `${value.slice(0, 8)}...${value.slice(-6)}` : value;
  }

  function entityLabel(type: string | null, id: string | null): string {
    if (!type && !id) return '-';
    if (!type) return compactId(id);
    if (!id) return type;
    return `${type} / ${compactId(id)}`;
  }

  function jsonSummary(value: string | null): string {
    if (!value) return t('pendingActions.noJson');

    try {
      const parsed = JSON.parse(value) as unknown;
      if (Array.isArray(parsed)) {
        return t('pendingActions.jsonArrayCount', { count: String(parsed.length) });
      }
      if (parsed && typeof parsed === 'object') {
        const keys = Object.keys(parsed);
        if (keys.length === 0) return t('pendingActions.jsonEmptyObject');
        const preview = keys.slice(0, 3).join(', ');
        return keys.length > 3
          ? t('pendingActions.jsonObjectPreviewMore', { keys: preview, count: String(keys.length - 3) })
          : t('pendingActions.jsonObjectPreview', { keys: preview });
      }
      return String(parsed);
    } catch {
      const trimmed = value.trim();
      return trimmed.length > 48 ? `${trimmed.slice(0, 48)}...` : trimmed;
    }
  }

  function jsonBlock(value: string | null): string {
    if (!value) return '';

    try {
      return JSON.stringify(JSON.parse(value), null, 2);
    } catch {
      return value;
    }
  }
</script>

<div class="page-content pending-actions-page">
  <div class="page-header pending-header">
    <div>
      <h1 class="page-title">{t('pendingActions.title')}</h1>
      <p class="page-subtitle">{t('pendingActions.subtitle')}</p>
    </div>

    <button class="btn btn-secondary btn-sm" type="button" onclick={loadActions} disabled={isLoading}>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
        <path d="M3 21v-5h5" />
        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
        <path d="M16 8h5V3" />
      </svg>
      {t('common.refresh')}
    </button>
  </div>

  <div class="pending-table card">
    <div class="table-scroll">
      <table class="pending-list" aria-label={t('pendingActions.title')}>
        <thead>
          <tr>
            <th>{t('pendingActions.actionType')}</th>
            <th>{t('pendingActions.toolName')}</th>
            <th>{t('pendingActions.entity')}</th>
            <th>{t('pendingActions.clientId')}</th>
            <th>{t('pendingActions.createdAt')}</th>
            <th>{t('pendingActions.input')}</th>
            <th>{t('pendingActions.proposedOutput')}</th>
            <th>{t('pendingActions.status')}</th>
            <th>{t('pendingActions.deviceId')}</th>
          </tr>
        </thead>
        <tbody>
          {#if isLoading}
            {#each Array(6) as _, index (index)}
              <tr class="skeleton-row">
                <td colspan="9"><div class="skeleton table-skeleton-cell"></div></td>
              </tr>
            {/each}
          {:else if error}
            <tr>
              <td colspan="9" class="state-cell">
                <div class="state-panel" role="alert">
                  <h2>{error}</h2>
                  <p>{t('pendingActions.loadFailedDesc')}</p>
                  <button class="btn btn-secondary btn-sm" type="button" onclick={loadActions}>
                    {t('common.retry')}
                  </button>
                </div>
              </td>
            </tr>
          {:else if actions.length === 0}
            <tr>
              <td colspan="9" class="state-cell">
                <div class="state-panel">
                  <h2>{t('pendingActions.emptyTitle')}</h2>
                  <p>{t('pendingActions.emptyDesc')}</p>
                </div>
              </td>
            </tr>
          {:else}
            {#each actions as action (action.id)}
              <tr>
                <td><span class="mono strong">{action.actionType}</span></td>
                <td>{action.toolName}</td>
                <td>{entityLabel(action.entityType, action.entityId)}</td>
                <td><span class="mono">{compactId(action.clientId)}</span></td>
                <td>
                  <time datetime={action.createdAt}>{formatTimestamp(action.createdAt)}</time>
                </td>
                <td>
                  <details class="json-detail">
                    <summary>{jsonSummary(action.inputJson)}</summary>
                    <pre class="json-block selectable">{jsonBlock(action.inputJson)}</pre>
                  </details>
                </td>
                <td>
                  {#if action.proposedOutputJson}
                    <details class="json-detail">
                      <summary>{jsonSummary(action.proposedOutputJson)}</summary>
                      <pre class="json-block selectable">{jsonBlock(action.proposedOutputJson)}</pre>
                    </details>
                  {:else}
                    <span class="muted">{t('pendingActions.noJson')}</span>
                  {/if}
                </td>
                <td><span class="status-badge">{action.status}</span></td>
                <td><span class="mono">{compactId(action.deviceId)}</span></td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .pending-actions-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .pending-header {
    align-items: flex-start;
    margin-block-end: 0;
  }

  .page-subtitle {
    margin-block-start: var(--space-2);
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .pending-table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .table-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .pending-list {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .pending-list th,
  .pending-list td {
    padding: var(--space-3) var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
  }

  .pending-list th {
    position: sticky;
    top: 0;
    z-index: 1;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    background-color: var(--surface-card);
  }

  .pending-list td {
    color: var(--text-secondary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .strong {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .muted {
    color: var(--text-tertiary);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    min-height: 20px;
    padding: var(--space-1) var(--space-3);
    border-radius: 9999px;
    background-color: var(--color-warning-50);
    color: var(--color-warning-600);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .state-cell {
    padding: var(--space-10) var(--space-4);
  }

  .state-panel {
    min-height: 240px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-secondary);
    text-align: center;
  }

  .state-panel h2 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  .state-panel p {
    margin: 0;
    max-width: 380px;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .json-detail {
    max-width: 260px;
  }

  .json-detail summary {
    cursor: pointer;
    color: var(--text-accent);
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
  }

  .json-block {
    margin-block-start: var(--space-2);
    max-width: 360px;
    max-height: 180px;
    overflow: auto;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-panel);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
    white-space: pre-wrap;
  }

  @media (max-width: 820px) {
    .pending-header {
      flex-direction: column;
    }
  }
</style>
