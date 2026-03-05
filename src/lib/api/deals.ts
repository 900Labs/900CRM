/**
 * src/lib/api/deals.ts — Tauri IPC wrappers for the deals/pipeline backend.
 *
 * @module api/deals
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Pipeline stage identifier. */
export type DealStage =
  | 'lead'
  | 'qualified'
  | 'proposal'
  | 'negotiation'
  | 'closedWon'
  | 'closedLost';

/** All pipeline stages in order. */
export const DEAL_STAGES: DealStage[] = [
  'lead',
  'qualified',
  'proposal',
  'negotiation',
  'closedWon',
  'closedLost',
];

/** A CRM deal record. */
export interface Deal {
  id: string;
  name: string;
  value: number;
  currency: string;
  stage: DealStage;
  probability: number;      // 0–100
  expectedCloseDate: string | null;
  contactId: string | null;
  contactName: string | null;
  description: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

/** Payload for creating a deal. */
export type CreateDealPayload = Omit<Deal, 'id' | 'createdAt' | 'updatedAt' | 'contactName'>;

/** Payload for updating a deal. */
export type UpdateDealPayload = Partial<CreateDealPayload>;

/** Parameters for listing deals. */
export interface ListDealsParams {
  stage?: DealStage;
  contactId?: string;
  sortBy?: 'name' | 'value' | 'createdAt' | 'expectedCloseDate';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

/** Deals grouped by stage for the Kanban view. */
export type DealsByStage = Record<DealStage, Deal[]>;

/** Summary stats for the pipeline. */
export interface PipelineSummary {
  totalDeals: number;
  totalValue: number;
  byStage: Record<DealStage, { count: number; value: number }>;
}

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Create a new deal.
 */
export async function createDeal(data: CreateDealPayload): Promise<Deal> {
  return invoke<Deal>('create_deal', { data });
}

/**
 * Fetch a single deal by ID.
 */
export async function getDeal(id: string): Promise<Deal> {
  return invoke<Deal>('get_deal', { id });
}

/**
 * List deals with optional filtering.
 */
export async function listDeals(params: ListDealsParams = {}): Promise<Deal[]> {
  return invoke<Deal[]>('list_deals', { params });
}

/**
 * List all deals grouped by pipeline stage (for Kanban).
 */
export async function listDealsByStage(): Promise<DealsByStage> {
  return invoke<DealsByStage>('list_deals_by_stage');
}

/**
 * Update a deal by ID.
 */
export async function updateDeal(id: string, data: UpdateDealPayload): Promise<Deal> {
  return invoke<Deal>('update_deal', { id, data });
}

/**
 * Move a deal to a different pipeline stage.
 *
 * @param id     Deal UUID
 * @param stage  New stage
 */
export async function moveDealStage(id: string, stage: DealStage): Promise<Deal> {
  return invoke<Deal>('move_deal_stage', { id, stage });
}

/**
 * Delete a deal by ID.
 */
export async function deleteDeal(id: string): Promise<void> {
  return invoke<void>('delete_deal', { id });
}

/**
 * Get pipeline summary statistics.
 */
export async function getPipelineSummary(): Promise<PipelineSummary> {
  return invoke<PipelineSummary>('get_pipeline_summary');
}
