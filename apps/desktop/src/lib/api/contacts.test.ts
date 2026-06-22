import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createContact,
  deleteContact,
  getContact,
  listContacts,
  mergeContacts,
  searchContacts,
  linkContactToOrganization,
  updateContact,
  type CreateContactPayload,
} from './contacts';

type BackendContact = {
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
};

function sampleContact(overrides: Partial<BackendContact> = {}): BackendContact {
  return {
    id: 'contact-1',
    contact_type: 'person',
    first_name: 'Amina',
    last_name: 'Diallo',
    org_name: '',
    email: 'amina@example.com',
    phone: '+123456',
    address: '123 Main',
    city: 'Lagos',
    country: 'NG',
    org_id: null,
    organization_id: null,
    notes: 'Important lead',
    created_at: '2026-03-01T00:00:00.000Z',
    updated_at: '2026-03-02T00:00:00.000Z',
    deleted_at: null,
    ...overrides,
  };
}

describe('contacts api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createContact payload to backend fields and maps response', async () => {
    const payload: CreateContactPayload = {
      firstName: 'Sara',
      lastName: 'Mills',
      email: 'sara@example.com',
      phone: '555-0100',
      organization: 'Acme',
      type: 'org',
      tags: [],
      notes: 'Prospect',
      website: null,
      address: 'Dock 4',
    };

    invokeMock.mockResolvedValue(
      sampleContact({
        contact_type: 'organization',
        first_name: payload.firstName,
        last_name: payload.lastName,
        org_name: payload.organization ?? '',
        email: payload.email ?? '',
        phone: payload.phone ?? '',
        address: payload.address ?? '',
      })
    );

    const contact = await createContact(payload);

    expect(invokeMock).toHaveBeenCalledWith('create_contact', {
      contact_type: 'organization',
      first_name: 'Sara',
      last_name: 'Mills',
      org_name: 'Acme',
      email: 'sara@example.com',
      phone: '555-0100',
      address: 'Dock 4',
      city: '',
      country: '',
      org_id: null,
      notes: 'Prospect',
    });
    expect(contact.type).toBe('org');
    expect(contact.organization).toBe('Acme');
  });

  it('maps getContact nullable and address fields correctly', async () => {
    invokeMock.mockResolvedValue(
      sampleContact({
        email: '  ',
        phone: '',
        notes: '   ',
        address: 'Line 1',
        city: 'Paris',
        country: 'FR',
      })
    );

    const contact = await getContact('contact-1');

    expect(invokeMock).toHaveBeenCalledWith('get_contact', { id: 'contact-1' });
    expect(contact.email).toBeNull();
    expect(contact.phone).toBeNull();
    expect(contact.notes).toBeNull();
    expect(contact.address).toBe('Line 1, Paris, FR');
  });

  it('maps listContacts params and pagination shape', async () => {
    invokeMock.mockResolvedValue({
      contacts: [sampleContact(), sampleContact({ id: 'contact-2', first_name: 'Benoit' })],
      total: 2,
      page: 2,
      per_page: 25,
    });

    const result = await listContacts({
      search: '  diallo  ',
      type: 'org',
      sortBy: 'updatedAt',
      sortDir: 'desc',
      page: 2,
      pageSize: 25,
    });

    expect(invokeMock).toHaveBeenCalledWith('list_contacts', {
      params: {
        page: 2,
        per_page: 25,
        sort_by: 'updated_at',
        sort_dir: 'desc',
        filter_type: 'organization',
        search_query: 'diallo',
      },
    });
    expect(result.pageSize).toBe(25);
    expect(result.total).toBe(2);
    expect(result.contacts).toHaveLength(2);
  });

  it('maps updateContact partial payload fields', async () => {
    invokeMock.mockResolvedValue(sampleContact({ contact_type: 'organization' }));

    await updateContact('contact-7', {
      type: 'org',
      firstName: 'Updated',
      notes: 'Keep warm',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_contact', {
      id: 'contact-7',
      contact_type: 'organization',
      first_name: 'Updated',
      last_name: undefined,
      org_name: undefined,
      email: undefined,
      phone: undefined,
      address: undefined,
      city: undefined,
      country: undefined,
      notes: 'Keep warm',
    });
  });

  it('uses delete_contact command', async () => {
    invokeMock.mockResolvedValue(undefined);

    await deleteContact('contact-9');
    expect(invokeMock).toHaveBeenCalledWith('delete_contact', { id: 'contact-9' });
  });

  it('searchContacts maps backend response list', async () => {
    invokeMock.mockResolvedValue([sampleContact(), sampleContact({ id: 'contact-2' })]);

    const result = await searchContacts('amina');

    expect(invokeMock).toHaveBeenCalledWith('search_contacts', { query: 'amina' });
    expect(result).toHaveLength(2);
  });

  it('mergeContacts sends target/source ids and maps result', async () => {
    invokeMock.mockResolvedValue(sampleContact({ id: 'contact-merged' }));

    const contact = await mergeContacts('contact-a', 'contact-b');

    expect(invokeMock).toHaveBeenCalledWith('merge_contacts', {
      target_id: 'contact-b',
      source_id: 'contact-a',
    });
    expect(contact.id).toBe('contact-merged');
  });

  it('links a contact to a normalized organization', async () => {
    invokeMock.mockResolvedValue(
      sampleContact({ organization_id: 'org-1', org_name: 'Acme Health' })
    );

    const contact = await linkContactToOrganization('contact-1', 'org-1');

    expect(invokeMock).toHaveBeenCalledWith('link_contact_to_organization', {
      contact_id: 'contact-1',
      organization_id: 'org-1',
    });
    expect(contact.organizationId).toBe('org-1');
    expect(contact.organization).toBe('Acme Health');
  });
});
