import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { getDashboardStats } from './dashboard';

describe('dashboard api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps stats and currency setting with defaults', async () => {
    invokeMock
      .mockResolvedValueOnce({
        total_contacts: 10,
        total_organizations: 4,
        active_deals: 3,
        pipeline_value: 9000,
        upcoming_activities: 5,
        overdue_activities: 2,
      })
      .mockResolvedValueOnce({ key: 'currency', value: 'EUR', updated_at: '2026-03-05T00:00:00Z' });

    const stats = await getDashboardStats();

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'get_dashboard_stats');
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'get_setting', { key: 'currency' });
    expect(stats).toEqual({
      totalContacts: 14,
      activeDeals: 3,
      pipelineValue: 9000,
      currency: 'EUR',
      upcomingTasks: 5,
      overdueActivities: 2,
    });
  });

  it('falls back when currency setting is missing', async () => {
    invokeMock
      .mockResolvedValueOnce({
        total_contacts: 0,
        total_organizations: 0,
        active_deals: 0,
        pipeline_value: 0,
        upcoming_activities: 0,
        overdue_activities: 0,
      })
      .mockResolvedValueOnce(null);

    const stats = await getDashboardStats();
    expect(stats.currency).toBe('USD');
  });
});
