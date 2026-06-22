<script lang="ts">
  /**
   * Dashboard.svelte — Main dashboard view for 900CRM.
   *
   * Displays:
   *   - 4 KPI stat cards (total contacts, active deals, pipeline value, upcoming tasks)
   *   - Recent activity feed
   *   - Quick action buttons
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { getDashboardStats } from '$lib/api/dashboard';
  import type { DashboardStats } from '$lib/api/dashboard';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { formatCurrency } from '$lib/utils/formatters';
  import StatCard from '$lib/components/StatCard.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';

  // ── State ───────────────────────────────────────────────────────────────────

  let stats = $state<DashboardStats | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  // ── Derived ────────────────────────────────────────────────────────────────

  const pipelineFormatted = $derived(
    stats
      ? formatCurrency(stats.pipelineValue, stats.currency, settingsStore.language)
      : '—'
  );

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(async () => {
    isLoading = true;
    try {
      const [dashStats] = await Promise.all([
        getDashboardStats(),
        activityStore.loadUpcoming(),
      ]);
      stats = dashStats;
    } catch (err) {
      error = t('errors.loadFailed');
      console.error('[Dashboard] Load error:', err);
    } finally {
      isLoading = false;
    }
  });
</script>

<div class="page-content dashboard-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('dashboard.title')}</h1>
    <div class="dashboard-actions">
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => uiStore.openModal('addContact')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('dashboard.addContact')}
      </button>
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addDeal')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('dashboard.addDeal')}
      </button>
    </div>
  </div>

  {#if error}
    <div class="dashboard-error" role="alert">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      {error}
    </div>
  {:else}
    <!-- KPI Cards -->
    <section class="stat-grid" aria-label={t('dashboard.title')}>
      <StatCard
        label={t('dashboard.totalContacts')}
        value={isLoading ? '…' : String(stats?.totalContacts ?? 0)}
        icon="users"
        loading={isLoading}
      />
      <StatCard
        label={t('dashboard.activeDeals')}
        value={isLoading ? '…' : String(stats?.activeDeals ?? 0)}
        icon="bar-chart"
        loading={isLoading}
      />
      <StatCard
        label={t('dashboard.pipelineValue')}
        value={isLoading ? '…' : pipelineFormatted}
        icon="dollar-sign"
        loading={isLoading}
        accent={true}
      />
      <StatCard
        label={t('dashboard.upcomingTasks')}
        value={isLoading ? '…' : String(stats?.upcomingTasks ?? 0)}
        icon="check-square"
        loading={isLoading}
      />
    </section>

    <!-- Content grid: activity feed + quick actions -->
    <div class="dashboard-grid">
      <!-- Recent Activity -->
      <section class="card dashboard-activity" aria-labelledby="activity-heading">
        <div class="card-header">
          <h2 class="section-title" id="activity-heading">
            {t('dashboard.recentActivity')}
          </h2>
        </div>
        <div class="card-body">
          <ActivityFeed
            activities={activityStore.upcoming}
            loading={activityStore.isLoading}
            maxItems={8}
          />
        </div>
      </section>

      <!-- Quick Actions -->
      <section class="card dashboard-quick" aria-labelledby="quick-heading">
        <div class="card-header">
          <h2 class="section-title" id="quick-heading">
            {t('dashboard.quickActions')}
          </h2>
        </div>
        <div class="card-body quick-actions">
          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addContact')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8M19 8v6M22 11h-6"/>
              </svg>
            </div>
            <span>{t('contacts.addContact')}</span>
          </button>

          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addDeal')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
            <span>{t('deals.addDeal')}</span>
          </button>

          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addActivity')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
              </svg>
            </div>
            <span>{t('activities.addActivity')}</span>
          </button>
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .dashboard-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .dashboard-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-6);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: var(--space-6);
    align-items: start;
  }

  @media (max-width: 900px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }

  .section-title {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .dashboard-activity .card-body,
  .dashboard-quick .card-body {
    padding-block-start: var(--space-4);
  }

  .quick-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .quick-action-btn {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    border-radius: var(--border-radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    cursor: pointer;
    width: 100%;
    text-align: start;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .quick-action-btn:hover {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .quick-icon {
    color: var(--color-primary-500);
    flex-shrink: 0;
  }

  .dashboard-error {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-5) var(--space-6);
    background-color: var(--color-danger-50);
    color: var(--color-danger-600);
    border-radius: var(--border-radius-md);
    font-size: var(--text-sm);
  }
</style>
