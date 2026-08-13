import { expect, loadHashRoute, test } from './tauri-shim';

test('renders the browser app shell and dashboard route', async ({ page, assertNoConsoleErrors }) => {
  await page.goto('/');

  await expect(page.getByText('900CRM')).toBeVisible();
  const navigation = page.getByRole('navigation', { name: 'Main navigation' });
  await expect(navigation).toBeVisible();
  await expect(navigation.getByText('Workspace')).toBeVisible();
  await expect(navigation.getByText('Review')).toBeVisible();
  await expect(navigation.getByText('Admin')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByText('Total Contacts')).toBeVisible();
  await expect(navigation.getByRole('link', { name: 'Reports' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('renders key hash routes without native Tauri dialogs', async ({ page, assertNoConsoleErrors }) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();
  await expect(page.getByText('No contacts yet')).toBeVisible();

  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();
  await expect(page.getByText('Lead').first()).toBeVisible();

  await loadHashRoute(page, '/reports');
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible();
  await expect(page.getByText('Pipeline Overview')).toBeVisible();
  await expect(page.getByText('Activity Overview')).toBeVisible();

  await loadHashRoute(page, '/settings');
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Preferences' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Integrations' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Data', exact: true })).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toBeVisible();
  await expect(page.getByText('Not available yet')).toBeVisible();
  await expect(page.getByText('Multi-device sync is not implemented')).toBeVisible();

  await assertNoConsoleErrors();
});

test('switches primary workspace routes from the sidebar', async ({ page, assertNoConsoleErrors }) => {
  await page.goto('/');

  const navigation = page.getByRole('navigation', { name: 'Main navigation' });
  await navigation.getByRole('link', { name: 'Contacts' }).click();
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Pipeline' }).click();
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Reports' }).click();
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Settings' }).click();
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();

  await assertNoConsoleErrors();
});
