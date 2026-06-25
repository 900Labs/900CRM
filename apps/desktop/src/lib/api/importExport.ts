/**
 * src/lib/api/importExport.ts - Tauri IPC wrappers for import/export commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ImportExportEntity = 'contacts' | 'deals' | 'organizations';
export type ImportPreflightEntity = 'contacts' | 'deals' | 'organizations';
export type ImportFormat = 'csv' | 'json';
export type ExportFormat = 'csv' | 'json';
export type ContactImportTargetField =
  | 'first_name'
  | 'last_name'
  | 'org_name'
  | 'email'
  | 'phone'
  | 'address'
  | 'city'
  | 'country'
  | 'notes';
export type OrganizationImportTargetField =
  | 'name'
  | 'email'
  | 'phone'
  | 'website'
  | 'address_line1'
  | 'address_line2'
  | 'city'
  | 'region'
  | 'country'
  | 'postal_code'
  | 'description';
export type DealImportTargetField =
  | 'title'
  | 'value'
  | 'currency'
  | 'stage'
  | 'expected_close'
  | 'notes';
export type ImportTargetField =
  | ContactImportTargetField
  | DealImportTargetField
  | OrganizationImportTargetField;
export type ImportColumnMapping<TTarget extends string = ImportTargetField> = Record<
  string,
  TTarget | null
>;

export interface ImportResult {
  created: number;
  skipped: number;
  errors: string[];
}

export interface LocalBackupMetadata {
  backup_format_version: number;
  created_at: string;
  app_version: string;
  schema_version: number;
  device_id: string;
  database_file: string;
}

export interface LocalBackup {
  backup_dir: string;
  database_path: string;
  metadata_path: string;
  metadata: LocalBackupMetadata;
}

export interface ImportWithBackupResult {
  import: ImportResult;
  backup: LocalBackup;
}

export interface ImportPreflightReport {
  entity_type: ImportPreflightEntity;
  total_rows: number;
  duplicate_warning_count: number;
  warnings: ImportDuplicateWarning[];
}

export interface ImportDuplicateWarning {
  entity_type: ImportPreflightEntity;
  row_number: number;
  match_type: 'email' | 'phone' | 'name' | 'title';
  csv_value: string;
  existing_entity_type: 'contact' | 'deal' | 'organization';
  existing_entity_id: string;
  existing_display_label: string;
  reason: string;
}

const importCommands: Record<ImportFormat, Record<ImportExportEntity, string>> = {
  csv: {
    contacts: 'import_contacts_csv',
    deals: 'import_deals_csv',
    organizations: 'import_organizations_csv',
  },
  json: {
    contacts: 'import_contacts_json',
    deals: 'import_deals_json',
    organizations: 'import_organizations_json',
  },
};

const exportCommands: Record<ExportFormat, Record<ImportExportEntity, string>> = {
  csv: {
    contacts: 'export_contacts_csv',
    deals: 'export_deals_csv',
    organizations: 'export_organizations_csv',
  },
  json: {
    contacts: 'export_contacts_json',
    deals: 'export_deals_json',
    organizations: 'export_organizations_json',
  },
};

const preflightCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_csv_import',
  deals: 'preflight_deals_csv_import',
  organizations: 'preflight_organizations_csv_import',
};

const preflightJsonCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_json_import',
  deals: 'preflight_deals_json_import',
  organizations: 'preflight_organizations_json_import',
};

const importWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'import_contacts_csv_with_mapping',
  deals: 'import_deals_csv_with_mapping',
  organizations: 'import_organizations_csv_with_mapping',
};

const preflightWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_csv_import_with_mapping',
  deals: 'preflight_deals_csv_import_with_mapping',
  organizations: 'preflight_organizations_csv_import_with_mapping',
};

function filePathArgs(filePath: string) {
  return { file_path: filePath };
}

function filePathAndMappingArgs<TTarget extends string>(
  filePath: string,
  mapping: ImportColumnMapping<TTarget>,
) {
  return { file_path: filePath, mapping };
}

export async function importContactsCsv(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.contacts, filePathArgs(filePath));
}

export async function importContactsJson(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.contacts, filePathArgs(filePath));
}

export async function exportContactsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.contacts, filePathArgs(filePath));
}

export async function exportContactsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.contacts, filePathArgs(filePath));
}

export async function preflightContactsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.contacts, filePathArgs(filePath));
}

export async function preflightContactsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.contacts, filePathArgs(filePath));
}

export async function importContactsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightContactsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importDealsCsv(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.deals, filePathArgs(filePath));
}

export async function importDealsJson(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.deals, filePathArgs(filePath));
}

export async function exportDealsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.deals, filePathArgs(filePath));
}

export async function exportDealsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.deals, filePathArgs(filePath));
}

export async function preflightDealsCsvImport(filePath: string): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.deals, filePathArgs(filePath));
}

export async function preflightDealsJsonImport(filePath: string): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.deals, filePathArgs(filePath));
}

export async function importDealsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightDealsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importOrganizationsCsv(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.organizations, filePathArgs(filePath));
}

export async function importOrganizationsJson(filePath: string): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.organizations, filePathArgs(filePath));
}

export async function exportOrganizationsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.organizations, filePathArgs(filePath));
}

export async function exportOrganizationsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.organizations, filePathArgs(filePath));
}

export async function preflightOrganizationsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.organizations, filePathArgs(filePath));
}

export async function preflightOrganizationsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.organizations, filePathArgs(filePath));
}

export async function importOrganizationsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightOrganizationsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importCsv(
  entity: ImportExportEntity,
  filePath: string,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv[entity], filePathArgs(filePath));
}

export async function importJson(
  entity: ImportExportEntity,
  filePath: string,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json[entity], filePathArgs(filePath));
}

export async function importData(
  entity: ImportExportEntity,
  format: ImportFormat,
  filePath: string,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands[format][entity], filePathArgs(filePath));
}

export async function exportCsv(entity: ImportExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv[entity], filePathArgs(filePath));
}

export async function exportJson(entity: ImportExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json[entity], filePathArgs(filePath));
}

export async function exportData(
  entity: ImportExportEntity,
  format: ExportFormat,
  filePath: string,
): Promise<number> {
  return invoke<number>(exportCommands[format][entity], filePathArgs(filePath));
}

export async function preflightCsv(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands[entity], filePathArgs(filePath));
}

export async function preflightJson(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands[entity], filePathArgs(filePath));
}

export async function importCsvWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightCsvWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping),
  );
}
