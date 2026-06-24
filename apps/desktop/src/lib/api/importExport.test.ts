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
  preflightContactsCsvImport,
  preflightCsv,
  preflightOrganizationsCsvImport,
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

  it('maps CSV preflight duplicate warning commands', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 2,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: 'contacts',
          row_number: 2,
          match_type: 'email',
          csv_value: 'ada@example.com',
          existing_entity_type: 'contact',
          existing_entity_id: 'contact-1',
          existing_display_label: 'Ada Lovelace',
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    await expect(preflightContactsCsvImport('/tmp/contacts.csv')).resolves.toMatchObject({
      entity_type: 'contacts',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_contacts_csv_import', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightOrganizationsCsvImport('/tmp/organizations.csv')).resolves.toMatchObject({
      entity_type: 'organizations',
      total_rows: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_csv_import', {
      file_path: '/tmp/organizations.csv',
    });
  });

  it('routes generic CSV preflight helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('organizations', '/tmp/orgs.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_csv_import', {
      file_path: '/tmp/orgs.csv',
    });
  });
});
