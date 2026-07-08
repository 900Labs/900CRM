import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import {
  buildDashboardAttentionSummary,
  buildDealStageFollowUpSuggestion,
} from '$lib/utils/localAutomation';

const NOW = new Date('2026-07-08T16:30:00');

function deal(overrides: Partial<Deal> = {}): Deal {
  return {
    id: 'deal-1',
    name: 'Solar rollout',
    value: 12000,
    currency: 'USD',
    stage: 'proposal',
    probability: 60,
    expectedCloseDate: '2026-08-01',
    contactId: 'contact-1',
    organizationId: 'organization-1',
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-08T08:00:00Z',
    ...overrides,
  };
}

function activity(overrides: Partial<Activity>): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-07-08',
    completedAt: null,
    status: 'pending',
    contactId: null,
    contactName: null,
    dealId: 'deal-1',
    dealName: null,
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

describe('local automation helpers', () => {
  it('drafts an explicit follow-up suggestion after an open stage move with no linked activity', () => {
    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toMatchObject({
      dealId: 'deal-1',
      dealName: 'Solar rollout',
      fromStage: 'lead',
      toStage: 'qualified',
      draft: {
        subject: 'Follow up on Solar rollout',
        type: 'task',
        dueDate: '2026-07-09',
        contactId: 'contact-1',
        organizationId: 'organization-1',
        dealId: 'deal-1',
      },
    });
  });

  it('does not suggest a draft for closed deals, unchanged stages, loaded activities, or unresolved context', () => {
    const futureActivity = activity({ dueDate: '2026-07-10' });

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'closedWon' }),
        activities: [],
        fromStage: 'proposal',
        toStage: 'closedWon',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'closedWon',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'proposal' }),
        activities: [],
        fromStage: 'proposal',
        toStage: 'proposal',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [futureActivity],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
        activityContextReady: false,
      }),
    ).toBeNull();
  });

  it('builds dashboard attention with local-day semantics', () => {
    const summary = buildDashboardAttentionSummary([
      activity({ id: 'yesterday', subject: 'Past due', dueDate: '2026-07-07' }),
      activity({ id: 'today', subject: 'Today call', dueDate: '2026-07-08' }),
      activity({ id: 'tomorrow', subject: 'Tomorrow email', dueDate: '2026-07-09' }),
      activity({ id: 'done', subject: 'Done', dueDate: '2026-07-07', status: 'completed' }),
    ], NOW);

    expect(summary).toMatchObject({
      overdueCount: 1,
      todayCount: 1,
      totalCount: 2,
    });
    expect(summary.items.map((item) => [item.id, item.bucket])).toEqual([
      ['yesterday', 'overdue'],
      ['today', 'today'],
    ]);
  });
});
