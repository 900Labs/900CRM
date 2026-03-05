/**
 * src/lib/api/activities.ts — Tauri IPC wrappers for the activities backend.
 *
 * @module api/activities
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Activity type. */
export type ActivityType = 'task' | 'call' | 'meeting' | 'email';

/** Activity status. */
export type ActivityStatus = 'pending' | 'completed' | 'overdue';

/** A CRM activity record. */
export interface Activity {
  id: string;
  type: ActivityType;
  subject: string;
  notes: string | null;
  dueDate: string | null;
  completedAt: string | null;
  status: ActivityStatus;
  contactId: string | null;
  contactName: string | null;
  dealId: string | null;
  dealName: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Payload for creating an activity. */
export type CreateActivityPayload = Omit<
  Activity,
  'id' | 'status' | 'completedAt' | 'contactName' | 'dealName' | 'createdAt' | 'updatedAt'
>;

/** Payload for updating an activity. */
export type UpdateActivityPayload = Partial<CreateActivityPayload>;

/** Parameters for listing activities. */
export interface ListActivitiesParams {
  type?: ActivityType;
  status?: ActivityStatus;
  contactId?: string;
  dealId?: string;
  sortBy?: 'dueDate' | 'createdAt' | 'subject';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Create a new activity.
 */
export async function createActivity(data: CreateActivityPayload): Promise<Activity> {
  return invoke<Activity>('create_activity', { data });
}

/**
 * Fetch a single activity by ID.
 */
export async function getActivity(id: string): Promise<Activity> {
  return invoke<Activity>('get_activity', { id });
}

/**
 * List activities with optional filtering.
 */
export async function listActivities(params: ListActivitiesParams = {}): Promise<Activity[]> {
  return invoke<Activity[]>('list_activities', { params });
}

/**
 * List upcoming (pending, not overdue) activities sorted by due date.
 */
export async function listUpcoming(): Promise<Activity[]> {
  return invoke<Activity[]>('list_upcoming_activities');
}

/**
 * Mark an activity as complete.
 *
 * @param id  Activity UUID
 */
export async function markComplete(id: string): Promise<Activity> {
  return invoke<Activity>('mark_activity_complete', { id });
}

/**
 * Mark an activity as incomplete (revert completion).
 *
 * @param id  Activity UUID
 */
export async function markIncomplete(id: string): Promise<Activity> {
  return invoke<Activity>('mark_activity_incomplete', { id });
}

/**
 * Update an activity by ID.
 */
export async function updateActivity(id: string, data: UpdateActivityPayload): Promise<Activity> {
  return invoke<Activity>('update_activity', { id, data });
}

/**
 * Delete an activity by ID.
 */
export async function deleteActivity(id: string): Promise<void> {
  return invoke<void>('delete_activity', { id });
}
