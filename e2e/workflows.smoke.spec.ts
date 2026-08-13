import { expect, loadHashRoute, test } from './tauri-shim';

test.describe.configure({ mode: 'serial' });

function localDateInputValue(date = new Date()): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

test('loads the dashboard sample workspace and shows the follow-up on the dashboard', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Start your workspace' })).toBeVisible();

  await page.getByRole('button', { name: 'Load sample workspace' }).click();
  await expect(page.getByText('Sample workspace loaded.').first()).toBeVisible();
  await expect(page.getByText('Call Amara about rollout timeline')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Load sample workspace' })).toBeHidden();

  const storedSampleState = async () =>
    page.evaluate(() => window.localStorage.getItem('900crm.browser-smoke.state') ?? '');
  await expect.poll(storedSampleState).toContain('Amara');
  await expect.poll(storedSampleState).toContain('Northstar Cooperative');
  await expect.poll(storedSampleState).toContain('Solar inventory rollout');
  await expect.poll(storedSampleState).toContain('Call Amara about rollout timeline');

  const sampleOrganizationId = await page.evaluate(() => {
    const raw = window.localStorage.getItem('900crm.browser-smoke.state');
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as {
      organizations?: Array<{ id: string; name: string }>;
    };
    return parsed.organizations?.find((organization) => organization.name === 'Northstar Cooperative')
      ?.id ?? null;
  });
  expect(sampleOrganizationId).toBeTruthy();

  await loadHashRoute(page, `/organizations/${sampleOrganizationId}`);
  await expect(page.getByRole('heading', { name: 'Northstar Cooperative' })).toBeVisible();
  const sampleWorkspace = page.locator('.account-workspace');
  await expect(
    sampleWorkspace.locator('.workspace-metric').filter({ hasText: 'People' }).getByText('1'),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Amara Okafor' })).toBeVisible();

  const sampleDealId = await page.evaluate(() => {
    const raw = window.localStorage.getItem('900crm.browser-smoke.state');
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as {
      deals?: Array<{ id: string; title?: string; name?: string }>;
    };
    return parsed.deals?.find((deal) => (deal.title ?? deal.name) === 'Solar inventory rollout')?.id ?? null;
  });
  expect(sampleDealId).toBeTruthy();

  await loadHashRoute(page, `/pipeline/${sampleDealId}`);
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Solar inventory rollout' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('creates a contact through the visible UI and shows it in Contacts and global search', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('First Name').fill('Ada');
  await dialog.getByLabel('Last Name').fill('Lovelace');
  await dialog.getByLabel('Email').fill('ada@example.test');
  await dialog.getByLabel('Phone').fill('+1 555 0101');
  await dialog.getByLabel('Organization').fill('Analytical Engine Guild');
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Ada Lovelace')).toBeVisible();
  await expect(page.getByText('ada@example.test')).toBeVisible();

  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Ada');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await expect(searchResults.getByText('Ada Lovelace')).toBeVisible();
  await expect(searchResults.getByText('Contacts')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows a customer 360 summary for a contact with linked sales work', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const contactDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(contactDialog).toBeVisible();

  await contactDialog.getByLabel('First Name').fill('Maya');
  await contactDialog.getByLabel('Last Name').fill('Chen');
  await contactDialog.getByLabel('Email').fill('maya@example.test');
  await contactDialog.getByLabel('Phone').fill('+1 555 0303');
  await contactDialog.getByLabel('Organization').fill('Greenfield Solar');
  await contactDialog.getByRole('button', { name: 'Save' }).click();

  await expect(contactDialog).toBeHidden();
  await expect(page.getByText('Maya Chen')).toBeVisible();

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const contacts = await invoke('list_contacts', {
      params: {
        page: 1,
        per_page: 50,
        search_query: 'Maya',
      },
    }) as { contacts: Array<{ id: string }> };
    const contact = contacts.contacts[0];
    if (!contact) {
      throw new Error('Seed contact was not created.');
    }
    const futureDate = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);
    const linkedOnlyDate = new Date(Date.now() + 11 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);

    await invoke('create_deal', {
      title: 'Solar upgrade expansion',
      value: 42000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-07-31',
      contact_id: contact.id,
      organization_id: '',
      notes: 'Expansion opportunity.',
    });

    await invoke('create_activity', {
      activity_type: 'call',
      title: 'Call Maya about implementation timeline',
      description: '',
      due_date: futureDate,
      contact_id: contact.id,
      deal_id: '',
    });

    const linkedOnlyActivity = await invoke('create_activity', {
      activity_type: 'meeting',
      title: 'Linked-only relationship review',
      description: '',
      due_date: linkedOnlyDate,
      contact_id: '',
      deal_id: '',
    }) as { id: string };

    await invoke('add_activity_link', {
      activity_id: linkedOnlyActivity.id,
      entity_type: 'contact',
      entity_id: contact.id,
    });

    return { contactId: contact.id };
  });

  await loadHashRoute(page, `/contacts/${seed.contactId}`);
  await expect(page).toHaveURL(new RegExp(`#/contacts/${seed.contactId}$`));
  await expect(page.getByRole('heading', { name: 'Maya Chen' })).toBeVisible();

  const workspace = page.locator('.customer-workspace');
  await expect(workspace.getByRole('heading', { name: 'Customer 360 Summary' })).toBeVisible();
  await expect(workspace.getByText('On Track')).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'Open Deals' }).getByText('1')).toBeVisible();
  await expect(workspace.getByText('$42,000')).toBeVisible();
  await expect(
    workspace.locator('.workspace-metric').filter({ hasText: 'Next Follow-Up' })
      .getByText('Call Maya about implementation timeline'),
  ).toBeVisible();
  const contactTimeline = page.locator('.detail-activity');
  await expect(contactTimeline.getByText('Linked-only relationship review')).toBeVisible();
  await expect(
    contactTimeline.getByRole('button', { name: 'Open Contact Maya Chen' }).first(),
  ).toBeVisible();

  await workspace.getByRole('button', { name: 'Add Follow-Up' }).click();
  let dialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Contact')).toHaveValue(seed.contactId);
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();

  await workspace.getByRole('button', { name: 'Add Deal' }).click();
  dialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Contact')).toHaveValue(seed.contactId);
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();

  await assertNoConsoleErrors();
});

