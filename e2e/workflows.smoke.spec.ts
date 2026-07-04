import { expect, loadHashRoute, test } from './tauri-shim';

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

  await assertNoConsoleErrors();
});

test('creates an activity through the visible UI and shows it in Activities', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/activities');
  await expect(page.getByRole('heading', { name: 'Activities' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
  const quickAddForm = page.getByRole('form', { name: 'Add Activity' });
  await quickAddForm.getByLabel('Subject').fill('Follow up on grant paperwork');
  await quickAddForm.getByRole('button', { name: 'Add', exact: true }).click();

  const activityRow = page.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  });
  await expect(activityRow).toBeVisible();
  await expect(activityRow.getByText('Upcoming')).toBeVisible();

  await assertNoConsoleErrors();
});
