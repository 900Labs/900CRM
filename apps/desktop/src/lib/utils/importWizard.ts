import { mapColumns, type ColumnMapping } from './csv';
import type {
  ContactImportTargetField,
  ImportColumnMapping,
  ImportPreflightEntity,
  ImportTargetField,
  OrganizationImportTargetField,
} from '$lib/api/importExport';

export type MappedImportEntity = ImportPreflightEntity;

export interface ImportFieldOption<TTarget extends ImportTargetField = ImportTargetField> {
  value: TTarget;
  label: string;
  required?: boolean;
}

export interface ImportMappingValidation {
  valid: boolean;
  errors: string[];
}

export const CONTACT_IMPORT_FIELDS: ImportFieldOption<ContactImportTargetField>[] = [
  { value: 'first_name', label: 'First name', required: true },
  { value: 'last_name', label: 'Last name' },
  { value: 'org_name', label: 'Organization' },
  { value: 'email', label: 'Email' },
  { value: 'phone', label: 'Phone' },
  { value: 'address', label: 'Address' },
  { value: 'city', label: 'City' },
  { value: 'country', label: 'Country' },
  { value: 'notes', label: 'Notes' },
];

export const ORGANIZATION_IMPORT_FIELDS: ImportFieldOption<OrganizationImportTargetField>[] = [
  { value: 'name', label: 'Name', required: true },
  { value: 'email', label: 'Email' },
  { value: 'phone', label: 'Phone' },
  { value: 'website', label: 'Website' },
  { value: 'address_line1', label: 'Address line 1' },
  { value: 'address_line2', label: 'Address line 2' },
  { value: 'city', label: 'City' },
  { value: 'region', label: 'Region' },
  { value: 'country', label: 'Country' },
  { value: 'postal_code', label: 'Postal code' },
  { value: 'description', label: 'Description' },
];

const CONTACT_ALIASES: Record<string, ContactImportTargetField> = {
  firstname: 'first_name',
  givenname: 'first_name',
  first: 'first_name',
  lastname: 'last_name',
  surname: 'last_name',
  familyname: 'last_name',
  last: 'last_name',
  organization: 'org_name',
  organisation: 'org_name',
  org: 'org_name',
  company: 'org_name',
  companyname: 'org_name',
  emailaddress: 'email',
  email: 'email',
  phone: 'phone',
  phonenumber: 'phone',
  telephone: 'phone',
  mobile: 'phone',
  address: 'address',
  streetaddress: 'address',
  city: 'city',
  country: 'country',
  notes: 'notes',
  note: 'notes',
};

const ORGANIZATION_ALIASES: Record<string, OrganizationImportTargetField> = {
  name: 'name',
  organization: 'name',
  organisation: 'name',
  org: 'name',
  company: 'name',
  companyname: 'name',
  emailaddress: 'email',
  email: 'email',
  phone: 'phone',
  phonenumber: 'phone',
  telephone: 'phone',
  website: 'website',
  web: 'website',
  url: 'website',
  address: 'address_line1',
  address1: 'address_line1',
  addressline1: 'address_line1',
  streetaddress: 'address_line1',
  address2: 'address_line2',
  addressline2: 'address_line2',
  city: 'city',
  region: 'region',
  state: 'region',
  province: 'region',
  country: 'country',
  postcode: 'postal_code',
  zipcode: 'postal_code',
  postalcode: 'postal_code',
  zip: 'postal_code',
  description: 'description',
  notes: 'description',
};

export function getImportFieldOptions(entity: MappedImportEntity): ImportFieldOption[] {
  return entity === 'contacts' ? CONTACT_IMPORT_FIELDS : ORGANIZATION_IMPORT_FIELDS;
}

export function suggestImportMapping(entity: MappedImportEntity, headers: string[]): ColumnMapping {
  const fields = getImportFieldOptions(entity).map((field) => field.value);
  const suggested = mapColumns(headers, fields);
  const aliases = entity === 'contacts' ? CONTACT_ALIASES : ORGANIZATION_ALIASES;

  for (const header of headers) {
    const alias = aliases[normalizeHeader(header)];
    if (alias) {
      suggested[header] = alias;
    }
  }

  return suggested;
}

export function validateImportMapping(
  entity: MappedImportEntity,
  mapping: ColumnMapping,
): ImportMappingValidation {
  const fieldOptions = getImportFieldOptions(entity);
  const allowedFields = new Set<string>(fieldOptions.map((field) => field.value));
  const assigned = new Map<string, string[]>();
  const errors: string[] = [];

  for (const [source, target] of Object.entries(mapping)) {
    if (target === null) {
      continue;
    }

    if (!allowedFields.has(target)) {
      errors.push(`"${source}" maps to an unsupported field.`);
      continue;
    }

    const sources = assigned.get(target) ?? [];
    sources.push(source);
    assigned.set(target, sources);
  }

  for (const field of fieldOptions) {
    if (field.required && !assigned.has(field.value)) {
      errors.push(`${field.label} is required.`);
    }
  }

  for (const [target, sources] of assigned.entries()) {
    if (sources.length > 1) {
      const label = fieldOptions.find((field) => field.value === target)?.label ?? target;
      errors.push(`${label} is mapped more than once: ${sources.join(', ')}.`);
    }
  }

  return { valid: errors.length === 0, errors };
}

export function toBackendMapping<TTarget extends ImportTargetField>(
  mapping: ColumnMapping,
): ImportColumnMapping<TTarget> {
  return Object.fromEntries(
    Object.entries(mapping).map(([source, target]) => [source, target === '' ? null : target]),
  ) as ImportColumnMapping<TTarget>;
}

function normalizeHeader(header: string): string {
  return header.toLowerCase().replace(/[^a-z0-9]/g, '');
}
