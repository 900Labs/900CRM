<script lang="ts">
  /**
   * Reports.svelte - Local reports hub for current pipeline and activity health.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import {
    getActivityFunnelReport,
    getPipelineConversionReport,
    type ActivityDueBuckets,
    type ActivityFunnelReport,
    type ActivityTypeMetric,
    type PipelineConversionReport,
    type PipelineStageMetric,
    type StageTransitionMetric,
  } from '$lib/api/reports';
  import { settingsStore } from '$lib/stores/settings';
  import { formatCompactNumber, formatPercent } from '$lib/utils/formatters';

  let pipelineReport = $state<PipelineConversionReport | null>(null);
  let activityReport = $state<ActivityFunnelReport | null>(null);
  let pipelineLoading = $state(true);
  let activityLoading = $state(true);
  let pipelineError = $state<string | null>(null);
  let activityError = $state<string | null>(null);
  let componentMounted = false;

  const REPORT_LOAD_TIMEOUT_MS = 8_000;

  const stageLabelKeyMap: Record<string, string> = {
    Lead: 'lead',
    Qualified: 'qualified',
    Proposal: 'proposal',
    Negotiation: 'negotiation',
    'Closed Won': 'closedWon',
    'Closed Lost': 'closedLost',
  };

  const visibleStageMetrics = $derived(
    (pipelineReport?.stage_metrics ?? []).filter((metric) => hasStageMetricData(metric))
  );

  const visibleTransitionMetrics = $derived(
    (pipelineReport?.transition_metrics ?? []).filter((metric) => metric.from_count > 0 || metric.to_count > 0)
  );

  const visibleActivityTypes = $derived(
    (activityReport?.by_type ?? []).filter((metric) => metric.total > 0)
  );

  const dueBucketRows = $derived(
    activityReport ? buildDueBucketRows(activityReport.due_buckets) : []
  );

  const reportsLoading = $derived(pipelineLoading || activityLoading);

  const showPipelineEmpty = $derived(
    !pipelineLoading && !pipelineError && pipelineReport !== null && pipelineReport.total_deals === 0
  );

  const showActivityEmpty = $derived(
    !activityLoading && !activityError && activityReport !== null && activityReport.total_activities === 0
  );

  function hasStageMetricData(metric: PipelineStageMetric): boolean {
    return metric.count > 0 || metric.stage_share > 0;
  }

  function buildDueBucketRows(buckets: ActivityDueBuckets) {
    return [
      { id: 'overdue', label: t('reports.activity.dueBuckets.overdue'), value: buckets.overdue },
      { id: 'dueToday', label: t('reports.activity.dueBuckets.dueToday'), value: buckets.due_today },
      { id: 'dueNext7Days', label: t('reports.activity.dueBuckets.dueNext7Days'), value: buckets.due_next_7_days },
      { id: 'dueLater', label: t('reports.activity.dueBuckets.dueLater'), value: buckets.due_later },
      { id: 'noDueDate', label: t('reports.activity.dueBuckets.noDueDate'), value: buckets.no_due_date },
    ];
  }

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

  function formatGeneratedAt(iso: string | null | undefined): string {
    if (!iso) return t('reports.notAvailable');
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return t('reports.notAvailable');

    try {
      return date.toLocaleString(settingsStore.language);
    } catch {
      return date.toISOString();
    }
  }

  function formatActivityTypeDetail(metric: ActivityTypeMetric): string {
    return `${asPercentRatio(metric.completion_rate)} / ${formatCompactNumber(metric.completed)}/${formatCompactNumber(metric.total)}`;
  }

  function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        reject(new Error(`${label} timed out after ${REPORT_LOAD_TIMEOUT_MS}ms`));
      }, REPORT_LOAD_TIMEOUT_MS);

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

  async function loadPipelineReport(): Promise<void> {
    pipelineLoading = true;
    pipelineError = null;

    try {
      const report = await withTimeout(getPipelineConversionReport(), 'Pipeline report');
      if (!componentMounted) return;
      pipelineReport = report;
    } catch (err) {
      if (!componentMounted) return;
      pipelineReport = null;
      pipelineError = t('reports.pipeline.loadFailed');
      console.error('[Reports] Pipeline report load error:', err);
    } finally {
      if (componentMounted) pipelineLoading = false;
    }
  }

  async function loadActivityReport(): Promise<void> {
    activityLoading = true;
    activityError = null;

    try {
      const report = await withTimeout(getActivityFunnelReport(), 'Activity report');
      if (!componentMounted) return;
      activityReport = report;
    } catch (err) {
      if (!componentMounted) return;
      activityReport = null;
      activityError = t('reports.activity.loadFailed');
      console.error('[Reports] Activity report load error:', err);
    } finally {
      if (componentMounted) activityLoading = false;
    }
  }

  async function loadReports(): Promise<void> {
    await Promise.allSettled([
      loadPipelineReport(),
      loadActivityReport(),
    ]);
  }

  onMount(() => {
    componentMounted = true;
    void loadReports();

    return () => {
      componentMounted = false;
    };
  });
</script>

<div class="page-content reports-page">
  <div class="page-header">
    <div>
      <h1 class="page-title">{t('reports.title')}</h1>
      <p class="page-subtitle">{t('reports.subtitle')}</p>
    </div>
    <button class="btn btn-secondary btn-sm" disabled={reportsLoading} onclick={loadReports} type="button">
      {reportsLoading ? t('reports.refreshing') : t('reports.refresh')}
    </button>
  </div>

  <section class="report-kpi-grid" aria-label={t('reports.summaryLabel')}>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.pipeline.winRate')}</span>
      <strong>{pipelineLoading || !pipelineReport ? '...' : asPercentRatio(pipelineReport.overall_win_rate)}</strong>
      <small>{t('reports.pipeline.winRateDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.pipeline.openDeals')}</span>
      <strong>{pipelineLoading || !pipelineReport ? '...' : formatCompactNumber(pipelineReport.open_deals)}</strong>
      <small>{t('reports.pipeline.openDealsDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.activity.completionRate')}</span>
      <strong>{activityLoading || !activityReport ? '...' : asPercentRatio(activityReport.completion_rate)}</strong>
      <small>{t('reports.activity.completionRateDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.activity.overdueRate')}</span>
      <strong>{activityLoading || !activityReport ? '...' : asPercentRatio(activityReport.overdue_rate)}</strong>
      <small>{t('reports.activity.overdueRateDetail')}</small>
    </article>
  </section>

  <div class="reports-grid">
    <section class="card report-section" aria-labelledby="pipeline-report-heading">
      <div class="report-section-header">
        <div>
          <h2 class="section-title" id="pipeline-report-heading">{t('reports.pipeline.title')}</h2>
          <p>{t('reports.pipeline.description')}</p>
        </div>
        <span>{t('reports.generatedAt', { value: formatGeneratedAt(pipelineReport?.generated_at) })}</span>
      </div>

      {#if pipelineError}
        <div class="report-alert" role="status">{pipelineError}</div>
      {:else if pipelineLoading && !pipelineReport}
        <div class="report-loading" role="status">{t('reports.loading')}</div>
      {:else if showPipelineEmpty}
        <p class="report-empty">{t('reports.pipeline.empty')}</p>
      {:else if pipelineReport}
        <div class="summary-row" aria-label={t('reports.pipeline.summaryLabel')}>
          <div>
            <span>{t('reports.pipeline.closedWon')}</span>
            <strong>{formatCompactNumber(pipelineReport.closed_won)}</strong>
          </div>
          <div>
            <span>{t('reports.pipeline.closedLost')}</span>
            <strong>{formatCompactNumber(pipelineReport.closed_lost)}</strong>
          </div>
          <div>
            <span>{t('reports.pipeline.totalDeals')}</span>
            <strong>{formatCompactNumber(pipelineReport.total_deals)}</strong>
          </div>
        </div>

        <div class="report-block">
          <h3>{t('reports.pipeline.stageDistribution')}</h3>
          {#if visibleStageMetrics.length === 0}
            <p class="report-empty">{t('reports.pipeline.emptyStages')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleStageMetrics as metric (metric.stage)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{stageLabel(metric.stage)}</span>
                    <span class="metric-value">
                      {formatCompactNumber(metric.count)} / {asPercentRatio(metric.stage_share)}
                    </span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.stage_share)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="report-block">
          <h3>{t('reports.pipeline.currentFunnel')}</h3>
          <p class="report-note">{t('reports.pipeline.currentFunnelNote')}</p>
          {#if visibleTransitionMetrics.length === 0}
            <p class="report-empty">{t('reports.pipeline.emptyFunnel')}</p>
          {:else}
            <ul class="funnel-list" role="list">
              {#each visibleTransitionMetrics as metric (metric.from_stage + metric.to_stage)}
                <li class="funnel-row">
                  <span>{stageLabel(metric.from_stage)} -> {stageLabel(metric.to_stage)}</span>
                  <strong>{asPercentRatio(metric.ratio)}</strong>
                  <small>{formatCompactNumber(metric.to_count)} / {formatCompactNumber(metric.from_count)}</small>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>

    <section class="card report-section" aria-labelledby="activity-report-heading">
      <div class="report-section-header">
        <div>
          <h2 class="section-title" id="activity-report-heading">{t('reports.activity.title')}</h2>
          <p>{t('reports.activity.description')}</p>
        </div>
        <span>{t('reports.generatedAt', { value: formatGeneratedAt(activityReport?.generated_at) })}</span>
      </div>

      {#if activityError}
        <div class="report-alert" role="status">{activityError}</div>
      {:else if activityLoading && !activityReport}
        <div class="report-loading" role="status">{t('reports.loading')}</div>
      {:else if showActivityEmpty}
        <p class="report-empty">{t('reports.activity.empty')}</p>
      {:else if activityReport}
        <div class="summary-row" aria-label={t('reports.activity.summaryLabel')}>
          <div>
            <span>{t('reports.activity.completed')}</span>
            <strong>{formatCompactNumber(activityReport.completed_activities)}</strong>
          </div>
          <div>
            <span>{t('reports.activity.pending')}</span>
            <strong>{formatCompactNumber(activityReport.pending_activities)}</strong>
          </div>
          <div>
            <span>{t('reports.activity.overdue')}</span>
            <strong>{formatCompactNumber(activityReport.overdue_activities)}</strong>
          </div>
        </div>

        <div class="report-block">
          <h3>{t('reports.activity.dueBuckets.title')}</h3>
          <ul class="bucket-grid" role="list">
            {#each dueBucketRows as bucket (bucket.id)}
              <li>
                <span>{bucket.label}</span>
                <strong>{formatCompactNumber(bucket.value)}</strong>
              </li>
            {/each}
          </ul>
        </div>

        <div class="report-block">
          <h3>{t('reports.activity.byType')}</h3>
          {#if visibleActivityTypes.length === 0}
            <p class="report-empty">{t('reports.activity.emptyTypes')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleActivityTypes as metric (metric.activity_type)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{activityTypeLabel(metric.activity_type)}</span>
                    <span class="metric-value">
                      {formatActivityTypeDetail(metric)}
                    </span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.completion_rate)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .reports-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .page-header {
    align-items: flex-start;
    gap: var(--space-4);
  }

  .page-subtitle {
    margin: var(--space-2) 0 0;
    max-width: 760px;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .report-kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: var(--space-4);
  }

  .report-kpi {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-5);
  }

  .report-kpi-label {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .report-kpi strong {
    color: var(--text-primary);
    font-size: var(--text-2xl);
    line-height: 1;
  }

  .report-kpi small {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .reports-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-6);
    align-items: start;
  }

  .report-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-6);
  }

  .report-section-header {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: flex-start;
  }

  .report-section-header p {
    margin: var(--space-2) 0 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .report-section-header > span {
    flex-shrink: 0;
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    text-align: end;
  }

  .section-title {
    margin: 0;
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .summary-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .summary-row div,
  .bucket-grid li {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
    border-radius: var(--border-radius-sm);
    background: var(--surface-hover);
  }

  .summary-row span,
  .bucket-grid span {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
  }

  .summary-row strong,
  .bucket-grid strong {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .report-block {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .report-block h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .report-note {
    margin: 0;
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    line-height: 1.4;
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
    text-align: end;
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

  .funnel-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .funnel-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: var(--space-3);
    align-items: center;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-xs);
  }

  .funnel-row span {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .funnel-row strong {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .funnel-row small {
    color: var(--text-tertiary);
  }

  .bucket-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-3);
  }

  .report-alert,
  .report-loading,
  .report-empty {
    margin: 0;
    padding: var(--space-4);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-sm);
  }

  .report-alert {
    background: var(--color-warning-50);
    color: var(--color-warning-600);
  }

  .report-loading,
  .report-empty {
    background: var(--surface-hover);
    color: var(--text-tertiary);
  }

  @media (max-width: 1100px) {
    .reports-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .report-section-header,
    .page-header {
      flex-direction: column;
    }

    .report-section-header > span {
      text-align: start;
    }

    .summary-row {
      grid-template-columns: 1fr;
    }

    .funnel-row {
      grid-template-columns: 1fr auto;
    }

    .funnel-row small {
      grid-column: 1 / -1;
    }
  }
</style>
