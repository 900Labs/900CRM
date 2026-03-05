/**
 * src/lib/api/dashboard.ts — Tauri IPC wrappers for dashboard stats.
 */

import { invoke } from '@tauri-apps/api/core';

export interface DashboardStats {
  totalContacts: number;
  activeDeals: number;
  pipelineValue: number;
  currency: string;
  upcomingTasks: number;
  overdueActivities: number;
}

interface BackendDashboardStats {
  total_contacts: number;
  total_organizations: number;
  active_deals: number;
  pipeline_value: number;
  upcoming_activities: number;
  overdue_activities: number;
}

interface BackendSetting {
  key: string;
  value: string;
  updated_at: string;
}

export async function getDashboardStats(): Promise<DashboardStats> {
  const [stats, currencySetting] = await Promise.all([
    invoke<BackendDashboardStats>('get_dashboard_stats'),
    invoke<BackendSetting | null>('get_setting', { key: 'currency' }),
  ]);

  return {
    totalContacts: (stats.total_contacts ?? 0) + (stats.total_organizations ?? 0),
    activeDeals: stats.active_deals ?? 0,
    pipelineValue: stats.pipeline_value ?? 0,
    currency: currencySetting?.value || 'USD',
    upcomingTasks: stats.upcoming_activities ?? 0,
    overdueActivities: stats.overdue_activities ?? 0,
  };
}
