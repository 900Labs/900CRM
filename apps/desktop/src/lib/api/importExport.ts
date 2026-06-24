/**
 * src/lib/api/importExport.ts - Tauri IPC wrappers for CSV import/export commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ImportExportEntity = 'contacts' | 'deals' | 'organizations';

export interface ImportResult {
  created: number;
  skipped: number;
  errors: string[];
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

function filePathArgs(filePath: string) {
  return { file_path: filePath };
}

export async function importContactsCsv(filePath: string): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands.contacts, filePathArgs(filePath));
}

export async function exportContactsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.contacts, filePathArgs(filePath));
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

export async function importCsv(
  entity: ImportExportEntity,
  filePath: string,
): Promise<ImportResult> {
  return invoke<ImportResult>(importCommands[entity], filePathArgs(filePath));
}

export async function exportCsv(entity: ImportExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands[entity], filePathArgs(filePath));
}
