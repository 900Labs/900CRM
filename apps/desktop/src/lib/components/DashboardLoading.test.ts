// @vitest-environment jsdom

import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const {
  getActivityFunnelReportMock,
  getDashboardStatsMock,
  getPipelineConversionReportMock,
  loadUpcomingMock,
  openModalMock,
} = vi.hoisted(() => ({
  getActivityFunnelReportMock: vi.fn(),
  getDashboardStatsMock: vi.fn(),
  getPipelineConversionReportMock: vi.fn(),
  loadUpcomingMock: vi.fn(),
  openModalMock: vi.fn(),
}));

vi.mock('$lib/i18n', () => ({
  t: (key: string) => key,
}));

vi.mock('$lib/api/dashboard', () => ({
  getDashboardStats: getDashboardStatsMock,
}));

vi.mock('$lib/api/reports', () => ({
  getActivityFunnelReport: getActivityFunnelReportMock,
  getPipelineConversionReport: getPipelineConversionReportMock,
}));

vi.mock('$lib/stores/activities', () => ({
  activityStore: {
    isLoading: false,
    loadUpcoming: loadUpcomingMock,
    upcoming: [],
  },
}));

vi.mock('$lib/stores/ui', () => ({
  uiStore: {
    openModal: openModalMock,
  },
}));

vi.mock('$lib/stores/settings', () => ({
  settingsStore: {
    currency: 'USD',
    language: 'en',
  },
}));

import Dashboard from '../../routes/Dashboard.svelte';

describe('Dashboard loading behavior', () => {
  beforeEach(() => {
    getActivityFunnelReportMock.mockReset();
    getDashboardStatsMock.mockReset();
    getPipelineConversionReportMock.mockReset();
    loadUpcomingMock.mockReset();
    openModalMock.mockReset();
  });

  it('renders KPI stats even when report calls are still pending', async () => {
    getDashboardStatsMock.mockResolvedValueOnce({
      activeDeals: 3,
      overdueActivities: 0,
      pipelineValue: 1200,
      pipelineValueByCurrency: [{ currency: 'USD', dealCount: 3, totalValue: 1200 }],
      totalContacts: 12,
      upcomingTasks: 4,
    });
    loadUpcomingMock.mockReturnValue(new Promise(() => {}));
    getActivityFunnelReportMock.mockReturnValue(new Promise(() => {}));
    getPipelineConversionReportMock.mockReturnValue(new Promise(() => {}));

    render(Dashboard);

    expect(await screen.findByText('12')).toBeTruthy();
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText('$1,200')).toBeTruthy();
    expect(screen.getByText('4')).toBeTruthy();
    expect(screen.getByText('dashboard.reports.pipelineTitle')).toBeTruthy();
    expect(screen.getByText('dashboard.reports.activityTitle')).toBeTruthy();
  });
});
