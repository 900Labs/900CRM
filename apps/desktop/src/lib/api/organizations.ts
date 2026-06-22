import { invoke } from '@tauri-apps/api/core';

export interface Organization {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  website: string | null;
  addressLine1: string | null;
  addressLine2: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
  postalCode: string | null;
  source: string | null;
  description: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

export type CreateOrganizationPayload = Omit<
  Organization,
  'id' | 'source' | 'createdAt' | 'updatedAt' | 'deletedAt'
>;
export type UpdateOrganizationPayload = Partial<CreateOrganizationPayload>;

interface BackendOrganization {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  website: string | null;
  address_line1: string | null;
  address_line2: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
  postal_code: string | null;
  source: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

type BackendOrganizationPayload = {
  name?: string;
  email?: string | null;
  phone?: string | null;
  website?: string | null;
  address_line1?: string | null;
  address_line2?: string | null;
  city?: string | null;
  region?: string | null;
  country?: string | null;
  postal_code?: string | null;
  description?: string | null;
};
type OptionalBackendOrganizationKey = Exclude<keyof BackendOrganizationPayload, 'name'>;

function toNullable(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function mapOrganization(organization: BackendOrganization): Organization {
  return {
    id: organization.id,
    name: organization.name,
    email: toNullable(organization.email),
    phone: toNullable(organization.phone),
    website: toNullable(organization.website),
    addressLine1: toNullable(organization.address_line1),
    addressLine2: toNullable(organization.address_line2),
    city: toNullable(organization.city),
    region: toNullable(organization.region),
    country: toNullable(organization.country),
    postalCode: toNullable(organization.postal_code),
    source: toNullable(organization.source),
    description: toNullable(organization.description),
    createdAt: organization.created_at,
    updatedAt: organization.updated_at,
    deletedAt: organization.deleted_at,
  };
}

function normalizeOptionalInput(value: string | null | undefined): string | null | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (value === null) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function toBackendCreatePayload(data: CreateOrganizationPayload): BackendOrganizationPayload {
  return {
    name: data.name,
    email: normalizeOptionalInput(data.email),
    phone: normalizeOptionalInput(data.phone),
    website: normalizeOptionalInput(data.website),
    address_line1: normalizeOptionalInput(data.addressLine1),
    address_line2: normalizeOptionalInput(data.addressLine2),
    city: normalizeOptionalInput(data.city),
    region: normalizeOptionalInput(data.region),
    country: normalizeOptionalInput(data.country),
    postal_code: normalizeOptionalInput(data.postalCode),
    description: normalizeOptionalInput(data.description),
  };
}

function setIfPresent<T extends keyof UpdateOrganizationPayload>(
  target: BackendOrganizationPayload,
  data: UpdateOrganizationPayload,
  frontendKey: T,
  backendKey: OptionalBackendOrganizationKey
) {
  if (Object.prototype.hasOwnProperty.call(data, frontendKey)) {
    target[backendKey] = normalizeOptionalInput(data[frontendKey] as string | null | undefined);
  }
}

function toBackendUpdatePayload(data: UpdateOrganizationPayload): BackendOrganizationPayload {
  const payload: BackendOrganizationPayload = {};
  if (Object.prototype.hasOwnProperty.call(data, 'name')) {
    payload.name = data.name;
  }
  setIfPresent(payload, data, 'email', 'email');
  setIfPresent(payload, data, 'phone', 'phone');
  setIfPresent(payload, data, 'website', 'website');
  setIfPresent(payload, data, 'addressLine1', 'address_line1');
  setIfPresent(payload, data, 'addressLine2', 'address_line2');
  setIfPresent(payload, data, 'city', 'city');
  setIfPresent(payload, data, 'region', 'region');
  setIfPresent(payload, data, 'country', 'country');
  setIfPresent(payload, data, 'postalCode', 'postal_code');
  setIfPresent(payload, data, 'description', 'description');
  return payload;
}

export async function createOrganization(
  data: CreateOrganizationPayload
): Promise<Organization> {
  const organization = await invoke<BackendOrganization>('create_organization', toBackendCreatePayload(data));
  return mapOrganization(organization);
}

export async function getOrganization(id: string): Promise<Organization> {
  const organization = await invoke<BackendOrganization>('get_organization', { id });
  return mapOrganization(organization);
}

export async function listOrganizations(): Promise<Organization[]> {
  const organizations = await invoke<BackendOrganization[]>('list_organizations');
  return organizations.map(mapOrganization);
}

export async function updateOrganization(
  id: string,
  data: UpdateOrganizationPayload
): Promise<Organization> {
  const organization = await invoke<BackendOrganization>('update_organization', {
    id,
    ...toBackendUpdatePayload(data),
  });
  return mapOrganization(organization);
}

export async function deleteOrganization(id: string): Promise<void> {
  await invoke<void>('delete_organization', { id });
}
