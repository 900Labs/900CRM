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
  merged?: number;
  skipped: number;
  errors: string[];
  rollback_plan?: ImportRollbackPlan | null;
}

export interface ImportOptions {
  mergeDuplicates?: boolean;
}

export type ImportRollbackOperation = 'created' | 'merged';

export interface ContactImportRollbackSnapshot {
  contact_type: string;
  first_name: string;
  last_name: string;
  org_name: string;
  email: string;
  phone: string;
  address: string;
  city: string;
  country: string;
  org_id?: string | null;
  organization_id?: string | null;
  notes: string;
  updated_at: string;
}

export interface DealImportRollbackSnapshot {
  title: string;
  value: number;
  currency: string;
  stage: string;
  probability: number;
  expected_close?: string | null;
  contact_id?: string | null;
  organization_id?: string | null;
  notes: string;
  updated_at: string;
}

export interface OrganizationImportRollbackSnapshot {
  name: string;
  email?: string | null;
  phone?: string | null;
  website?: string | null;
  address_line1?: string | null;
  address_line2?: string | null;
  city?: string | null;
  region?: string | null;
  country?: string | null;
  postal_code?: string | null;
  source?: string | null;
  description?: string | null;
  updated_at: string;
}

export type ImportRollbackAction =
  | {
      entity_type: 'contact';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: ContactImportRollbackSnapshot | null;
      post_import: ContactImportRollbackSnapshot;
    }
  | {
      entity_type: 'deal';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: DealImportRollbackSnapshot | null;
      post_import: DealImportRollbackSnapshot;
    }
  | {
      entity_type: 'organization';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: OrganizationImportRollbackSnapshot | null;
      post_import: OrganizationImportRollbackSnapshot;
    };

export interface ImportRollbackPlan {
  token: string;
  actions: ImportRollbackAction[];
}

export interface ImportRollbackRowError {
  entity_type: 'contact' | 'deal' | 'organization';
  entity_id: string;
  row_number: number;
  code: string;
  message: string;
}

export interface ImportRollbackResult {
  token: string;
  rolled_back: number;
  skipped: number;
  errors: ImportRollbackRowError[];
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

export interface JsonImportPreview {
  total_rows: number;
  headers: string[];
  rows: JsonImportPreviewRow[];
}

export interface JsonImportPreviewRow {
  row_number: number;
  values: Record<string, string>;
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

const previewJsonCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preview_contacts_json_import',
  deals: 'preview_deals_json_import',
  organizations: 'preview_organizations_json_import',
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

const importJsonWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'import_contacts_json_with_mapping',
  deals: 'import_deals_json_with_mapping',
  organizations: 'import_organizations_json_with_mapping',
};

const preflightJsonWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_json_import_with_mapping',
  deals: 'preflight_deals_json_import_with_mapping',
  organizations: 'preflight_organizations_json_import_with_mapping',
};

function importOptionArgs(options?: ImportOptions) {
  return options?.mergeDuplicates === undefined
    ? {}
    : { merge_duplicates: options.mergeDuplicates };
}

function filePathArgs(filePath: string, options?: ImportOptions) {
  return { file_path: filePath, ...importOptionArgs(options) };
}

function filePathAndMappingArgs<TTarget extends string>(
  filePath: string,
  mapping: ImportColumnMapping<TTarget>,
  options?: ImportOptions,
) {
  return { file_path: filePath, mapping, ...importOptionArgs(options) };
}

export async function importContactsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.contacts, filePathArgs(filePath, options));
}

export async function importContactsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.contacts, filePathArgs(filePath, options));
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

export async function previewContactsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.contacts, filePathArgs(filePath));
}

export async function rollbackCompletedImport(
  rollbackPlan: ImportRollbackPlan,
): Promise<ImportRollbackResult> {
  return invoke<ImportRollbackResult>('rollback_completed_import', {
    rollback_plan: rollbackPlan,
  });
}

export async function importContactsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importContactsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping, options),
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

export async function preflightContactsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importDealsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.deals, filePathArgs(filePath, options));
}

export async function importDealsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.deals, filePathArgs(filePath, options));
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

export async function previewDealsJsonImport(filePath: string): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.deals, filePathArgs(filePath));
}

export async function importDealsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importDealsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping, options),
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

export async function preflightDealsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importOrganizationsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.organizations, filePathArgs(filePath, options));
}

export async function importOrganizationsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.organizations, filePathArgs(filePath, options));
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

export async function previewOrganizationsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.organizations, filePathArgs(filePath));
}

export async function importOrganizationsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importOrganizationsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping, options),
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

export async function preflightOrganizationsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importCsv(
  entity: ImportExportEntity,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.csv[entity],
    filePathArgs(filePath, options),
  );
}

export async function importJson(
  entity: ImportExportEntity,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.json[entity],
    filePathArgs(filePath, options),
  );
}

export async function importData(
  entity: ImportExportEntity,
  format: ImportFormat,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands[format][entity],
    filePathArgs(filePath, options),
  );
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

export async function previewJson(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands[entity], filePathArgs(filePath));
}

export async function importCsvWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importJsonWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping, options),
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

export async function preflightJsonWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping),
  );
}
