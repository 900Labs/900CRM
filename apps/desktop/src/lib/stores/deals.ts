/**
 * src/lib/stores/deals.ts — Deal/pipeline state management for 900CRM.
 *
 * @module stores/deals
 */

import {
  listDeals,
  createDeal,
  updateDeal,
  moveDealStage,
  deleteDeal,
  getPipelineSummary,
} from '$lib/api/deals';
import type {
  Deal,
  CreateDealPayload,
  UpdateDealPayload,
  DealsByStage,
  DealStage,
  PipelineSummary,
  ListDealsParams,
} from '$lib/api/deals';
import { runLoadingAction, runSavingAction, runStoreAction } from './actionRunner';
import { uiStore } from './ui';

const notifier = {
  success: (message: string) => uiStore.toastSuccess(message),
  error: (message: string) => uiStore.toastError(message),
};

function emptyDealsByStage(): DealsByStage {
  return {
    lead: [],
    qualified: [],
    proposal: [],
    negotiation: [],
    closedWon: [],
    closedLost: [],
  };
}

function groupDealsByStage(deals: Deal[]): DealsByStage {
  const grouped = emptyDealsByStage();
  for (const deal of deals) {
    grouped[deal.stage].push(deal);
  }
  return grouped;
}

class DealStore {
  /** Flat list of all deals (for list views). */
  deals = $state<Deal[]>([]);

  /** Deals grouped by stage (for Kanban). */
  dealsByStage = $derived<DealsByStage>(groupDealsByStage(this.deals));

  /** Currently selected deal. */
  selectedDeal = $state<Deal | null>(null);

  /** Pipeline summary stats. */
  summary = $state<PipelineSummary | null>(null);

  /** Whether the pipeline is loading. */
  isLoading = $state<boolean>(false);

  /** Whether a save/move is in progress. */
  isSaving = $state<boolean>(false);

  async loadDeals(params: ListDealsParams = {}): Promise<void> {
    await runLoadingAction({
      setLoading: (value) => {
        this.isLoading = value;
      },
      notifier,
      errorMessage: 'Failed to load deals',
      action: async () => {
        this.deals = await listDeals(params);
      },
    });
  }

  async loadPipelineBoard(): Promise<void> {
    await this.loadDeals();
  }

  async loadPipelineSummary(): Promise<void> {
    await runStoreAction({
      notifier,
      errorMessage: 'Failed to load pipeline summary',
      action: async () => {
        this.summary = await getPipelineSummary();
      },
      onError: () => {
        this.summary = null;
      },
    });
  }

  async createDeal(data: CreateDealPayload): Promise<Deal> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Deal created',
      errorMessage: 'Failed to create deal',
      action: () => createDeal(data),
      onSuccess: (deal) => {
        this.deals = [...this.deals, deal];
      },
    });
  }

  async updateDeal(id: string, data: UpdateDealPayload): Promise<Deal> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Deal updated',
      errorMessage: 'Failed to update deal',
      action: () => updateDeal(id, data),
      onSuccess: (deal) => {
        this.deals = this.deals.map((d) => (d.id === id ? deal : d));

        if (this.selectedDeal?.id === id) {
          this.selectedDeal = deal;
        }
      },
    });
  }

  async moveDealStage(id: string, toStage: DealStage): Promise<void> {
    const deal = this.deals.find((d) => d.id === id);
    if (!deal || deal.stage === toStage) return;

    const previous = deal;
    const updatedDeal = { ...deal, stage: toStage };
    this.deals = this.deals.map((d) => (d.id === id ? updatedDeal : d));

    await runStoreAction({
      notifier,
      errorMessage: 'Failed to move deal',
      action: () => moveDealStage(id, toStage),
      onError: () => {
        this.deals = this.deals.map((d) => (d.id === id ? previous : d));
      },
    });
  }

  async deleteDeal(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      successMessage: 'Deal deleted',
      errorMessage: 'Failed to delete deal',
      action: () => deleteDeal(id),
      onSuccess: () => {
        this.deals = this.deals.filter((d) => d.id !== id);

        if (this.selectedDeal?.id === id) {
          this.selectedDeal = null;
        }
      },
    });
  }

  selectDeal(deal: Deal | null): void {
    this.selectedDeal = deal;
  }
}

export const dealStore = new DealStore();
