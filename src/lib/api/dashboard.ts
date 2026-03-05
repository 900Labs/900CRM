/**
 * src/lib/api/dashboard.ts — Tauri IPC wrappers for dashboard statistics.
 *
 * @module api/dashboard
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Dashboard KPI statistics. */
export interface DashboardStats {
  totalContacts: number;
  activeDeals: number;
  pipelineValue: number;
  currency: string;
  upcomingTasks: number;
  overdueActivities: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Fetch dashboard KPI statistics.
 *
 * @returns DashboardStats
 */
export async function getDashboardStats(): Promise<DashboardStats> {
  return invoke<DashboardStats>('get_dashboard_stats');
}