test('creates an organization through the visible UI and shows it in Organizations', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await expect(page.getByRole('heading', { name: 'Organizations', exact: true })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Name').fill('Atlas Cooperative');
  await dialog.getByLabel('Email').fill('hello@atlas.example');
  await dialog.getByLabel('Phone').fill('+1 555 0202');
  await dialog.getByLabel('Website').fill('https://atlas.example');
  await dialog.getByLabel('City').fill('Lagos');
  await dialog.getByLabel('Country').fill('Nigeria');
  await dialog.getByLabel('Description').fill('Regional partner for solar deployments.');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Atlas Cooperative')).toBeVisible();
  await expect(page.getByText('hello@atlas.example')).toBeVisible();
  await expect(page.getByText('Lagos, Nigeria')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows an account 360 workspace for an organization with linked work', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await expect(page.getByRole('heading', { name: 'Organizations', exact: true })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Name').fill('Helios Account');
  await dialog.getByLabel('Email').fill('hello@helios.example');
  await dialog.getByLabel('Website').fill('https://helios.example');
  await dialog.getByLabel('City').fill('Nairobi');
  await dialog.getByLabel('Country').fill('Kenya');
  await dialog.getByLabel('Description').fill('Regional account for off-grid clinics.');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();
  await expect(dialog).toBeHidden();

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }
    const futureDate = new Date(Date.now() + 14 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);

    const organizations = await invoke('list_organizations') as Array<{ id: string; name: string }>;
    const organization = organizations.find((candidate) => candidate.name === 'Helios Account');
    if (!organization) {
      throw new Error('Seed organization was not created.');
    }

    const contact = await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Nia',
      last_name: 'Mensah',
      org_name: 'Helios Account',
      email: 'nia@helios.example',
      phone: '+254 555 0101',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
    }) as { id: string };

    await invoke('link_contact_to_organization', {
      contact_id: contact.id,
      organization_id: organization.id,
    });

    const deal = await invoke('create_deal', {
      title: 'Clinic electrification rollout',
      value: 73000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-08-15',
      contact_id: '',
      organization_id: organization.id,
      notes: 'Account-level expansion opportunity.',
    }) as { id: string };

    const activity = await invoke('create_activity', {
      activity_type: 'meeting',
      title: 'Review Helios implementation plan',
      description: '',
      due_date: futureDate,
      contact_id: '',
      deal_id: '',
    }) as { id: string };

    await invoke('add_activity_link', {
      activity_id: activity.id,
      entity_type: 'organization',
      entity_id: organization.id,
    });

    await invoke('add_activity_link', {
      activity_id: activity.id,
      entity_type: 'deal',
      entity_id: deal.id,
    });

    return { organizationId: organization.id, contactId: contact.id, dealId: deal.id };
  });

  await loadHashRoute(page, `/organizations/${seed.organizationId}`);
  await expect(page).toHaveURL(new RegExp(`#/organizations/${seed.organizationId}$`));
  await expect(page.getByRole('heading', { name: 'Helios Account' })).toBeVisible();

  const workspace = page.locator('.account-workspace');
  await expect(workspace.getByRole('heading', { name: 'Account 360 Summary' })).toBeVisible();
  await expect(workspace.getByText('On Track')).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'People' }).getByText('1')).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'Open Deals' }).getByText('1')).toBeVisible();
  await expect(workspace.getByText('$73,000')).toBeVisible();
  await expect(
    workspace.locator('.workspace-metric').filter({ hasText: 'Next Follow-Up' })
      .getByText('Review Helios implementation plan'),
  ).toBeVisible();

  await expect(page.getByRole('button', { name: 'Nia Mensah' })).toBeVisible();
  await expect(
    page.getByLabel('Linked Deals').getByText('Clinic electrification rollout'),
  ).toBeVisible();
  await expect(
    page.getByLabel('Account Activity').getByText('Review Helios implementation plan'),
  ).toBeVisible();
  const accountActivity = page.getByLabel('Account Activity');
  await expect(
    accountActivity.getByRole('button', { name: 'Open Organization Helios Account' }).first(),
  ).toBeVisible();
  await expect(accountActivity.getByText('Deal')).toBeVisible();
  await expect(accountActivity.getByText('Clinic electrification rollout')).toBeVisible();

  await workspace.getByRole('button', { name: 'Add Follow-Up' }).click();
  let accountDialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(accountDialog).toBeVisible();
  await expect(accountDialog.getByLabel('Organization')).toHaveValue(seed.organizationId);
  await accountDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(accountDialog).toBeHidden();

  await workspace.getByRole('button', { name: 'Add Deal' }).click();
  accountDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(accountDialog).toBeVisible();
  await expect(accountDialog.getByLabel('Organization')).toHaveValue(seed.organizationId);
  await accountDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(accountDialog).toBeHidden();

  await loadHashRoute(page, '/');
  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Helios');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await searchResults.getByText('Helios Account').click();
  await expect(page).toHaveURL(new RegExp(`#/organizations/${seed.organizationId}$`));

  await assertNoConsoleErrors();
});

