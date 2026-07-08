import type { Activity, ActivityType } from '$lib/api/activities';
import type { Deal, DealStage } from '$lib/api/deals';
import { addLocalDays, buildActivityWorkbench } from '$lib/utils/activityWorkbench';
import { isDealClosed, nextDealActivity } from '$lib/utils/pipelineGuidance';

export interface SuggestedActivityDraft {
  subject: string;
  type: ActivityType;
  dueDate: string;
  notes: string;
  contactId: string;
  organizationId: string;
  dealId: string;
}

export interface DealStageFollowUpSuggestion {
  id: string;
  dealId: string;
  dealName: string;
  fromStage: DealStage;
  toStage: DealStage;
  draft: SuggestedActivityDraft;
}

export interface DashboardAttentionItem {
  id: string;
  subject: string;
  dueDate: string | null;
  bucket: 'overdue' | 'today';
}

export interface DashboardAttentionSummary {
  overdueCount: number;
  todayCount: number;
  totalCount: number;
  items: DashboardAttentionItem[];
}

export const LOCAL_AUTOMATION_FOLLOW_UP_DAYS = 1;

export function buildDealStageFollowUpSuggestion({
  deal,
  activities,
  fromStage,
  toStage,
  now = new Date(),
  activityContextReady = true,
  activityContextError = false,
}: {
  deal: Deal;
  activities: Activity[];
  fromStage: DealStage;
  toStage: DealStage;
  now?: Date;
  activityContextReady?: boolean;
  activityContextError?: boolean;
}): DealStageFollowUpSuggestion | null {
  if (!activityContextReady || activityContextError) {
    return null;
  }

  if (
    fromStage === toStage ||
    isDealClosed({ stage: fromStage }) ||
    isDealClosed({ stage: toStage }) ||
    isDealClosed(deal)
  ) {
    return null;
  }

  if (nextDealActivity(activities, now)) {
    return null;
  }

  const dueDate = addLocalDays(now, LOCAL_AUTOMATION_FOLLOW_UP_DAYS);

  return {
    id: `${deal.id}:${fromStage}:${toStage}:${dueDate}`,
    dealId: deal.id,
    dealName: deal.name,
    fromStage,
    toStage,
    draft: {
      subject: `Follow up on ${deal.name}`,
      type: 'task',
      dueDate,
      notes: `Suggested locally after moving the deal to ${toStage}. Review and save only if this next step is right.`,
      contactId: deal.contactId ?? '',
      organizationId: deal.organizationId ?? '',
      dealId: deal.id,
    },
  };
}

export function buildDashboardAttentionSummary(
  activities: Activity[],
  now: Date = new Date(),
  limit = 4,
): DashboardAttentionSummary {
  const workbench = buildActivityWorkbench(activities, now);
  const overdue = workbench.buckets.find((bucket) => bucket.bucket === 'overdue')?.activities ?? [];
  const today = workbench.buckets.find((bucket) => bucket.bucket === 'today')?.activities ?? [];
  const overdueIds = new Set(overdue.map((activity) => activity.id));
  const items = [...overdue, ...today].slice(0, limit).map((activity) => ({
    id: activity.id,
    subject: activity.subject,
    dueDate: activity.dueDate,
    bucket: overdueIds.has(activity.id) ? 'overdue' as const : 'today' as const,
  }));

  return {
    overdueCount: workbench.summary.overdue,
    todayCount: workbench.summary.today,
    totalCount: workbench.summary.overdue + workbench.summary.today,
    items,
  };
}
