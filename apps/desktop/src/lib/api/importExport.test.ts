import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  exportContactsCsv,
  exportContactsJson,
  exportCsv,
  exportData,
  exportDealsCsv,
  exportDealsJson,
  exportJson,
  exportOrganizationsCsv,
  exportOrganizationsJson,
  importContactsCsv,
  importContactsCsvWithMapping,
  importContactsJson,
  importCsv,
  importCsvWithMapping,
  importData,
  importDealsCsv,
  importDealsCsvWithMapping,
  importDealsJson,
  importJson,
  importOrganizationsCsv,
  importOrganizationsCsvWithMapping,
  importOrganizationsJson,
  preflightContactsCsvImport,
  preflightContactsCsvImportWithMapping,
  preflightContactsJsonImport,
  preflightCsv,
  preflightCsvWithMapping,
  preflightDealsCsvImport,
  preflightDealsCsvImportWithMapping,
  preflightDealsJsonImport,
  preflightJson,
  preflightOrganizationsCsvImport,
  preflightOrganizationsCsvImportWithMapping,
  preflightOrganizationsJsonImport,
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

    invokeMock.mockResolvedValueOnce({ created: 2, skipped: 0, errors: [] });
    await expect(importContactsJson('/tmp/contacts.json')).resolves.toEqual({
      created: 2,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportContactsCsv('/tmp/contacts-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_csv', {
      file_path: '/tmp/contacts-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportContactsJson('/tmp/contacts-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_json', {
      file_path: '/tmp/contacts-export.json',
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

    invokeMock.mockResolvedValueOnce({ created: 3, skipped: 0, errors: [] });
    await expect(importDealsJson('/tmp/deals.json')).resolves.toEqual({
      created: 3,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportDealsCsv('/tmp/deals-export.csv')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_csv', {
      file_path: '/tmp/deals-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportDealsJson('/tmp/deals-export.json')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_json', {
      file_path: '/tmp/deals-export.json',
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

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await expect(importOrganizationsJson('/tmp/organizations.json')).resolves.toEqual({
      created: 1,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json', {
      file_path: '/tmp/organizations.json',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportOrganizationsCsv('/tmp/organizations-export.csv')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/organizations-export.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportOrganizationsJson('/tmp/organizations-export.json')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_json', {
      file_path: '/tmp/organizations-export.json',
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

    invokeMock.mockResolvedValueOnce({ created: 2, skipped: 0, errors: [] });
    await importJson('contacts', '/tmp/contacts.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json', {
      file_path: '/tmp/contacts.json',
    });
  });

  it('routes generic import/export helpers by entity and format', async () => {
    invokeMock.mockResolvedValueOnce({ created: 2, skipped: 0, errors: [] });
    await importData('deals', 'json', '/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce({ created: 3, skipped: 0, errors: [] });
    await importData('contacts', 'csv', '/tmp/contacts.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportJson('organizations', '/tmp/orgs-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_json', {
      file_path: '/tmp/orgs-export.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportData('deals', 'json', '/tmp/deals-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_json', {
      file_path: '/tmp/deals-export.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportData('contacts', 'csv', '/tmp/contacts-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_csv', {
      file_path: '/tmp/contacts-export.csv',
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

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightDealsCsvImport('/tmp/deals.csv')).resolves.toMatchObject({
      entity_type: 'deals',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_csv_import', {
      file_path: '/tmp/deals.csv',
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

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('deals', '/tmp/deals.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_csv_import', {
      file_path: '/tmp/deals.csv',
    });
  });

  it('maps JSON preflight duplicate warning commands', async () => {
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
    await expect(preflightContactsJsonImport('/tmp/contacts.json')).resolves.toMatchObject({
      entity_type: 'contacts',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_contacts_json_import', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightOrganizationsJsonImport('/tmp/organizations.json')).resolves.toMatchObject({
      entity_type: 'organizations',
      total_rows: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_json_import', {
      file_path: '/tmp/organizations.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightDealsJsonImport('/tmp/deals.json')).resolves.toMatchObject({
      entity_type: 'deals',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_json_import', {
      file_path: '/tmp/deals.json',
    });
  });

  it('routes generic JSON preflight helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('organizations', '/tmp/orgs.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_json_import', {
      file_path: '/tmp/orgs.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('deals', '/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_json_import', {
      file_path: '/tmp/deals.json',
    });
  });

  it('maps contact CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      'Given Name': 'first_name',
      Surname: 'last_name',
      'Email Address': 'email',
      Ignore: null,
    } as const;

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await expect(importContactsCsvWithMapping('/tmp/contacts.csv', mapping)).resolves.toEqual({
      created: 1,
      skipped: 0,
      errors: [],
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv_with_mapping', {
      file_path: '/tmp/contacts.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightContactsCsvImportWithMapping('/tmp/contacts.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_contacts_csv_import_with_mapping',
      {
        file_path: '/tmp/contacts.csv',
        mapping,
      },
    );
  });

  it('maps organization CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      Company: 'name',
      Inbox: 'email',
      Telephone: 'phone',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await importOrganizationsCsvWithMapping('/tmp/organizations.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv_with_mapping', {
      file_path: '/tmp/organizations.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightOrganizationsCsvImportWithMapping('/tmp/organizations.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_csv_import_with_mapping',
      {
        file_path: '/tmp/organizations.csv',
        mapping,
      },
    );
  });

  it('maps deal CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      Opportunity: 'title',
      Amount: 'value',
      Phase: 'stage',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await importDealsCsvWithMapping('/tmp/deals.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv_with_mapping', {
      file_path: '/tmp/deals.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightDealsCsvImportWithMapping('/tmp/deals.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_csv_import_with_mapping',
      {
        file_path: '/tmp/deals.csv',
        mapping,
      },
    );
  });

  it('routes generic mapped CSV helpers by entity', async () => {
    const mapping = {
      Company: 'name',
      Inbox: 'email',
    } as const;

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await importCsvWithMapping('organizations', '/tmp/orgs.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv_with_mapping', {
      file_path: '/tmp/orgs.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('organizations', '/tmp/orgs.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_csv_import_with_mapping',
      {
        file_path: '/tmp/orgs.csv',
        mapping,
      },
    );

    const dealMapping = {
      Opportunity: 'title',
      Amount: 'value',
    } as const;

    invokeMock.mockResolvedValueOnce({ created: 1, skipped: 0, errors: [] });
    await importCsvWithMapping('deals', '/tmp/deals.csv', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv_with_mapping', {
      file_path: '/tmp/deals.csv',
      mapping: dealMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('deals', '/tmp/deals.csv', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_csv_import_with_mapping',
      {
        file_path: '/tmp/deals.csv',
        mapping: dealMapping,
      },
    );
  });
});
