import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

type InvokeArgs = Record<string, unknown> | undefined;

declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke: (cmd: string, args?: InvokeArgs) => Promise<unknown>;
      transformCallback: (callback: (...args: unknown[]) => unknown) => number;
      unregisterCallback: (_id: number) => void;
      convertFileSrc: (filePath: string) => string;
      metadata: {
        currentWindow: { label: string };
        currentWebview: { label: string };
      };
    };
  }
}

test.beforeEach(async ({ page }) => {
  const consoleErrors: string[] = [];

  page.on('console', (message) => {
    if (message.type() === 'error') {
      consoleErrors.push(message.text());
    }
  });
  page.on('pageerror', (error) => {
    consoleErrors.push(error.message);
  });

  await page.addInitScript(() => {
    const responses: Record<string, unknown> = {
      get_settings: {
        language: 'en',
        currency: 'USD',
        theme: 'system',
        date_format: 'MMM D, YYYY',
        sync_enabled: 'false',
        sync_url: '',
        notifications_enabled: 'false',
        reminder_lead_minutes: '30',
        email_integration_enabled: 'false',
        smtp_host: '',
        smtp_port: '587',
        smtp_username: '',
        smtp_password: '',
        smtp_from: '',
        imap_host: '',
        imap_port: '993',
        imap_username: '',
        imap_password: '',
      },
      get_dashboard_stats: {
        total_contacts: 0,
        total_organizations: 0,
        active_deals: 0,
        pipeline_value: 0,
        pipeline_value_by_currency: [],
        upcoming_activities: 0,
        overdue_activities: 0,
      },
      get_pipeline_conversion_report: {
        generated_at: '2026-06-24T00:00:00Z',
        total_deals: 0,
        open_deals: 0,
        closed_won: 0,
        closed_lost: 0,
        overall_win_rate: 0,
        stage_metrics: [],
        transition_metrics: [],
      },
      get_activity_funnel_report: {
        generated_at: '2026-06-24T00:00:00Z',
        total_activities: 0,
        completed_activities: 0,
        pending_activities: 0,
        overdue_activities: 0,
        completion_rate: 0,
        overdue_rate: 0,
        by_type: [],
        due_buckets: {
          overdue: 0,
          due_today: 0,
          due_next_7_days: 0,
          due_later: 0,
          no_due_date: 0,
        },
      },
      list_upcoming_activities: [],
      list_contacts: {
        contacts: [],
        total: 0,
        page: 1,
        per_page: 50,
      },
      list_contact_duplicate_candidates: [],
      list_custom_field_defs: [],
      list_organizations: [],
      list_deals: [],
      list_custom_field_values_for_type: [],
      list_activities: [],
      list_recent_audit_log: [],
      list_pending_proposed_actions: [],
      list_external_clients: [],
      'plugin:notification|is_permission_granted': false,
    };

    let callbackId = 0;

    window.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (Object.prototype.hasOwnProperty.call(responses, cmd)) {
          return responses[cmd];
        }
        throw new Error(`Unmocked Tauri invoke in browser smoke test: ${cmd}`);
      },
      transformCallback: () => {
        callbackId += 1;
        return callbackId;
      },
      unregisterCallback: () => {},
      convertFileSrc: (filePath: string) => filePath,
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { label: 'main' },
      },
    };
  });

  await page.exposeFunction('__assertNoConsoleErrors', () => {
    const unexpected = consoleErrors.filter(
      (entry) => !entry.includes('favicon.ico') && !entry.includes('[vite]'),
    );
    if (unexpected.length > 0) {
      throw new Error(`Unexpected browser console errors:\n${unexpected.join('\n')}`);
    }
  });
});

async function loadHashRoute(page: Page, route: string) {
  await page.goto(`/#${route}`);
  await page.reload();
  await expect(page).toHaveURL(new RegExp(`#${route}$`));
}

test('renders the browser app shell and dashboard route', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByText('900CRM')).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Main navigation' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByText('Total Contacts')).toBeVisible();
  await expect(page.getByText('Pipeline Conversion')).toBeVisible();

  await page.evaluate(() => window.__assertNoConsoleErrors());
});

test('renders key hash routes without native Tauri dialogs', async ({ page }) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();
  await expect(page.getByText('No contacts yet')).toBeVisible();

  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();
  await expect(page.getByText('Lead').first()).toBeVisible();

  await loadHashRoute(page, '/settings');
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toBeVisible();

  await page.evaluate(() => window.__assertNoConsoleErrors());
});
