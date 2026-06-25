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
  importContactsJsonWithMapping,
  importCsv,
  importCsvWithMapping,
  importData,
  importDealsCsv,
  importDealsCsvWithMapping,
  importDealsJson,
  importDealsJsonWithMapping,
  importJsonWithMapping,
  importJson,
  importOrganizationsCsv,
  importOrganizationsCsvWithMapping,
  importOrganizationsJson,
  importOrganizationsJsonWithMapping,
  preflightContactsCsvImport,
  preflightContactsCsvImportWithMapping,
  preflightContactsJsonImport,
  preflightContactsJsonImportWithMapping,
  preflightCsv,
  preflightCsvWithMapping,
  preflightDealsCsvImport,
  preflightDealsCsvImportWithMapping,
  preflightDealsJsonImport,
  preflightDealsJsonImportWithMapping,
  preflightJson,
  preflightJsonWithMapping,
  preflightOrganizationsCsvImport,
  preflightOrganizationsCsvImportWithMapping,
  preflightOrganizationsJsonImport,
  preflightOrganizationsJsonImportWithMapping,
  previewContactsJsonImport,
  previewDealsJsonImport,
  previewJson,
  previewOrganizationsJsonImport,
} from './importExport';

const sampleBackup = {
  backup_dir: '/tmp/app-data/pre-import-backups/backup-1',
  database_path: '/tmp/app-data/pre-import-backups/backup-1/900crm.db',
  metadata_path: '/tmp/app-data/pre-import-backups/backup-1/metadata.json',
  metadata: {
    backup_format_version: 1,
    created_at: '2026-06-25T00:00:00Z',
    app_version: '0.1.0',
    schema_version: 1,
    device_id: 'device-1',
    database_file: '900crm.db',
  },
};

function importWithBackup(created: number, skipped = 0, errors: string[] = []) {
  return {
    import: { created, skipped, errors },
    backup: sampleBackup,
  };
}

describe('import/export API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps contact CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2, 1, ['Row 4']));
    await expect(importContactsCsv('/tmp/contacts.csv')).resolves.toEqual(
      importWithBackup(2, 1, ['Row 4']),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importContactsJson('/tmp/contacts.json')).resolves.toEqual(importWithBackup(2));
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
    invokeMock.mockResolvedValueOnce(importWithBackup(3));
    await expect(importDealsCsv('/tmp/deals.csv')).resolves.toEqual(importWithBackup(3));
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv', {
      file_path: '/tmp/deals.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(3));
    await expect(importDealsJson('/tmp/deals.json')).resolves.toEqual(importWithBackup(3));
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
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importOrganizationsCsv('/tmp/organizations.csv')).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/organizations.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importOrganizationsJson('/tmp/organizations.json')).resolves.toEqual(
      importWithBackup(1),
    );
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
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsv('organizations', '/tmp/orgs.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/orgs.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportCsv('organizations', '/tmp/orgs-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/orgs-export.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await importJson('contacts', '/tmp/contacts.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json', {
      file_path: '/tmp/contacts.json',
    });
  });

  it('routes generic import/export helpers by entity and format', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await importData('deals', 'json', '/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(3));
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

  it('passes duplicate auto-merge only for contact and organization imports when enabled', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importContactsCsv('/tmp/contacts.csv', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
      merge_duplicates: true,
    });

    const contactMapping = { Email: 'email', First: 'first_name' } as const;
    invokeMock.mockResolvedValueOnce(importWithBackup(0));
    await importContactsJsonWithMapping('/tmp/contacts.json', contactMapping, {
      mergeDuplicates: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json_with_mapping', {
      file_path: '/tmp/contacts.json',
      mapping: contactMapping,
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importOrganizationsJson('/tmp/organizations.json', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json', {
      file_path: '/tmp/organizations.json',
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importData('deals', 'csv', '/tmp/deals.csv', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv', {
      file_path: '/tmp/deals.csv',
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

  it('maps JSON preview commands', async () => {
    const preview = {
      total_rows: 1,
      headers: ['first_name', 'email'],
      rows: [{ row_number: 2, values: { first_name: 'Ada', email: 'ada@example.com' } }],
    };

    invokeMock.mockResolvedValueOnce(preview);
    await expect(previewContactsJsonImport('/tmp/contacts.json')).resolves.toEqual(preview);
    expect(invokeMock).toHaveBeenLastCalledWith('preview_contacts_json_import', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['title'] });
    await previewDealsJsonImport('/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_deals_json_import', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['name'] });
    await previewOrganizationsJsonImport('/tmp/organizations.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_organizations_json_import', {
      file_path: '/tmp/organizations.json',
    });
  });

  it('routes generic JSON preview helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['name'],
      rows: [{ row_number: 2, values: { name: 'Acme' } }],
    });

    await previewJson('organizations', '/tmp/orgs.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_organizations_json_import', {
      file_path: '/tmp/orgs.json',
    });
  });

  it('maps contact CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      'Given Name': 'first_name',
      Surname: 'last_name',
      'Email Address': 'email',
      Ignore: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importContactsCsvWithMapping('/tmp/contacts.csv', mapping)).resolves.toEqual(
      importWithBackup(1),
    );
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

  it('maps contact JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      given: 'first_name',
      surname: 'last_name',
      mail: 'email',
      ignore: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importContactsJsonWithMapping('/tmp/contacts.json', mapping)).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json_with_mapping', {
      file_path: '/tmp/contacts.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightContactsJsonImportWithMapping('/tmp/contacts.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_contacts_json_import_with_mapping',
      {
        file_path: '/tmp/contacts.json',
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

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
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

  it('maps organization JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      company: 'name',
      inbox: 'email',
      telephone: 'phone',
      skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importOrganizationsJsonWithMapping('/tmp/organizations.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json_with_mapping', {
      file_path: '/tmp/organizations.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightOrganizationsJsonImportWithMapping('/tmp/organizations.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_json_import_with_mapping',
      {
        file_path: '/tmp/organizations.json',
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

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
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

  it('maps deal JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      opportunity: 'title',
      amount: 'value',
      phase: 'stage',
      skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importDealsJsonWithMapping('/tmp/deals.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json_with_mapping', {
      file_path: '/tmp/deals.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightDealsJsonImportWithMapping('/tmp/deals.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_json_import_with_mapping',
      {
        file_path: '/tmp/deals.json',
        mapping,
      },
    );
  });

  it('routes generic mapped CSV helpers by entity', async () => {
    const mapping = {
      Company: 'name',
      Inbox: 'email',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
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

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
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

  it('routes generic mapped JSON helpers by entity', async () => {
    const mapping = {
      company: 'name',
      inbox: 'email',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('organizations', '/tmp/orgs.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json_with_mapping', {
      file_path: '/tmp/orgs.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('organizations', '/tmp/orgs.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_json_import_with_mapping',
      {
        file_path: '/tmp/orgs.json',
        mapping,
      },
    );

    const dealMapping = {
      opportunity: 'title',
      amount: 'value',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('deals', '/tmp/deals.json', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json_with_mapping', {
      file_path: '/tmp/deals.json',
      mapping: dealMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('deals', '/tmp/deals.json', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_json_import_with_mapping',
      {
        file_path: '/tmp/deals.json',
        mapping: dealMapping,
      },
    );
  });
});
