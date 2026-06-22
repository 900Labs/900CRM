/**
 * src/lib/stores/activities.ts — Activity state management for 900CRM.
 *
 * @module stores/activities
 */

import {
  listActivities,
  listUpcoming,
  createActivity,
  updateActivity,
  markComplete,
  markIncomplete,
  deleteActivity,
} from '$lib/api/activities';
import type {
  Activity,
  CreateActivityPayload,
  UpdateActivityPayload,
  ListActivitiesParams,
} from '$lib/api/activities';
import { runLoadingAction, runSavingAction, runStoreAction } from './actionRunner';
import { uiStore } from './ui';

const notifier = {
  success: (message: string) => uiStore.toastSuccess(message),
  error: (message: string) => uiStore.toastError(message),
};

// ─────────────────────────────────────────────────────────────────────────────
// ActivityStore
// ─────────────────────────────────────────────────────────────────────────────

class ActivityStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** Current page of activities. */
  activities = $state<Activity[]>([]);

  /** Upcoming (pending, not overdue) activities. */
  upcoming = $state<Activity[]>([]);

  /** Overdue activities (for dashboard warning). */
  overdue = $derived(
    this.activities.filter((a) => a.status === 'overdue')
  );

  /** Active filters. */
  filters = $state<ListActivitiesParams>({
    sortBy: 'dueDate',
    sortDir: 'asc',
  });

  /** Whether the list is loading. */
  isLoading = $state<boolean>(false);

  /** Whether a save is in progress. */
  isSaving = $state<boolean>(false);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load activities with the current filter state.
   */
  async loadActivities(): Promise<void> {
    await runLoadingAction({
      setLoading: (value) => {
        this.isLoading = value;
      },
      notifier,
      errorMessage: 'Failed to load activities',
      action: async () => {
        this.activities = await listActivities(this.filters);
      },
    });
  }

  /**
   * Load upcoming activities (for dashboard).
   */
  async loadUpcoming(): Promise<void> {
    await runStoreAction({
      notifier,
      errorMessage: 'Failed to load upcoming activities',
      action: async () => {
        this.upcoming = await listUpcoming();
      },
      onError: () => {
        this.upcoming = [];
      },
    });
  }

  /**
   * Create a new activity.
   *
   * @param data  Activity creation payload
   * @returns     The created Activity
   */
  async createActivity(data: CreateActivityPayload): Promise<Activity> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Activity created',
      errorMessage: 'Failed to create activity',
      action: () => createActivity(data),
      onSuccess: (activity) => {
        this.activities = [activity, ...this.activities];
      },
    });
  }

  /**
   * Mark an activity as complete.
   *
   * @param id  Activity UUID
   */
  async markComplete(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      errorMessage: 'Failed to mark activity complete',
      action: () => markComplete(id),
      onSuccess: (updated) => {
        this.activities = this.activities.map((a) => (a.id === id ? updated : a));
        this.upcoming = this.upcoming.filter((a) => a.id !== id);
      },
    });
  }

  /**
   * Mark an activity as incomplete.
   *
   * @param id  Activity UUID
   */
  async markIncomplete(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      errorMessage: 'Failed to mark activity incomplete',
      action: () => markIncomplete(id),
      onSuccess: (updated) => {
        this.activities = this.activities.map((a) => (a.id === id ? updated : a));
      },
    });
  }

  /**
   * Update an activity.
   *
   * @param id    Activity UUID
   * @param data  Fields to update
   */
  async updateActivity(id: string, data: UpdateActivityPayload): Promise<Activity> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Activity updated',
      errorMessage: 'Failed to update activity',
      action: () => updateActivity(id, data),
      onSuccess: (activity) => {
        this.activities = this.activities.map((a) => (a.id === id ? activity : a));
      },
    });
  }

  /**
   * Delete an activity.
   *
   * @param id  Activity UUID
   */
  async deleteActivity(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      successMessage: 'Activity deleted',
      errorMessage: 'Failed to delete activity',
      action: () => deleteActivity(id),
      onSuccess: () => {
        this.activities = this.activities.filter((a) => a.id !== id);
        this.upcoming = this.upcoming.filter((a) => a.id !== id);
      },
    });
  }

  /**
   * Update filter state and reload.
   *
   * @param updates  Partial filter changes
   */
  async setFilters(updates: Partial<ListActivitiesParams>): Promise<void> {
    this.filters = { ...this.filters, ...updates };
    await this.loadActivities();
  }
}

/** Singleton activities store. */
export const activityStore = new ActivityStore();
