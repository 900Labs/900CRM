/**
 * src/lib/api/contacts.ts — Tauri IPC wrappers for contact commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ContactType = 'person' | 'org';

export interface Contact {
  id: string;
  firstName: string;
  lastName: string;
  email: string | null;
  phone: string | null;
  organization: string | null;
  organizationId: string | null;
  type: ContactType;
  tags: string[];
  notes: string | null;
  website: string | null;
  address: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

export type CreateContactPayload = Omit<
  Contact,
  'id' | 'createdAt' | 'updatedAt' | 'deletedAt' | 'organizationId'
> & {
  organizationId?: string | null;
};
export type UpdateContactPayload = Partial<CreateContactPayload>;

export interface ListContactsParams {
  search?: string;
  type?: ContactType;
  tags?: string[];
  sortBy?: 'name' | 'createdAt' | 'updatedAt';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

export interface ContactListResponse {
  contacts: Contact[];
  total: number;
  page: number;
  pageSize: number;
}

interface BackendContact {
  id: string;
  contact_type: 'person' | 'organization' | string;
  first_name: string;
  last_name: string;
  org_name: string;
  email: string;
  phone: string;
  address: string;
  city: string;
  country: string;
  org_id: string | null;
  organization_id: string | null;
  notes: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

interface BackendContactListResponse {
  contacts: BackendContact[];
  total: number;
  page: number;
  per_page: number;
}

function toNullable(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function mapContact(contact: BackendContact): Contact {
  const addressParts = [contact.address, contact.city, contact.country]
    .map((part) => part?.trim())
    .filter((part): part is string => Boolean(part));

  return {
    id: contact.id,
    firstName: contact.first_name,
    lastName: contact.last_name,
    email: toNullable(contact.email),
    phone: toNullable(contact.phone),
    organization: toNullable(contact.org_name),
    organizationId: contact.organization_id ?? contact.org_id ?? null,
    type: contact.contact_type === 'organization' ? 'org' : 'person',
    tags: [],
    notes: toNullable(contact.notes),
    website: null,
    address: addressParts.length > 0 ? addressParts.join(', ') : null,
    createdAt: contact.created_at,
    updatedAt: contact.updated_at,
    deletedAt: contact.deleted_at,
  };
}

function toBackendSortKey(sortBy: ListContactsParams['sortBy']): string {
  switch (sortBy) {
    case 'createdAt':
      return 'created_at';
    case 'updatedAt':
      return 'updated_at';
    case 'name':
    default:
      return 'first_name';
  }
}

function toBackendContactType(type: ContactType | undefined): string | undefined {
  if (!type) {
    return undefined;
  }
  return type === 'org' ? 'organization' : 'person';
}

export async function createContact(data: CreateContactPayload): Promise<Contact> {
  const contact = await invoke<BackendContact>('create_contact', {
    contact_type: data.type === 'org' ? 'organization' : 'person',
    first_name: data.firstName ?? '',
    last_name: data.lastName ?? '',
    org_name: data.organization ?? '',
    email: data.email ?? '',
    phone: data.phone ?? '',
    address: data.address ?? '',
    city: '',
    country: '',
    org_id: data.organizationId ?? null,
    notes: data.notes ?? '',
  });

  return mapContact(contact);
}

export async function getContact(id: string): Promise<Contact> {
  const contact = await invoke<BackendContact>('get_contact', { id });
  return mapContact(contact);
}

export async function listContacts(params: ListContactsParams = {}): Promise<ContactListResponse> {
  const result = await invoke<BackendContactListResponse>('list_contacts', {
    params: {
      page: params.page ?? 1,
      per_page: params.pageSize ?? 50,
      sort_by: toBackendSortKey(params.sortBy),
      sort_dir: params.sortDir ?? 'asc',
      filter_type: toBackendContactType(params.type),
      search_query: params.search?.trim() ? params.search.trim() : undefined,
    },
  });

  return {
    contacts: result.contacts.map(mapContact),
    total: result.total,
    page: result.page,
    pageSize: result.per_page,
  };
}

export async function updateContact(id: string, data: UpdateContactPayload): Promise<Contact> {
  const contact = await invoke<BackendContact>('update_contact', {
    id,
    contact_type: data.type ? (data.type === 'org' ? 'organization' : 'person') : undefined,
    first_name: data.firstName,
    last_name: data.lastName,
    org_name: data.organization,
    email: data.email ?? undefined,
    phone: data.phone ?? undefined,
    address: data.address ?? undefined,
    city: undefined,
    country: undefined,
    notes: data.notes ?? undefined,
  });

  return mapContact(contact);
}

export async function deleteContact(id: string): Promise<void> {
  await invoke<void>('delete_contact', { id });
}

export async function searchContacts(query: string): Promise<Contact[]> {
  const contacts = await invoke<BackendContact[]>('search_contacts', { query });
  return contacts.map(mapContact);
}

export async function mergeContacts(sourceId: string, targetId: string): Promise<Contact> {
  const contact = await invoke<BackendContact>('merge_contacts', {
    target_id: targetId,
    source_id: sourceId,
  });
  return mapContact(contact);
}

export async function linkContactToOrganization(
  contactId: string,
  organizationId: string | null
): Promise<Contact> {
  const contact = await invoke<BackendContact>('link_contact_to_organization', {
    contact_id: contactId,
    organization_id: organizationId,
  });

  return mapContact(contact);
}
