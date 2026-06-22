import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createDeal,
  deleteDeal,
  getPipelineSummary,
  listDeals,
  listDealsByStage,
  moveDealStage,
  updateDeal,
  type CreateDealPayload,
} from './deals';

type BackendDeal = {
  id: string;
  title: string;
  value: number;
  currency: string;
  stage: string;
  probability: number;
  expected_close: string | null;
  contact_id: string | null;
  notes: string;
  created_at: string;
  updated_at: string;
};

function sampleDeal(overrides: Partial<BackendDeal> = {}): BackendDeal {
  return {
    id: 'deal-1',
    title: 'ACME Expansion',
    value: 5000,
    currency: 'USD',
    stage: 'Lead',
    probability: 20,
    expected_close: '2026-05-01T00:00:00.000Z',
    contact_id: 'contact-1',
    notes: 'High potential',
    created_at: '2026-03-01T00:00:00.000Z',
    updated_at: '2026-03-02T00:00:00.000Z',
    ...overrides,
  };
}

describe('deals api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createDeal payload to backend command', async () => {
    const payload: CreateDealPayload = {
      name: 'Renewal',
      value: 900,
      currency: 'EUR',
      stage: 'qualified',
      probability: 55,
      expectedCloseDate: '2026-06-01T00:00:00.000Z',
      contactId: 'contact-7',
      description: 'Urgent',
      tags: [],
    };
    invokeMock.mockResolvedValue(
      sampleDeal({
        title: payload.name,
        stage: 'Qualified',
        value: payload.value,
        currency: payload.currency,
        probability: payload.probability,
        expected_close: payload.expectedCloseDate,
        contact_id: payload.contactId,
        notes: payload.description ?? '',
      })
    );

    const deal = await createDeal(payload);

    expect(invokeMock).toHaveBeenCalledWith('create_deal', {
      title: 'Renewal',
      value: 900,
      currency: 'EUR',
      stage: 'Qualified',
      probability: 55,
      expected_close: '2026-06-01T00:00:00.000Z',
      contact_id: 'contact-7',
      notes: 'Urgent',
    });
    expect(deal.stage).toBe('qualified');
  });

  it('throws when listDeals receives unknown backend stages', async () => {
    invokeMock.mockResolvedValue([
      sampleDeal({ id: '1', title: 'B', stage: 'Negotiation', value: 500 }),
      sampleDeal({ id: '2', title: 'A', stage: 'Unknown Stage', value: 100 }),
      sampleDeal({ id: '3', title: 'C', stage: 'Lead', value: 800, contact_id: 'contact-9' }),
    ]);

    await expect(
      listDeals({
        stage: 'lead',
        contactId: 'contact-1',
        sortBy: 'name',
        sortDir: 'asc',
      })
    ).rejects.toThrow('Unsupported deal stage');

    expect(invokeMock).toHaveBeenCalledWith('list_deals');
  });

  it('groups deals by stage', async () => {
    invokeMock.mockResolvedValue([
      sampleDeal({ id: '1', stage: 'Lead' }),
      sampleDeal({ id: '2', stage: 'Closed Won' }),
    ]);

    const grouped = await listDealsByStage();

    expect(grouped.lead).toHaveLength(1);
    expect(grouped.closedWon).toHaveLength(1);
  });

  it('uses update/move/delete commands', async () => {
    invokeMock.mockResolvedValue(sampleDeal({ stage: 'Closed Lost' }));

    await updateDeal('deal-1', { name: 'Renamed', stage: 'closedLost' });
    expect(invokeMock).toHaveBeenCalledWith('update_deal', {
      id: 'deal-1',
      title: 'Renamed',
      value: undefined,
      currency: undefined,
      stage: 'Closed Lost',
      probability: undefined,
      expected_close: undefined,
      contact_id: undefined,
      notes: undefined,
    });

    await moveDealStage('deal-1', 'proposal');
    expect(invokeMock).toHaveBeenCalledWith('move_deal_stage', {
      id: 'deal-1',
      stage: 'Proposal',
    });

    await deleteDeal('deal-1');
    expect(invokeMock).toHaveBeenCalledWith('delete_deal', { id: 'deal-1' });
  });

  it('maps pipeline summary totals and stages', async () => {
    invokeMock.mockResolvedValue([
      { stage: 'Lead', count: 2, total_value: 1000, weighted_value: 0 },
      { stage: 'Closed Won', count: 1, total_value: 3000, weighted_value: 0 },
    ]);

    const summary = await getPipelineSummary();

    expect(invokeMock).toHaveBeenCalledWith('get_pipeline_summary');
    expect(summary.totalDeals).toBe(3);
    expect(summary.totalValue).toBe(4000);
    expect(summary.byStage.lead.count).toBe(2);
    expect(summary.byStage.closedWon.value).toBe(3000);
  });
});
