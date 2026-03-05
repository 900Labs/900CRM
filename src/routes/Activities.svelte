<script lang="ts">
  /**
   * Activities.svelte — Activity list view for 900CRM.
   *
   * Features:
   *   - Filter by type (task/call/meeting/email) and status (pending/completed/overdue)
   *   - Sorted by due date ascending
   *   - Quick-add activity form (subject, type, due date)
   *   - Mark complete / mark incomplete toggle
   *   - Empty state, loading state, error state
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import type { ActivityType, ActivityStatus, CreateActivityPayload } from '$lib/api/activities';
  import { formatDate, formatRelativeTime } from '$lib/utils/formatters';
  import EmptyState from '$lib/components/EmptyState.svelte';

  // ── State ────────────────────────────────────────────────────────────────────

  let typeFilter   = $state<ActivityType | ''>('');
  let statusFilter = $state<ActivityStatus | ''>('');

  // Quick-add form
  let showQuickAdd  = $state(false);
  let qaSubject     = $state('');
  let qaType        = $state<ActivityType>('task');
  let qaDueDate     = $state('');
  let qaSubmitting  = $state(false);

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(async () => {
    await activityStore.loadActivities();
  });

  // ── Handlers ─────────────────────────────────────────────────────────────────

  async function handleTypeFilter(type: ActivityType | '') {
    typeFilter = type;
    await activityStore.setFilters({
      type: type || undefined,
      status: statusFilter || undefined,
    });
  }

  async function handleStatusFilter(status: ActivityStatus | '') {
    statusFilter = status;
    await activityStore.setFilters({
      type: typeFilter || undefined,
      status: status || undefined,
    });
  }

  async function handleToggleComplete(activity: { id: string; status: ActivityStatus }) {
    if (activity.status === 'completed') {
      await activityStore.markIncomplete(activity.id);
    } else {
      await activityStore.markComplete(activity.id);
    }
  }

  async function handleQuickAdd() {
    if (!qaSubject.trim()) return;
    qaSubmitting = true;
    try {
      const payload: CreateActivityPayload = {
        type:      qaType,
        subject:   qaSubject.trim(),
        notes:     null,
        dueDate:   qaDueDate || null,
        contactId: null,
        dealId:    null,
      };
      await activityStore.createActivity(payload);

      // Reset form
      qaSubject  = '';
      qaType     = 'task';
      qaDueDate  = '';
      showQuickAdd = false;
    } catch (err) {
      console.error('[Activities] Quick-add error:', err);
    } finally {
      qaSubmitting = false;
    }
  }

  function cancelQuickAdd() {
    showQuickAdd  = false;
    qaSubject     = '';
    qaType        = 'task';
    qaDueDate     = '';
  }

  // ── Type icon helper ─────────────────────────────────────────────────────────

  function activityIcon(type: ActivityType): string {
    switch (type) {
      case 'task':    return 'M9 11l3 3L22 4M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11';
      case 'call':    return 'M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.79 19.79 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72 12.84 12.84 0 00.7 2.81 2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45 12.84 12.84 0 002.81.7A2 2 0 0122 16.92z';
      case 'meeting': return 'M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75';
      case 'email':   return 'M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2M22 6l-10 7L2 6';
      default:        return 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5';
    }
  }

  // ── Status badge class ───────────────────────────────────────────────────────

  function statusClass(status: ActivityStatus): string {
    switch (status) {
      case 'completed': return 'status-completed';
      case 'overdue':   return 'status-overdue';
      default:          return 'status-pending';
    }
  }
</script>

<div class="page-content activities-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('activities.title')}</h1>
    <div class="toolbar">
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => showQuickAdd = !showQuickAdd}
        type="button"
        aria-expanded={showQuickAdd}
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('activities.addActivity')}
      </button>
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addActivity')}
        type="button"
      >
        {t('activities.addActivity')} ({t('common.all')})
      </button>
    </div>
  </div>

  <!-- Quick-add form -->
  {#if showQuickAdd}
    <form
      class="card quick-add-form"
      onsubmit={(e) => { e.preventDefault(); handleQuickAdd(); }}
      aria-label={t('activities.addActivity')}
    >
      <div class="quick-add-row">
        <!-- Type selector -->
        <div class="qa-field qa-field--type">
          <label class="sr-only" for="qa-type">{t('activities.type')}</label>
          <select
            id="qa-type"
            class="input select-input"
            bind:value={qaType}
          >
            {#each (['task', 'call', 'meeting', 'email'] as ActivityType[]) as type (type)}
              <option value={type}>{t(`activities.${type}`)}</option>
            {/each}
          </select>
        </div>

        <!-- Subject input -->
        <div class="qa-field qa-field--subject">
          <label class="sr-only" for="qa-subject">{t('activities.subject')}</label>
          <input
            id="qa-subject"
            class="input"
            type="text"
            bind:value={qaSubject}
            placeholder={t('activities.subject')}
            autocomplete="off"
            required
          />
        </div>

        <!-- Due date -->
        <div class="qa-field qa-field--date">
          <label class="sr-only" for="qa-due">{t('activities.dueDate')}</label>
          <input
            id="qa-due"
            class="input"
            type="date"
            bind:value={qaDueDate}
            aria-label={t('activities.dueDate')}
          />
        </div>

        <!-- Actions -->
        <div class="qa-actions">
          <button
            class="btn btn-primary btn-sm"
            type="submit"
            disabled={qaSubmitting || !qaSubject.trim()}
          >
            {qaSubmitting ? t('common.loading') : t('common.add')}
          </button>
          <button
            class="btn btn-secondary btn-sm"
            type="button"
            onclick={cancelQuickAdd}
          >
            {t('common.cancel')}
          </button>
        </div>
      </div>
    </form>
  {/if}

  <!-- Filters -->
  <div class="activities-filters">
    <!-- Type filters -->
    <div class="filter-group" role="group" aria-label={t('activities.type')}>
      <span class="filter-group-label">{t('activities.type')}:</span>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'task', label: t('activities.task') },
        { value: 'call', label: t('activities.call') },
        { value: 'meeting', label: t('activities.meeting') },
        { value: 'email', label: t('activities.email') },
      ] as opt (opt.value)}
        <button
          class="filter-chip"
          class:active={typeFilter === opt.value}
          onclick={() => handleTypeFilter(opt.value as ActivityType | '')}
          type="button"
        >
          {opt.label}
        </button>
      {/each}
    </div>

    <!-- Status filters -->
    <div class="filter-group" role="group" aria-label={t('common.status')}>
      <span class="filter-group-label">{t('common.status')}:</span>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'pending', label: t('activities.upcoming') },
        { value: 'completed', label: t('activities.completed') },
        { value: 'overdue', label: t('activities.overdue') },
      ] as opt (opt.value)}
        <button
          class="filter-chip"
          class:active={statusFilter === opt.value}
          onclick={() => handleStatusFilter(opt.value as ActivityStatus | '')}
          type="button"
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- Activity list -->
  <div class="card activities-list-card">
    {#if activityStore.isLoading}
      <!-- Skeleton rows -->
      <ul class="activities-list" aria-label={t('common.loading')}>
        {#each [1, 2, 3, 4, 5] as i (i)}
          <li class="activity-row skeleton-row">
            <div class="skeleton skeleton-icon"></div>
            <div class="skeleton-text-block">
              <div class="skeleton skeleton-title"></div>
              <div class="skeleton skeleton-sub"></div>
            </div>
          </li>
        {/each}
      </ul>

    {:else if activityStore.activities.length === 0}
      <EmptyState
        icon="activities"
        title={t('activities.noActivities')}
        description={t('activities.noActivitiesDesc')}
        actionLabel={t('activities.addActivity')}
        onaction={() => uiStore.openModal('addActivity')}
      />

    {:else}
      <ul class="activities-list" role="list">
        {#each activityStore.activities as activity (activity.id)}
          <li class="activity-row" class:activity-row--completed={activity.status === 'completed'}>
            <!-- Type icon -->
            <div class="activity-icon-wrap activity-type-{activity.type}" aria-hidden="true">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <path d={activityIcon(activity.type)}/>
              </svg>
            </div>

            <!-- Content -->
            <div class="activity-content">
              <div class="activity-header-row">
                <span class="activity-subject">{activity.subject}</span>
                <span class="activity-status-badge {statusClass(activity.status)}">
                  {t(`activities.${activity.status === 'pending' ? 'upcoming' : activity.status}`)}
                </span>
              </div>
              <div class="activity-meta">
                {#if activity.dueDate}
                  <span class="activity-due">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
                    </svg>
                    {formatDate(activity.dueDate, settingsStore.dateFormat as 'MMM D, YYYY')}
                  </span>
                {/if}
                {#if activity.contactName}
                  <span class="activity-linked">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2M12 11a4 4 0 100-8 4 4 0 000 8"/>
                    </svg>
                    {activity.contactName}
                  </span>
                {/if}
                {#if activity.dealName}
                  <span class="activity-linked">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                    </svg>
                    {activity.dealName}
                  </span>
                {/if}
                <span class="activity-time">{formatRelativeTime(activity.createdAt)}</span>
              </div>
            </div>

            <!-- Mark complete toggle -->
            <button
              class="btn-complete"
              class:btn-complete--done={activity.status === 'completed'}
              onclick={() => handleToggleComplete(activity)}
              type="button"
              aria-label={activity.status === 'completed'
                ? t('activities.markIncomplete')
                : t('activities.markComplete')}
              title={activity.status === 'completed'
                ? t('activities.markIncomplete')
                : t('activities.markComplete')}
            >
              {#if activity.status === 'completed'}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
              {:else}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                  <circle cx="12" cy="12" r="9"/>
                </svg>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .activities-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  /* ── Quick-add form ──────────────────────────────────────────────────────── */

  .quick-add-form {
    padding: var(--space-5);
  }

  .quick-add-row {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .qa-field { flex-shrink: 0; }

  .qa-field--type    { min-width: 120px; }
  .qa-field--subject { flex: 1; min-width: 200px; }
  .qa-field--date    { min-width: 148px; }

  .qa-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .select-input {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2313343B' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--space-3) center;
    padding-inline-end: var(--space-8);
    cursor: pointer;
  }

  /* ── Filters ─────────────────────────────────────────────────────────────── */

  .activities-filters {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .filter-group-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    white-space: nowrap;
    margin-inline-end: var(--space-1);
  }

  .filter-chip {
    padding: var(--space-1) var(--space-3);
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

  /* ── Activity list card ──────────────────────────────────────────────────── */

  .activities-list-card {
    flex: 1;
    overflow: hidden;
  }

  .activities-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  /* ── Activity row ────────────────────────────────────────────────────────── */

  .activity-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-block-end: var(--border-width) solid var(--border-default);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .activity-row:last-child {
    border-block-end: none;
  }

  .activity-row:hover {
    background-color: var(--surface-raised);
  }

  .activity-row--completed {
    opacity: 0.65;
  }

  /* ── Activity type icon ──────────────────────────────────────────────────── */

  .activity-icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-block-start: 1px;
  }

  .activity-type-task    { background: #E8F4F7; color: #20808D; }
  .activity-type-call    { background: #E8F5EC; color: #2D8659; }
  .activity-type-meeting { background: #FFF8E1; color: #D4A017; }
  .activity-type-email   { background: #FEF3E2; color: #A84B2F; }

  /* ── Activity content ────────────────────────────────────────────────────── */

  .activity-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .activity-header-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .activity-subject {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .activity-meta {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex-wrap: wrap;
  }

  .activity-due,
  .activity-linked,
  .activity-time {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* ── Status badges ───────────────────────────────────────────────────────── */

  .activity-status-badge {
    display: inline-block;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    border-radius: 9999px;
    padding: 1px var(--space-2);
    white-space: nowrap;
  }

  .status-pending   { background: #E8F4F7; color: #20808D; }
  .status-completed { background: #E8F5EC; color: #2D8659; }
  .status-overdue   { background: #FFF0F0; color: #C0392B; }

  /* ── Mark complete button ────────────────────────────────────────────────── */

  .btn-complete {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    margin-block-start: 2px;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .btn-complete:hover {
    border-color: var(--color-success);
    color: var(--color-success);
  }

  .btn-complete--done {
    background-color: var(--color-success);
    border-color: var(--color-success);
    color: #fff;
  }

  .btn-complete--done:hover {
    background-color: #1f6640;
    border-color: #1f6640;
    color: #fff;
  }

  /* ── Skeleton rows ───────────────────────────────────────────────────────── */

  .skeleton-row {
    align-items: center;
    padding: var(--space-4) var(--space-6);
  }

  .skeleton-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .skeleton-text-block {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .skeleton-title {
    height: 14px;
    width: 60%;
    border-radius: var(--radius-sm);
  }

  .skeleton-sub {
    height: 11px;
    width: 40%;
    border-radius: var(--radius-sm);
  }

  /* ── Screen reader only ──────────────────────────────────────────────────── */

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>
