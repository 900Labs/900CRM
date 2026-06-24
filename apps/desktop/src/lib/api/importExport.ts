/**
 * src/lib/api/importExport.ts - Tauri IPC wrappers for CSV import/export commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ImportExportEntity = 'contacts' | 'deals' | 'organizations';
export type ImportPreflightEntity = 'contacts' | 'organizations';

export interface ImportResult {
  created: number;
  skipped: number;
  errors: string[];
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
  match_type: 'email' | 'phone' | 'name';
  csv_value: string;
  existing_entity_type: 'contact' | 'organization';
  existing_entity_id: string;
  existing_display_label: string;
  reason: string;
}

const importCommands: Record<ImportExportEntity, string> = {
  contacts: 'import_contacts_csv',
  deals: 'import_deals_csv',
  organizations: 'import_organizations_csv',
};

const exportCommands: Record<ImportExportEntity, string> = {
  contacts: 'export_contacts_csv',
  deals: 'export_deals_csv',
  organizations: 'export_organizations_csv',
};

const preflightCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_csv_import',
  organizations: 'preflight_organizations_csv_import',
};

function filePathArgs(filePath: string) {
  return { file_path: filePath };
}

export async function importContactsCsv(filePath: string): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands.contacts, filePathArgs(filePath));
}

export async function exportContactsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.contacts, filePathArgs(filePath));
}

export async function preflightContactsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.contacts, filePathArgs(filePath));
}

export async function importDealsCsv(filePath: string): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands.deals, filePathArgs(filePath));
}

export async function exportDealsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.deals, filePathArgs(filePath));
}

export async function importOrganizationsCsv(filePath: string): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands.organizations, filePathArgs(filePath));
}

export async function exportOrganizationsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.organizations, filePathArgs(filePath));
}

export async function preflightOrganizationsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.organizations, filePathArgs(filePath));
}

export async function importCsv(
  entity: ImportExportEntity,
  filePath: string,
): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands[entity], filePathArgs(filePath));
}

export async function exportCsv(entity: ImportExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands[entity], filePathArgs(filePath));
}

export async function preflightCsv(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands[entity], filePathArgs(filePath));
}
