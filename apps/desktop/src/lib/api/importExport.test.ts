import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  exportContactsCsv,
  exportCsv,
  exportDealsCsv,
  exportOrganizationsCsv,
  importContactsCsv,
  importCsv,
  importDealsCsv,
  importOrganizationsCsv,
} from './importExport';

describe('import/export API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps contact CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce({ created: 2, skipped: 1, errors: ['Row 4'] });
    await expect(importContactsCsv('/tmp/contacts.csv')).resolves.toEqual({
      created: 2,
      skipped: 1,
      errors: ['Row 4'],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportContactsCsv('/tmp/contacts-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_csv', {
      file_path: '/tmp/contacts-export.csv',
    });
  });

  it('maps deal CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce({ created: 3, skipped: 0, errors: [] });
    await expect(importDealsCsv('/tmp/deals.csv')).resolves.toEqual({
      created: 3,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv', {
      file_path: '/tmp/deals.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportDealsCsv('/tmp/deals-export.csv')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_csv', {
      file_path: '/tmp/deals-export.csv',
    });
  });

  it('maps organization CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await expect(importOrganizationsCsv('/tmp/organizations.csv')).resolves.toEqual({
      created: 1,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/organizations.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportOrganizationsCsv('/tmp/organizations-export.csv')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/organizations-export.csv',
    });
  });

  it('routes generic CSV helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await importCsv('organizations', '/tmp/orgs.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/orgs.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportCsv('organizations', '/tmp/orgs-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/orgs-export.csv',
    });
  });
});
