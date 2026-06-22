import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createActivity,
  deleteActivity,
  getActivity,
  listActivities,
  listUpcoming,
  markComplete,
  markIncomplete,
  updateActivity,
  type CreateActivityPayload,
} from './activities';

type BackendActivity = {
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
};

function sampleActivity(overrides: Partial<BackendActivity> = {}): BackendActivity {
  return {
    id: 'activity-1',
    activity_type: 'task',
    title: 'Follow up',
    description: 'Call customer',
    due_date: '2026-03-01T09:00:00.000Z',
    completed: false,
    contact_id: 'contact-1',
    deal_id: 'deal-1',
    created_at: '2026-02-25T10:00:00.000Z',
    updated_at: '2026-02-25T10:00:00.000Z',
    ...overrides,
  };
}

describe('activities api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    vi.useRealTimers();
  });

  it('maps createActivity payload and result fields', async () => {
    const payload: CreateActivityPayload = {
      type: 'call',
      subject: 'Demo call',
      notes: 'Bring proposal',
      dueDate: '2026-03-05T12:00:00.000Z',
      contactId: 'contact-5',
      dealId: 'deal-8',
    };
    invokeMock.mockResolvedValue(
      sampleActivity({
        activity_type: 'call',
        title: payload.subject,
        description: payload.notes ?? '',
        due_date: payload.dueDate,
        contact_id: payload.contactId,
        deal_id: payload.dealId,
      })
    );

    const activity = await createActivity(payload);

    expect(invokeMock).toHaveBeenCalledWith('create_activity', {
      activity_type: 'call',
      title: 'Demo call',
      description: 'Bring proposal',
      due_date: '2026-03-05T12:00:00.000Z',
      contact_id: 'contact-5',
      deal_id: 'deal-8',
    });
    expect(activity.type).toBe('call');
    expect(activity.subject).toBe('Demo call');
    expect(activity.notes).toBe('Bring proposal');
    expect(activity.status).toBe('overdue');
  });

  it('throws when backend returns an unknown activity type', async () => {
    invokeMock.mockResolvedValue(
      sampleActivity({
        activity_type: 'unknown_type',
        description: '   ',
      })
    );

    await expect(getActivity('activity-1')).rejects.toThrow('Unsupported activity type');
    expect(invokeMock).toHaveBeenCalledWith('get_activity', { id: 'activity-1' });
  });

  it('filters and sorts listActivities by status, type, and dueDate', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-03-05T12:00:00.000Z'));

    invokeMock.mockResolvedValue([
      sampleActivity({
        id: '1',
        activity_type: 'task',
        title: 'B task',
        due_date: '2026-03-03T00:00:00.000Z',
        completed: false,
      }),
      sampleActivity({
        id: '2',
        activity_type: 'task',
        title: 'A task',
        due_date: '2026-03-07T00:00:00.000Z',
        completed: false,
      }),
      sampleActivity({
        id: '3',
        activity_type: 'call',
        title: 'Call done',
        completed: true,
      }),
    ]);

    const pendingTasks = await listActivities({
      status: 'pending',
      type: 'task',
      sortBy: 'dueDate',
      sortDir: 'asc',
    });

    expect(invokeMock).toHaveBeenCalledWith('list_activities');
    expect(pendingTasks.map((activity) => activity.id)).toEqual(['2']);
    expect(pendingTasks[0]?.status).toBe('pending');
  });

  it('returns overdue activities when due date is in the past', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-03-05T12:00:00.000Z'));

    invokeMock.mockResolvedValue([
      sampleActivity({
        id: 'old',
        due_date: '2026-03-01T00:00:00.000Z',
        completed: false,
      }),
      sampleActivity({
        id: 'future',
        due_date: '2026-03-08T00:00:00.000Z',
        completed: false,
      }),
    ]);

    const overdue = await listActivities({ status: 'overdue' });
    expect(overdue.map((activity) => activity.id)).toEqual(['old']);
  });

  it('throws when list returns an unknown activity type', async () => {
    invokeMock.mockResolvedValue([
      sampleActivity({ activity_type: 'task' }),
      sampleActivity({ id: 'bad', activity_type: 'unsupported' }),
    ]);

    await expect(listActivities()).rejects.toThrow('Unsupported activity type');
  });

  it('requests fixed upcoming limit from backend', async () => {
    invokeMock.mockResolvedValue([sampleActivity()]);

    const activities = await listUpcoming();

    expect(invokeMock).toHaveBeenCalledWith('list_upcoming_activities', { limit: 10 });
    expect(activities).toHaveLength(1);
  });

  it('uses correct IPC commands for completion and updates', async () => {
    invokeMock.mockResolvedValue(sampleActivity({ completed: true }));

    await markComplete('activity-9');
    expect(invokeMock).toHaveBeenLastCalledWith('mark_activity_complete', { id: 'activity-9' });

    await markIncomplete('activity-9');
    expect(invokeMock).toHaveBeenLastCalledWith('mark_activity_incomplete', { id: 'activity-9' });

    await updateActivity('activity-9', {
      subject: 'Changed',
      notes: 'Updated note',
      type: 'email',
    });
    expect(invokeMock).toHaveBeenLastCalledWith('update_activity', {
      id: 'activity-9',
      activity_type: 'email',
      title: 'Changed',
      description: 'Updated note',
      due_date: undefined,
      contact_id: undefined,
      deal_id: undefined,
    });
  });

  it('uses delete_activity IPC command', async () => {
    invokeMock.mockResolvedValue(undefined);

    await deleteActivity('activity-44');

    expect(invokeMock).toHaveBeenCalledWith('delete_activity', { id: 'activity-44' });
  });
});
