import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  listContacts,
  listContactDuplicateCandidates,
  mergeContacts,
} from './contacts';

const backendContact = {
  id: 'contact-target',
  contact_type: 'person',
  first_name: 'Ada',
  last_name: 'Lovelace',
  org_name: '',
  email: 'ada@example.com',
  phone: '+15550100',
  address: '',
  city: '',
  country: '',
  org_id: null,
  notes: '',
  created_at: '2026-06-24T10:00:00Z',
  updated_at: '2026-06-24T10:00:00Z',
  deleted_at: null,
};

describe('contacts API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps contact duplicate candidate commands', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        source_id: 'contact-source',
        source_display_label: 'Ada Source',
        target_id: 'contact-target',
        target_display_label: 'Ada Target',
        match_type: 'email',
        matched_value: 'ada@example.com',
        reason: 'Same email address: ada@example.com',
      },
    ]);

    await expect(listContactDuplicateCandidates()).resolves.toEqual([
      {
        sourceId: 'contact-source',
        sourceDisplayLabel: 'Ada Source',
        targetId: 'contact-target',
        targetDisplayLabel: 'Ada Target',
        matchType: 'email',
        matchedValue: 'ada@example.com',
        reason: 'Same email address: ada@example.com',
      },
    ]);
    expect(invokeMock).toHaveBeenCalledWith('list_contact_duplicate_candidates');
  });

  it('maps merge contact source and target ids', async () => {
    invokeMock.mockResolvedValueOnce(backendContact);

    await expect(mergeContacts('contact-source', 'contact-target')).resolves.toMatchObject({
      id: 'contact-target',
      firstName: 'Ada',
      lastName: 'Lovelace',
      email: 'ada@example.com',
    });
    expect(invokeMock).toHaveBeenCalledWith('merge_contacts', {
      source_id: 'contact-source',
      target_id: 'contact-target',
    });
  });

  it('maps normalized organization id from contact list responses', async () => {
    invokeMock.mockResolvedValueOnce({
      contacts: [
        {
          ...backendContact,
          id: 'contact-linked',
          organization_id: 'organization-1',
        },
      ],
      total: 1,
      page: 1,
      per_page: 50,
    });

    await expect(listContacts()).resolves.toMatchObject({
      contacts: [
        {
          id: 'contact-linked',
          organizationId: 'organization-1',
        },
      ],
    });
  });
});
