/**
 * src/lib/api/deals.ts — Tauri IPC wrappers for deals/pipeline.
 */

import { invoke } from '@tauri-apps/api/core';

export type DealStage =
  | 'lead'
  | 'qualified'
  | 'proposal'
  | 'negotiation'
  | 'closedWon'
  | 'closedLost';

export const DEAL_STAGES: DealStage[] = [
  'lead',
  'qualified',
  'proposal',
  'negotiation',
  'closedWon',
  'closedLost',
];

export interface Deal {
  id: string;
  name: string;
  value: number;
  currency: string;
  stage: DealStage;
  probability: number;
  expectedCloseDate: string | null;
  contactId: string | null;
  contactName: string | null;
  description: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

export type CreateDealPayload = Omit<Deal, 'id' | 'createdAt' | 'updatedAt' | 'contactName'>;
export type UpdateDealPayload = Partial<CreateDealPayload>;

export interface ListDealsParams {
  stage?: DealStage;
  contactId?: string;
  sortBy?: 'name' | 'value' | 'createdAt' | 'expectedCloseDate';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

export type DealsByStage = Record<DealStage, Deal[]>;

export interface PipelineSummary {
  totalDeals: number;
  totalValue: number;
  byStage: Record<DealStage, { count: number; value: number }>;
}

interface BackendDeal {
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
}

interface BackendPipelineSummary {
  stage: string;
  count: number;
  total_value: number;
  weighted_value: number;
}

const BACKEND_STAGE_MAP: Record<DealStage, string> = {
  lead: 'Lead',
  qualified: 'Qualified',
  proposal: 'Proposal',
  negotiation: 'Negotiation',
  closedWon: 'Closed Won',
  closedLost: 'Closed Lost',
};

const UI_STAGE_MAP: Record<string, DealStage> = {
  lead: 'lead',
  qualified: 'qualified',
  proposal: 'proposal',
  negotiation: 'negotiation',
  'closed won': 'closedWon',
  'closedwon': 'closedWon',
  'closed lost': 'closedLost',
  'closedlost': 'closedLost',
};

function toUiStage(stage: string): DealStage {
  const key = stage.toLowerCase().trim();
  return UI_STAGE_MAP[key] ?? 'lead';
}

function toBackendStage(stage: DealStage | undefined): string | undefined {
  if (!stage) {
    return undefined;
  }
  return BACKEND_STAGE_MAP[stage];
}

function mapDeal(deal: BackendDeal): Deal {
  return {
    id: deal.id,
    name: deal.title,
    value: Number.isFinite(deal.value) ? deal.value : 0,
    currency: deal.currency || 'USD',
    stage: toUiStage(deal.stage),
    probability: deal.probability ?? 0,
    expectedCloseDate: deal.expected_close,
    contactId: deal.contact_id,
    contactName: null,
    description: deal.notes?.trim() ? deal.notes : null,
    tags: [],
    createdAt: deal.created_at,
    updatedAt: deal.updated_at,
  };
}

function sortDeals(items: Deal[], params: ListDealsParams): Deal[] {
  const sorted = [...items];
  const direction = params.sortDir === 'desc' ? -1 : 1;

  if (!params.sortBy) {
    return sorted;
  }

  sorted.sort((a, b) => {
    switch (params.sortBy) {
      case 'value':
        return (a.value - b.value) * direction;
      case 'createdAt':
        return ((Date.parse(a.createdAt) || 0) - (Date.parse(b.createdAt) || 0)) * direction;
      case 'expectedCloseDate':
        return ((Date.parse(a.expectedCloseDate ?? '') || 0) - (Date.parse(b.expectedCloseDate ?? '') || 0)) * direction;
      case 'name':
      default:
        return a.name.localeCompare(b.name) * direction;
    }
  });

  return sorted;
}

export async function createDeal(data: CreateDealPayload): Promise<Deal> {
  const deal = await invoke<BackendDeal>('create_deal', {
    title: data.name,
    value: data.value,
    currency: data.currency,
    stage: toBackendStage(data.stage),
    probability: data.probability,
    expected_close: data.expectedCloseDate,
    contact_id: data.contactId,
    notes: data.description ?? '',
  });

  return mapDeal(deal);
}

export async function getDeal(id: string): Promise<Deal> {
  const deal = await invoke<BackendDeal>('get_deal', { id });
  return mapDeal(deal);
}

export async function listDeals(params: ListDealsParams = {}): Promise<Deal[]> {
  const deals = await invoke<BackendDeal[]>('list_deals');
  let mapped = deals.map(mapDeal);

  if (params.stage) {
    mapped = mapped.filter((deal) => deal.stage === params.stage);
  }

  if (params.contactId) {
    mapped = mapped.filter((deal) => deal.contactId === params.contactId);
  }

  mapped = sortDeals(mapped, params);

  return mapped;
}

export async function listDealsByStage(): Promise<DealsByStage> {
  const allDeals = await listDeals();

  const grouped: DealsByStage = {
    lead: [],
    qualified: [],
    proposal: [],
    negotiation: [],
    closedWon: [],
    closedLost: [],
  };

  for (const deal of allDeals) {
    grouped[deal.stage].push(deal);
  }

  return grouped;
}

export async function updateDeal(id: string, data: UpdateDealPayload): Promise<Deal> {
  const deal = await invoke<BackendDeal>('update_deal', {
    id,
    title: data.name,
    value: data.value,
    currency: data.currency,
    stage: toBackendStage(data.stage),
    probability: data.probability,
    expected_close: data.expectedCloseDate,
    contact_id: data.contactId,
    notes: data.description,
  });

  return mapDeal(deal);
}

export async function moveDealStage(id: string, stage: DealStage): Promise<Deal> {
  const deal = await invoke<BackendDeal>('move_deal_stage', {
    id,
    stage: toBackendStage(stage),
  });

  return mapDeal(deal);
}

export async function deleteDeal(id: string): Promise<void> {
  await invoke<void>('delete_deal', { id });
}

export async function getPipelineSummary(): Promise<PipelineSummary> {
  const summary = await invoke<BackendPipelineSummary[]>('get_pipeline_summary');

  const byStage: PipelineSummary['byStage'] = {
    lead: { count: 0, value: 0 },
    qualified: { count: 0, value: 0 },
    proposal: { count: 0, value: 0 },
    negotiation: { count: 0, value: 0 },
    closedWon: { count: 0, value: 0 },
    closedLost: { count: 0, value: 0 },
  };

  for (const row of summary) {
    const stage = toUiStage(row.stage);
    byStage[stage] = {
      count: row.count ?? 0,
      value: row.total_value ?? 0,
    };
  }

  const totalDeals = Object.values(byStage).reduce((sum, item) => sum + item.count, 0);
  const totalValue = Object.values(byStage).reduce((sum, item) => sum + item.value, 0);

  return {
    totalDeals,
    totalValue,
    byStage,
  };
}
