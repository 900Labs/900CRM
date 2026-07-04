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
  import { createActivity } from '$lib/api/activities';
  import { createContact } from '$lib/api/contacts';
  import { getDashboardStats } from '$lib/api/dashboard';
  import type { DashboardStats } from '$lib/api/dashboard';
  import { createDeal } from '$lib/api/deals';
  import { createOrganization } from '$lib/api/organizations';
  import {
    getActivityFunnelReport,
    getPipelineConversionReport,
    type ActivityFunnelReport,
    type PipelineConversionReport,
  } from '$lib/api/reports';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { formatCompactNumber, formatCurrency, formatPercent } from '$lib/utils/formatters';
  import StatCard from '$lib/components/StatCard.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';

  // ── State ───────────────────────────────────────────────────────────────────

  let stats = $state<DashboardStats | null>(null);
  let pipelineReport = $state<PipelineConversionReport | null>(null);
  let activityReport = $state<ActivityFunnelReport | null>(null);
  let statsLoading = $state(true);
  let reportsLoading = $state(true);
  let error = $state<string | null>(null);
  let reportError = $state<string | null>(null);
  let sampleLoading = $state(false);
  let sampleLoaded = $state(false);
  let sampleMessage = $state<string | null>(null);
  let sampleError = $state<string | null>(null);
  let componentMounted = false;

  const DASHBOARD_LOAD_TIMEOUT_MS = 8_000;

  const stageLabelKeyMap: Record<string, string> = {
    Lead: 'lead',
    Qualified: 'qualified',
    Proposal: 'proposal',
    Negotiation: 'negotiation',
    'Closed Won': 'closedWon',
    'Closed Lost': 'closedLost',
  };

  // ── Derived ────────────────────────────────────────────────────────────────

  const pipelineFormatted = $derived(
    (() => {
      if (!stats) return '—';
      const buckets = stats.pipelineValueByCurrency.filter((bucket) => bucket.totalValue !== 0);

      if (buckets.length === 0) {
        return formatCurrency(0, settingsStore.currency, settingsStore.language);
      }

      if (buckets.length === 1) {
        return formatCurrency(buckets[0].totalValue, buckets[0].currency, settingsStore.language);
      }

      const preview = buckets
        .slice(0, 2)
        .map((bucket) => formatCurrency(bucket.totalValue, bucket.currency, settingsStore.language))
        .join(' · ');

      return buckets.length > 2 ? `${preview} +${buckets.length - 2}` : preview;
    })()
  );

  const visibleStageMetrics = $derived(
    (pipelineReport?.stage_metrics ?? []).filter((metric) => metric.count > 0 || metric.stage_share > 0).slice(0, 6)
  );

  const visibleActivityTypes = $derived(
    (activityReport?.by_type ?? []).slice(0, 4)
  );

  const hasFirstRunContacts = $derived((stats?.totalContacts ?? 0) > 0);
  const hasFirstRunDeals = $derived((stats?.activeDeals ?? 0) > 0);
  const hasFirstRunActivities = $derived((stats?.upcomingTasks ?? 0) > 0);

  const showFirstRun = $derived(
    !sampleLoaded && !error && (!hasFirstRunContacts || !hasFirstRunDeals || !hasFirstRunActivities)
  );

  const sampleDataAvailable = $derived(
    !sampleLoaded && !hasFirstRunContacts && !hasFirstRunDeals && !hasFirstRunActivities
  );

  function asPercentRatio(ratio: number, decimals = 1): string {
    return formatPercent(Math.max(0, Math.min(100, ratio * 100)), decimals);
  }

  function ratioWidth(ratio: number): string {
    return `${Math.max(0, Math.min(100, ratio * 100))}%`;
  }

  function stageLabel(stage: string): string {
    const key = stageLabelKeyMap[stage];
    return key ? t(`deals.stages.${key}`) : stage;
  }

  function activityTypeLabel(type: string): string {
    const normalized = type.trim().toLowerCase();
    const map: Record<string, string> = {
      task: t('activities.task'),
      call: t('activities.call'),
      meeting: t('activities.meeting'),
      email: t('activities.email'),
      note: t('activities.notes'),
    };
    return map[normalized] ?? type;
  }

  function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        reject(new Error(`${label} timed out after ${DASHBOARD_LOAD_TIMEOUT_MS}ms`));
      }, DASHBOARD_LOAD_TIMEOUT_MS);

      promise.then(
        (value) => {
          window.clearTimeout(timeout);
          resolve(value);
        },
        (err) => {
          window.clearTimeout(timeout);
          reject(err);
        },
      );
    });
  }

  async function loadStats(): Promise<void> {
    statsLoading = true;
    error = null;

    try {
      const dashStats = await withTimeout(getDashboardStats(), 'Dashboard stats');
      if (!componentMounted) return;
      stats = dashStats;
    } catch (err) {
      if (!componentMounted) return;
      error = t('errors.loadFailed');
      console.error('[Dashboard] Stats load error:', err);
    } finally {
      if (componentMounted) statsLoading = false;
    }
  }

  async function loadUpcomingActivities(): Promise<void> {
    try {
      await withTimeout(activityStore.loadUpcoming(), 'Upcoming activities');
    } catch (err) {
      console.error('[Dashboard] Upcoming activity load error:', err);
    }
  }

  async function loadReports(): Promise<void> {
    reportsLoading = true;
    reportError = null;

    try {
      const [pipelineResult, activityResult] = await Promise.allSettled([
        withTimeout(getPipelineConversionReport(), 'Pipeline report'),
        withTimeout(getActivityFunnelReport(), 'Activity report'),
      ]);

      if (!componentMounted) return;

      if (pipelineResult.status === 'fulfilled') {
        pipelineReport = pipelineResult.value;
      }
      if (activityResult.status === 'fulfilled') {
        activityReport = activityResult.value;
      }
      if (pipelineResult.status === 'rejected' || activityResult.status === 'rejected') {
        reportError = t('dashboard.reports.loadFailed');
      }
    } finally {
      if (componentMounted) reportsLoading = false;
    }
  }

  async function refreshDashboard(): Promise<void> {
    await Promise.allSettled([
      loadStats(),
      loadUpcomingActivities(),
      loadReports(),
    ]);
  }

  function futureIsoDate(daysFromNow: number): string {
    const date = new Date();
    date.setDate(date.getDate() + daysFromNow);
    date.setHours(9, 0, 0, 0);
    return date.toISOString();
  }

  function openDataSettings(): void {
    window.location.hash = '#/settings';
  }

  async function loadSampleWorkspace(): Promise<void> {
    if (sampleLoading) return;

    sampleLoading = true;
    sampleMessage = null;
    sampleError = null;

    try {
      const organization = await createOrganization({
        name: 'Northstar Cooperative',
        email: 'hello@northstar.example',
        phone: '+1 555 0140',
        website: 'https://northstar.example',
        city: 'Austin',
        region: 'TX',
        country: 'United States',
        description: 'Sample account for reviewing 900CRM workflows.',
      });

      const contact = await createContact({
        firstName: 'Amara',
        lastName: 'Okafor',
        email: 'amara@northstar.example',
        phone: '+1 555 0141',
        organization: organization.name,
        type: 'person',
        tags: [],
        notes: 'Sample contact created by the dashboard starter.',
        website: null,
        address: '120 Market Street',
      });

      const deal = await createDeal({
        name: 'Solar inventory rollout',
        value: 18500,
        currency: settingsStore.currency,
        stage: 'proposal',
        probability: 65,
        expectedCloseDate: futureIsoDate(21),
        contactId: contact.id,
        organizationId: organization.id,
        description: 'Sample opportunity for a staged inventory rollout.',
        tags: [],
      });

      await createActivity({
        type: 'call',
        subject: 'Call Amara about rollout timeline',
        notes: 'Confirm stakeholders, target install dates, and next quote details.',
        dueDate: futureIsoDate(2),
        contactId: contact.id,
        dealId: deal.id,
      });

      sampleMessage = t('dashboard.firstRun.sampleLoaded');
      sampleLoaded = true;
      uiStore.toastSuccess(sampleMessage);
      await refreshDashboard();
    } catch (err) {
      sampleError = t('dashboard.firstRun.sampleFailed');
      uiStore.toastError(sampleError);
      console.error('[Dashboard] Sample workspace load error:', err);
    } finally {
      sampleLoading = false;
    }
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(() => {
    componentMounted = true;
    void refreshDashboard();

    return () => {
      componentMounted = false;
    };
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
        value={statsLoading ? '…' : String(stats?.totalContacts ?? 0)}
        icon="users"
        loading={statsLoading}
      />
      <StatCard
        label={t('dashboard.activeDeals')}
        value={statsLoading ? '…' : String(stats?.activeDeals ?? 0)}
        icon="bar-chart"
        loading={statsLoading}
      />
      <StatCard
        label={t('dashboard.pipelineValue')}
        value={statsLoading ? '…' : pipelineFormatted}
        icon="dollar-sign"
        loading={statsLoading}
        accent={true}
      />
      <StatCard
        label={t('dashboard.upcomingTasks')}
        value={statsLoading ? '…' : String(stats?.upcomingTasks ?? 0)}
        icon="check-square"
        loading={statsLoading}
      />
    </section>

    {#if showFirstRun}
      <section class="first-run-panel" aria-labelledby="first-run-heading">
        <div class="first-run-copy">
          <span class="first-run-eyebrow">{t('dashboard.firstRun.eyebrow')}</span>
          <h2 class="first-run-title" id="first-run-heading">{t('dashboard.firstRun.title')}</h2>
          <p>{t('dashboard.firstRun.subtitle')}</p>

          {#if sampleMessage}
            <div class="first-run-status success" role="status">{sampleMessage}</div>
          {/if}
          {#if sampleError}
            <div class="first-run-status error" role="alert">{sampleError}</div>
          {/if}
        </div>

        <div class="first-run-checklist" aria-label={t('dashboard.firstRun.checklistLabel')}>
          <div class:complete={hasFirstRunContacts} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addContact')}</span>
              <small>{t('dashboard.firstRun.addContactHint')}</small>
            </div>
          </div>
          <div class:complete={hasFirstRunDeals} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addDeal')}</span>
              <small>{t('dashboard.firstRun.addDealHint')}</small>
            </div>
          </div>
          <div class:complete={hasFirstRunActivities} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addFollowUp')}</span>
              <small>{t('dashboard.firstRun.addFollowUpHint')}</small>
            </div>
          </div>
        </div>

        <div class="first-run-actions" aria-label={t('dashboard.firstRun.actionsLabel')}>
          {#if sampleDataAvailable}
            <button
              class="btn btn-primary btn-sm"
              disabled={sampleLoading}
              onclick={loadSampleWorkspace}
              type="button"
            >
              {sampleLoading ? t('dashboard.firstRun.loadingSample') : t('dashboard.firstRun.loadSample')}
            </button>
          {/if}
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addContact')} type="button">
            {t('dashboard.firstRun.contactAction')}
          </button>
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addDeal')} type="button">
            {t('dashboard.firstRun.dealAction')}
          </button>
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addActivity')} type="button">
            {t('dashboard.firstRun.followUpAction')}
          </button>
          <button class="btn btn-ghost btn-sm" onclick={openDataSettings} type="button">
            {t('dashboard.firstRun.dataAction')}
          </button>
        </div>
      </section>
    {/if}

    <section class="report-grid" aria-label={t('dashboard.reports.title')}>
      <section class="card report-card" aria-labelledby="pipeline-report-heading">
        <div class="card-header report-header">
          <h2 class="section-title" id="pipeline-report-heading">{t('dashboard.reports.pipelineTitle')}</h2>
          <span class="report-kpi-value">
            {reportsLoading || !pipelineReport ? '…' : asPercentRatio(pipelineReport.overall_win_rate)}
          </span>
        </div>
        <div class="report-subtitle">{t('dashboard.reports.winRate')}</div>

        <div class="report-summary-grid">
          <div class="summary-stat">
            <span class="summary-stat-label">{t('dashboard.reports.closedWon')}</span>
            <span class="summary-stat-value">
              {reportsLoading || !pipelineReport ? '…' : formatCompactNumber(pipelineReport.closed_won)}
            </span>
          </div>
          <div class="summary-stat">
            <span class="summary-stat-label">{t('dashboard.reports.openDeals')}</span>
            <span class="summary-stat-value">
              {reportsLoading || !pipelineReport ? '…' : formatCompactNumber(pipelineReport.open_deals)}
            </span>
          </div>
        </div>

        {#if !reportsLoading && pipelineReport}
          {#if visibleStageMetrics.length === 0}
            <p class="report-empty">{t('dashboard.reports.noPipelineData')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleStageMetrics as metric (metric.stage)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{stageLabel(metric.stage)}</span>
                    <span class="metric-value">{formatCompactNumber(metric.count)}</span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.stage_share)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        {/if}
      </section>

      <section class="card report-card" aria-labelledby="activity-report-heading">
        <div class="card-header report-header">
          <h2 class="section-title" id="activity-report-heading">{t('dashboard.reports.activityTitle')}</h2>
          <span class="report-kpi-value">
            {reportsLoading || !activityReport ? '…' : asPercentRatio(activityReport.completion_rate)}
          </span>
        </div>
        <div class="report-subtitle">{t('dashboard.reports.completionRate')}</div>

        <div class="report-summary-grid">
          <div class="summary-stat">
            <span class="summary-stat-label">{t('dashboard.reports.pending')}</span>
            <span class="summary-stat-value">
              {reportsLoading || !activityReport ? '…' : formatCompactNumber(activityReport.pending_activities)}
            </span>
          </div>
          <div class="summary-stat">
            <span class="summary-stat-label">{t('dashboard.reports.overdueRate')}</span>
            <span class="summary-stat-value">
              {reportsLoading || !activityReport ? '…' : asPercentRatio(activityReport.overdue_rate)}
            </span>
          </div>
        </div>

        {#if !reportsLoading && activityReport}
          {#if visibleActivityTypes.length === 0}
            <p class="report-empty">{t('dashboard.reports.noActivityData')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleActivityTypes as metric (metric.activity_type)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{activityTypeLabel(metric.activity_type)}</span>
                    <span class="metric-value">{asPercentRatio(metric.completion_rate)}</span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.completion_rate)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        {/if}
      </section>
    </section>

    {#if reportError}
      <div class="dashboard-warning" role="status">
        {reportError}
      </div>
    {/if}

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

  .report-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-6);
  }

  .first-run-panel {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(220px, 0.8fr);
    gap: var(--space-6);
    align-items: start;
    padding: var(--space-6);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background: var(--surface-raised);
  }

  .first-run-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .first-run-eyebrow {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .first-run-title {
    margin: 0;
    font-size: var(--text-xl);
    color: var(--text-primary);
  }

  .first-run-copy p {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .first-run-status {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-sm);
  }

  .first-run-status.success {
    background: var(--color-success-50);
    color: var(--color-success-600);
  }

  .first-run-status.error {
    background: var(--color-danger-50);
    color: var(--color-danger-600);
  }

  .first-run-checklist,
  .first-run-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .first-run-step {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    gap: var(--space-3);
    align-items: start;
    color: var(--text-secondary);
  }

  .first-run-step span:not(.step-dot) {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .first-run-step small {
    display: block;
    margin-top: 2px;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.4;
  }

  .step-dot {
    width: 18px;
    height: 18px;
    border-radius: 999px;
    border: var(--border-width) solid var(--border-default);
    background: var(--surface-base);
    margin-top: 1px;
  }

  .first-run-step.complete .step-dot {
    border-color: var(--color-success-500);
    background: var(--color-success-500);
    box-shadow: inset 0 0 0 4px var(--surface-raised);
  }

  .first-run-actions {
    grid-column: 1 / -1;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .report-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .report-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .report-kpi-value {
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .report-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin-top: calc(var(--space-4) * -1);
  }

  .report-summary-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .summary-stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3);
    border-radius: var(--border-radius-sm);
    background: var(--surface-hover);
  }

  .summary-stat-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .summary-stat-value {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .metric-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .metric-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .metric-row-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    font-size: var(--text-xs);
  }

  .metric-label {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metric-value {
    color: var(--text-primary);
    font-weight: var(--weight-medium);
  }

  .metric-bar-track {
    width: 100%;
    height: 6px;
    border-radius: 999px;
    background: var(--surface-hover);
    overflow: hidden;
  }

  .metric-bar-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--color-primary-500);
  }

  .report-empty {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--text-tertiary);
  }

  @media (max-width: 900px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }

    .first-run-panel {
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

  .dashboard-warning {
    padding: var(--space-4) var(--space-5);
    border-radius: var(--border-radius-md);
    background-color: var(--color-warning-50);
    color: var(--color-warning-600);
    font-size: var(--text-sm);
  }
</style>
