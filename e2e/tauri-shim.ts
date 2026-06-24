import { expect, test as base } from '@playwright/test';
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

type BrowserSmokeFixtures = {
  assertNoConsoleErrors: () => Promise<void>;
};

export const test = base.extend<BrowserSmokeFixtures>({
  assertNoConsoleErrors: async ({ page }, use) => {
    const consoleErrors = collectConsoleErrors(page);
    await installTauriShim(page);

    await use(async () => {
      const unexpected = consoleErrors.filter(
        (entry) => !entry.includes('favicon.ico') && !entry.includes('[vite]'),
      );
      if (unexpected.length > 0) {
        throw new Error(`Unexpected browser console errors:\n${unexpected.join('\n')}`);
      }
    });
  },
});

export { expect };

function collectConsoleErrors(page: Page): string[] {
  const consoleErrors: string[] = [];

  page.on('console', (message) => {
    if (message.type() === 'error') {
      consoleErrors.push(message.text());
    }
  });
  page.on('pageerror', (error) => {
    consoleErrors.push(error.message);
  });

  return consoleErrors;
}

async function installTauriShim(page: Page) {
  await page.addInitScript(() => {
    type BackendContact = {
      id: string;
      contact_type: string;
      first_name: string;
      last_name: string;
      org_name: string;
      email: string;
      phone: string;
      address: string;
      city: string;
      country: string;
      org_id: string | null;
      notes: string;
      created_at: string;
      updated_at: string;
      deleted_at: string | null;
    };

    type BackendOrganization = {
      id: string;
      name: string;
      email: string | null;
      phone: string | null;
      website: string | null;
      address_line1: string | null;
      address_line2: string | null;
      city: string | null;
      region: string | null;
      country: string | null;
      postal_code: string | null;
      source: string | null;
      description: string | null;
      created_at: string;
      updated_at: string;
      deleted_at: string | null;
      device_id: string;
    };

    type BackendDeal = {
      id: string;
      title: string;
      value: number;
      currency: string;
      stage: string;
      probability: number;
      expected_close: string | null;
      contact_id: string | null;
      organization_id: string | null;
      notes: string;
      created_at: string;
      updated_at: string;
    };

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

    type BackendActivityLink = {
      id: string;
      activity_id: string;
      entity_type: 'contact' | 'organization' | 'deal';
      entity_id: string;
      created_at: string;
      deleted_at: string | null;
      device_id: string;
    };

    type BackendGlobalSearchResult = {
      entity_type: 'contact' | 'organization' | 'deal' | 'activity' | 'note' | 'tag';
      entity_id: string;
      title: string;
      subtitle: string;
      match_field: string;
    };

    const timestamp = '2026-06-24T12:00:00.000Z';
    const state = {
      nextId: 1,
      contacts: [] as BackendContact[],
      organizations: [] as BackendOrganization[],
      deals: [] as BackendDeal[],
      activities: [] as BackendActivity[],
      activityLinks: [] as BackendActivityLink[],
    };

    function nextId(prefix: string): string {
      const id = `${prefix}-${state.nextId}`;
      state.nextId += 1;
      return id;
    }

    function stringArg(args: InvokeArgs, key: string, fallback = ''): string {
      const value = args?.[key];
      return typeof value === 'string' ? value : fallback;
    }

    function nullableStringArg(args: InvokeArgs, key: string): string | null {
      const value = stringArg(args, key).trim();
      return value.length > 0 ? value : null;
    }

    function numberArg(args: InvokeArgs, key: string, fallback = 0): number {
      const value = args?.[key];
      return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
    }

    function pageArgs(args: InvokeArgs): Record<string, unknown> {
      const params = args?.params;
      return params && typeof params === 'object' ? params as Record<string, unknown> : {};
    }

    function matchesText(fields: (string | null | undefined)[], query: string | undefined): boolean {
      const normalized = query?.trim().toLowerCase();
      if (!normalized) return true;
      return fields.some((field) => field?.toLowerCase().includes(normalized));
    }

    function listContacts(args: InvokeArgs) {
      const params = pageArgs(args);
      const page = typeof params.page === 'number' ? params.page : 1;
      const perPage = typeof params.per_page === 'number' ? params.per_page : 50;
      const filterType = typeof params.filter_type === 'string' ? params.filter_type : undefined;
      const searchQuery = typeof params.search_query === 'string' ? params.search_query : undefined;

      let contacts = state.contacts.filter((contact) => !contact.deleted_at);
      if (filterType) {
        contacts = contacts.filter((contact) => contact.contact_type === filterType);
      }
      contacts = contacts.filter((contact) =>
        matchesText(
          [
            contact.first_name,
            contact.last_name,
            contact.org_name,
            contact.email,
            contact.phone,
          ],
          searchQuery,
        )
      );
      contacts = [...contacts].sort((left, right) =>
        `${left.first_name} ${left.last_name}`.localeCompare(`${right.first_name} ${right.last_name}`),
      );

      const start = (page - 1) * perPage;
      return {
        contacts: contacts.slice(start, start + perPage),
        total: contacts.length,
        page,
        per_page: perPage,
      };
    }

    function createContact(args: InvokeArgs): BackendContact {
      const contact: BackendContact = {
        id: nextId('contact'),
        contact_type: stringArg(args, 'contact_type') === 'organization' ? 'organization' : 'person',
        first_name: stringArg(args, 'first_name'),
        last_name: stringArg(args, 'last_name'),
        org_name: stringArg(args, 'org_name'),
        email: stringArg(args, 'email'),
        phone: stringArg(args, 'phone'),
        address: stringArg(args, 'address'),
        city: stringArg(args, 'city'),
        country: stringArg(args, 'country'),
        org_id: null,
        notes: stringArg(args, 'notes'),
        created_at: timestamp,
        updated_at: timestamp,
        deleted_at: null,
      };
      state.contacts.push(contact);
      return contact;
    }

    function createOrganization(args: InvokeArgs): BackendOrganization {
      const organization: BackendOrganization = {
        id: nextId('organization'),
        name: stringArg(args, 'name'),
        email: nullableStringArg(args, 'email'),
        phone: nullableStringArg(args, 'phone'),
        website: nullableStringArg(args, 'website'),
        address_line1: nullableStringArg(args, 'address_line1'),
        address_line2: nullableStringArg(args, 'address_line2'),
        city: nullableStringArg(args, 'city'),
        region: nullableStringArg(args, 'region'),
        country: nullableStringArg(args, 'country'),
        postal_code: nullableStringArg(args, 'postal_code'),
        source: 'manual',
        description: nullableStringArg(args, 'description'),
        created_at: timestamp,
        updated_at: timestamp,
        deleted_at: null,
        device_id: 'browser-smoke-device',
      };
      state.organizations.push(organization);
      return organization;
    }

    function createDeal(args: InvokeArgs): BackendDeal {
      const deal: BackendDeal = {
        id: nextId('deal'),
        title: stringArg(args, 'title'),
        value: numberArg(args, 'value'),
        currency: stringArg(args, 'currency', 'USD'),
        stage: stringArg(args, 'stage', 'Lead'),
        probability: numberArg(args, 'probability', 10),
        expected_close: nullableStringArg(args, 'expected_close'),
        contact_id: nullableStringArg(args, 'contact_id'),
        organization_id: nullableStringArg(args, 'organization_id'),
        notes: stringArg(args, 'notes'),
        created_at: timestamp,
        updated_at: timestamp,
      };
      state.deals.push(deal);
      return deal;
    }

    function createActivity(args: InvokeArgs): BackendActivity {
      const activity: BackendActivity = {
        id: nextId('activity'),
        activity_type: stringArg(args, 'activity_type', 'task'),
        title: stringArg(args, 'title'),
        description: stringArg(args, 'description'),
        due_date: nullableStringArg(args, 'due_date'),
        completed: false,
        contact_id: nullableStringArg(args, 'contact_id'),
        deal_id: nullableStringArg(args, 'deal_id'),
        created_at: timestamp,
        updated_at: timestamp,
      };
      state.activities.unshift(activity);
      return activity;
    }

    function addActivityLink(args: InvokeArgs): BackendActivityLink {
      const link: BackendActivityLink = {
        id: nextId('activity-link'),
        activity_id: stringArg(args, 'activity_id'),
        entity_type: stringArg(args, 'entity_type') as BackendActivityLink['entity_type'],
        entity_id: stringArg(args, 'entity_id'),
        created_at: timestamp,
        deleted_at: null,
        device_id: 'browser-smoke-device',
      };
      state.activityLinks.push(link);
      return link;
    }

    function listActivityLinks(args: InvokeArgs): BackendActivityLink[] {
      const activityId = stringArg(args, 'activity_id');
      return state.activityLinks.filter((link) => link.activity_id === activityId && !link.deleted_at);
    }

    function globalSearch(args: InvokeArgs): BackendGlobalSearchResult[] {
      const query = stringArg(args, 'query').toLowerCase();
      const limit = numberArg(args, 'limit', 8);
      if (!query.trim()) {
        return [];
      }

      const results: BackendGlobalSearchResult[] = [];
      for (const contact of state.contacts) {
        if (matchesText([contact.first_name, contact.last_name, contact.email], query)) {
          results.push({
            entity_type: 'contact',
            entity_id: contact.id,
            title: [contact.first_name, contact.last_name].filter(Boolean).join(' ') || contact.email || contact.id,
            subtitle: contact.email || contact.org_name || 'Contact',
            match_field: 'contact',
          });
        }
      }
      for (const organization of state.organizations) {
        if (matchesText([organization.name, organization.email, organization.website], query)) {
          results.push({
            entity_type: 'organization',
            entity_id: organization.id,
            title: organization.name,
            subtitle: organization.email || organization.website || 'Organization',
            match_field: 'organization',
          });
        }
      }
      for (const deal of state.deals) {
        if (matchesText([deal.title, deal.notes], query)) {
          results.push({
            entity_type: 'deal',
            entity_id: deal.id,
            title: deal.title,
            subtitle: `${deal.stage} deal`,
            match_field: 'deal',
          });
        }
      }
      for (const activity of state.activities) {
        if (matchesText([activity.title, activity.description], query)) {
          results.push({
            entity_type: 'activity',
            entity_id: activity.id,
            title: activity.title,
            subtitle: activity.activity_type,
            match_field: 'activity',
          });
        }
      }

      return results.slice(0, limit);
    }

    const staticResponses: Record<string, unknown> = {
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
      list_contact_duplicate_candidates: [],
      list_custom_field_defs: [],
      list_custom_field_values_for_type: [],
      list_recent_audit_log: [],
      list_pending_proposed_actions: [],
      list_external_clients: [],
      'plugin:notification|is_permission_granted': false,
    };

    let callbackId = 0;

    window.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args?: InvokeArgs) => {
        switch (cmd) {
          case 'list_contacts':
            return listContacts(args);
          case 'create_contact':
            return createContact(args);
          case 'list_organizations':
            return state.organizations.filter((organization) => !organization.deleted_at);
          case 'create_organization':
            return createOrganization(args);
          case 'list_deals':
            return state.deals;
          case 'create_deal':
            return createDeal(args);
          case 'get_pipeline_summary':
            return [];
          case 'list_activities':
            return state.activities;
          case 'create_activity':
            return createActivity(args);
          case 'list_activity_links':
            return listActivityLinks(args);
          case 'add_activity_link':
            return addActivityLink(args);
          case 'global_search':
            return globalSearch(args);
          default:
            if (Object.prototype.hasOwnProperty.call(staticResponses, cmd)) {
              return staticResponses[cmd];
            }
            throw new Error(`Unmocked Tauri invoke in browser smoke test: ${cmd}`);
        }
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
}

export async function loadHashRoute(page: Page, route: string) {
  await page.goto(`/#${route}`);
  await page.reload();
  await expect(page).toHaveURL(new RegExp(`#${route}$`));
}
