/**
 * src/lib/api/activities.ts — Tauri IPC wrappers for activity commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ActivityType = 'task' | 'call' | 'meeting' | 'email';
export type ActivityStatus = 'pending' | 'completed' | 'overdue';

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

export type CreateActivityPayload = Omit<
  Activity,
  'id' | 'status' | 'completedAt' | 'contactName' | 'dealName' | 'createdAt' | 'updatedAt'
>;

export type UpdateActivityPayload = Partial<CreateActivityPayload>;

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

interface BackendActivity {
  id: string;
  activity_type: string;
  title: string;
  description: string;
  due_date: string | null;
  completed: boolean;
  contact_id: string | null;
  deal_id: string | null;
  created_at: string;
  updated_at: string;
}

function toActivityType(value: string): ActivityType | null {
  if (value === 'task' || value === 'call' || value === 'meeting' || value === 'email') {
    return value;
  }

  if (import.meta.env.DEV && typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent('activities-type-fallback', {
        detail: `Unknown backend activity type "${value}" is not supported`,
      })
    );
  }

  return null;
}

function toActivityStatus(completed: boolean, dueDate: string | null): ActivityStatus {
  if (completed) {
    return 'completed';
  }

  if (dueDate) {
    const dueTs = Date.parse(dueDate);
    if (!Number.isNaN(dueTs) && dueTs < Date.now()) {
      return 'overdue';
    }
  }

  return 'pending';
}

function mapActivity(activity: BackendActivity): Activity | null {
  const type = toActivityType(activity.activity_type);
  if (!type) {
    return null;
  }

  const status = toActivityStatus(activity.completed, activity.due_date);

  return {
    id: activity.id,
    type,
    subject: activity.title,
    notes: activity.description?.trim() ? activity.description : null,
    dueDate: activity.due_date,
    completedAt: activity.completed ? activity.updated_at : null,
    status,
    contactId: activity.contact_id,
    contactName: null,
    dealId: activity.deal_id,
    dealName: null,
    createdAt: activity.created_at,
    updatedAt: activity.updated_at,
  };
}

function requireMappedActivity(activity: BackendActivity, command: string): Activity {
  const mapped = mapActivity(activity);
  if (mapped) {
    return mapped;
  }
  throw new Error(`Unsupported activity type returned by ${command}: "${activity.activity_type}"`);
}

function sortActivities(items: Activity[], params: ListActivitiesParams): Activity[] {
  const sorted = [...items];
  const direction = params.sortDir === 'desc' ? -1 : 1;

  if (!params.sortBy) {
    return sorted;
  }

  sorted.sort((a, b) => {
    switch (params.sortBy) {
      case 'createdAt':
        return ((Date.parse(a.createdAt) || 0) - (Date.parse(b.createdAt) || 0)) * direction;
      case 'subject':
        return a.subject.localeCompare(b.subject) * direction;
      case 'dueDate':
      default:
        return ((Date.parse(a.dueDate ?? '') || 0) - (Date.parse(b.dueDate ?? '') || 0)) * direction;
    }
  });

  return sorted;
}

export async function createActivity(data: CreateActivityPayload): Promise<Activity> {
  const activity = await invoke<BackendActivity>('create_activity', {
    activity_type: data.type,
    title: data.subject,
    description: data.notes ?? '',
    due_date: data.dueDate,
    contact_id: data.contactId,
    deal_id: data.dealId,
  });

  return requireMappedActivity(activity, 'create_activity');
}

export async function getActivity(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('get_activity', { id });
  return requireMappedActivity(activity, 'get_activity');
}

export async function listActivities(params: ListActivitiesParams = {}): Promise<Activity[]> {
  const activities = await invoke<BackendActivity[]>('list_activities');

  let mapped = activities.map((activity) => requireMappedActivity(activity, 'list_activities'));

  if (params.type) {
    mapped = mapped.filter((activity) => activity.type === params.type);
  }

  if (params.status) {
    mapped = mapped.filter((activity) => activity.status === params.status);
  }

  if (params.contactId) {
    mapped = mapped.filter((activity) => activity.contactId === params.contactId);
  }

  if (params.dealId) {
    mapped = mapped.filter((activity) => activity.dealId === params.dealId);
  }

  mapped = sortActivities(mapped, params);

  return mapped;
}

export async function listUpcoming(): Promise<Activity[]> {
  const activities = await invoke<BackendActivity[]>('list_upcoming_activities', {
    limit: 10,
  });

  return activities.map((activity) => requireMappedActivity(activity, 'list_upcoming_activities'));
}

export async function markComplete(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('mark_activity_complete', { id });
  return requireMappedActivity(activity, 'mark_activity_complete');
}

export async function markIncomplete(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('mark_activity_incomplete', { id });
  return requireMappedActivity(activity, 'mark_activity_incomplete');
}

export async function updateActivity(id: string, data: UpdateActivityPayload): Promise<Activity> {
  const activity = await invoke<BackendActivity>('update_activity', {
    id,
    activity_type: data.type,
    title: data.subject,
    description: data.notes,
    due_date: data.dueDate,
    contact_id: data.contactId,
    deal_id: data.dealId,
  });

  return requireMappedActivity(activity, 'update_activity');
}

export async function deleteActivity(id: string): Promise<void> {
  await invoke<void>('delete_activity', { id });
}
