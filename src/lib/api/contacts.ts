/**
 * src/lib/api/contacts.ts — Tauri IPC wrappers for the contacts backend.
 *
 * All functions use invoke() from @tauri-apps/api/core. They are fully typed
 * and throw on error (caller wraps in try/catch + toast).
 *
 * @module api/contacts
 */

import { invoke } from '@tauri-apps/api/core';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Contact type discriminant. */
export type ContactType = 'person' | 'org';

/** A CRM contact record. */
export interface Contact {
  id: string;
  firstName: string;
  lastName: string;
  email: string | null;
  phone: string | null;
  organization: string | null;
  type: ContactType;
  tags: string[];
  notes: string | null;
  website: string | null;
  address: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

/** Payload for creating a new contact. */
export type CreateContactPayload = Omit<Contact, 'id' | 'createdAt' | 'updatedAt' | 'deletedAt'>;

/** Payload for updating a contact (all fields optional). */
export type UpdateContactPayload = Partial<CreateContactPayload>;

/** Parameters for listing contacts. */
export interface ListContactsParams {
  search?: string;
  type?: ContactType;
  tags?: string[];
  sortBy?: 'name' | 'createdAt' | 'updatedAt';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

/** Paginated list response. */
export interface ContactListResponse {
  contacts: Contact[];
  total: number;
  page: number;
  pageSize: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// API functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Create a new contact.
 *
 * @param data  Contact creation payload
 * @returns     The created Contact
 */
export async function createContact(data: CreateContactPayload): Promise<Contact> {
  return invoke<Contact>('create_contact', { data });
}

/**
 * Fetch a single contact by ID.
 *
 * @param id  Contact UUID
 * @returns   Contact or throws if not found
 */
export async function getContact(id: string): Promise<Contact> {
  return invoke<Contact>('get_contact', { id });
}

/**
 * List contacts with optional filtering, sorting, and pagination.
 *
 * @param params  Query parameters
 * @returns       Paginated ContactListResponse
 */
export async function listContacts(params: ListContactsParams = {}): Promise<ContactListResponse> {
  return invoke<ContactListResponse>('list_contacts', { params });
}

/**
 * Update a contact by ID.
 *
 * @param id    Contact UUID
 * @param data  Fields to update
 * @returns     Updated Contact
 */
export async function updateContact(id: string, data: UpdateContactPayload): Promise<Contact> {
  return invoke<Contact>('update_contact', { id, data });
}

/**
 * Soft-delete a contact by ID.
 *
 * @param id  Contact UUID
 */
export async function deleteContact(id: string): Promise<void> {
  return invoke<void>('delete_contact', { id });
}

/**
 * Full-text search across contact fields.
 *
 * @param query  Search string
 * @returns      Array of matching contacts (up to 20)
 */
export async function searchContacts(query: string): Promise<Contact[]> {
  return invoke<Contact[]>('search_contacts', { query });
}

/**
 * Merge two contacts: moves all linked deals/activities from sourceId to
 * targetId, then soft-deletes sourceId.
 *
 * @param sourceId  The contact to merge from (will be deleted)
 * @param targetId  The contact to merge into (will be kept)
 * @returns         The updated target Contact
 */
export async function mergeContacts(sourceId: string, targetId: string): Promise<Contact> {
  return invoke<Contact>('merge_contacts', { sourceId, targetId });
}