test('routes global contact search results into direct contact detail outside the loaded list page', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/');

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    for (let index = 0; index < 55; index += 1) {
      await invoke('create_contact', {
        contact_type: 'person',
        first_name: `Contact ${String(index).padStart(2, '0')}`,
        last_name: 'Paged',
        org_name: '',
        email: `contact-${index}@paged.example`,
        phone: '',
        address: '',
        city: '',
        country: '',
        org_id: '',
        notes: '',
      });
    }

    const target = await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Zzzara',
      last_name: 'Searchonly',
      org_name: '',
      email: 'zzzara.searchonly@example.test',
      phone: '',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
    }) as { id: string };

    return { contactId: target.id };
  });

  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Zzzara');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await searchResults.getByText('Zzzara Searchonly').click();

  await expect(page).toHaveURL(new RegExp(`#/contacts/${seed.contactId}$`));
  await expect(page.getByRole('heading', { name: 'Zzzara Searchonly' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('creates a deal through the visible UI and shows it in Pipeline', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Deal Name').fill('Village Solar Rollout');
  await dialog.getByLabel('Value').fill('12500');
  await dialog.getByLabel('Description').fill('Starter pipeline opportunity.');
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Village Solar Rollout')).toBeVisible();
  await expect(page.getByText('$12,500').first()).toBeVisible();

  const overview = page.getByTestId('pipeline-forecast-overview');
  const forecastSummary = overview.getByLabel('Pipeline forecast summary');
  await expect(overview.getByRole('heading', { name: 'Forecast and Stage Health' })).toBeVisible();
  await expect(overview.getByText('Open Pipeline')).toBeVisible();
  await expect(forecastSummary.getByText('$12,500')).toBeVisible();
  await expect(overview.getByRole('heading', { name: 'Stage Health', exact: true })).toBeVisible();

  await assertNoConsoleErrors();
});

test('opens a pipeline deal guidance drawer and refreshes follow-up state', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  const expectedClose = new Date(Date.now() + 21 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);
  const followUpDate = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const dealDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dealDialog).toBeVisible();
  await dealDialog.getByLabel('Deal Name').fill('Guided pipeline rollout');
  await dealDialog.getByLabel('Value').fill('50000');
  await dealDialog.getByLabel('Probability').fill('50');
  await dealDialog.locator('#modal-deal-close-date').fill(expectedClose);
  await dealDialog.getByLabel('Description').fill('Needs the next sales action.');
  await dealDialog.getByRole('button', { name: 'Save' }).click();
  await expect(dealDialog).toBeHidden();

  const overview = page.getByTestId('pipeline-forecast-overview');
  const forecastSummary = overview.getByLabel('Pipeline forecast summary');
  await expect(overview.getByText('Weighted Forecast')).toBeVisible();
  await expect(forecastSummary.getByText('$25,000').first()).toBeVisible();
  await expect(overview.getByText('Focus Stage')).toBeVisible();

  const needsFollowUpCard = page.getByRole('button', { name: /Guided pipeline rollout/ });
  await expect(needsFollowUpCard).toContainText('Needs Follow-Up');
  await needsFollowUpCard.click();

  let drawer = page.getByRole('dialog', { name: 'Guided pipeline rollout' });
  await expect(drawer).toBeVisible();
  await expect(drawer.getByText('Needs Follow-Up')).toBeVisible();
  await expect(drawer.getByText('$50,000')).toBeVisible();
  await expect(drawer.getByText('$25,000')).toBeVisible();

  await drawer.getByRole('button', { name: 'Add Follow-Up' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('#modal-activity-deal')).not.toHaveValue('');
  await dialog.getByLabel('Subject').fill('Schedule guided rollout call');
  await dialog.locator('#modal-activity-due-date').fill(followUpDate);
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  drawer = page.getByRole('dialog', { name: 'Guided pipeline rollout' });
  await expect(drawer.getByText('Stale')).toBeVisible();
  await expect(drawer.getByText('Schedule guided rollout call')).toBeVisible();

  await drawer.getByRole('button', { name: 'Close' }).click();
  await expect(drawer).toBeHidden();

  await assertNoConsoleErrors();
});

test('creates an activity through the visible UI and shows it in Activities', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/activities');
  await expect(page.getByRole('heading', { name: 'Activities' })).toBeVisible();

  const today = localDateInputValue();

  await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
  const quickAddForm = page.getByRole('form', { name: 'Add Activity' });
  await quickAddForm.getByLabel('Subject').fill('Follow up on grant paperwork');
  await quickAddForm.getByLabel('Due Date').fill(today);
  await quickAddForm.getByRole('button', { name: 'Add', exact: true }).click();

  const workbench = page.getByTestId('activity-workbench');
  await expect(workbench.getByRole('heading', { name: 'Due Workbench' })).toBeVisible();
  await expect(workbench.getByText('Open work')).toBeVisible();

  const todaySection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'Today', exact: true }),
  });
  const activityRow = todaySection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  });
  await expect(activityRow).toBeVisible();
  await expect(activityRow.getByText('Upcoming')).toBeVisible();

  await activityRow.getByRole('button', { name: 'Snooze' }).click();
  const thisWeekSection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'This Week', exact: true }),
  });
  const snoozedRow = thisWeekSection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  });
  await expect(snoozedRow).toBeVisible();

  await snoozedRow.getByRole('button', { name: 'Mark Complete' }).click();
  const completedSection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'Completed', exact: true }),
  });
  await expect(completedSection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  })).toBeVisible();

  await assertNoConsoleErrors();
});
