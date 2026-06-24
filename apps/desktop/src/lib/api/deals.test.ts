import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  addDealContact,
  createDeal,
  linkDealToOrganization,
  listDealContacts,
  removeDealContact,
  updateDeal,
} from './deals';

const backendDeal = {
  id: 'deal-1',
  title: 'Clinic expansion',
  value: 12000,
  currency: 'USD',
  stage: 'Proposal',
  probability: 50,
  expected_close: '2026-07-15',
  contact_id: 'contact-1',
  organization_id: 'org-1',
  notes: 'Regional rollout',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T09:00:00Z',
};

const backendDealContact = {
  id: 'deal-contact-1',
  deal_id: 'deal-1',
  contact_id: 'contact-1',
  role: 'Decision maker',
  is_primary: true,
  created_at: '2026-06-24T08:30:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

describe('deal API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createDeal organization_id and backend organization_id', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await expect(
      createDeal({
        name: 'Clinic expansion',
        value: 12000,
        currency: 'usd',
        stage: 'proposal',
        probability: 50,
        expectedCloseDate: '2026-07-15',
        contactId: 'contact-1',
        organizationId: ' org-1 ',
        description: 'Regional rollout',
        tags: [],
      }),
    ).resolves.toMatchObject({
      id: 'deal-1',
      name: 'Clinic expansion',
      organizationId: 'org-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('create_deal', {
      title: 'Clinic expansion',
      value: 12000,
      currency: 'usd',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-07-15',
      contact_id: 'contact-1',
      organization_id: 'org-1',
      notes: 'Regional rollout',
    });
  });

  it('maps updateDeal organization_id only when supplied', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await updateDeal('deal-1', { name: 'Clinic expansion' });

    expect(invokeMock).toHaveBeenCalledWith('update_deal', {
      id: 'deal-1',
      title: 'Clinic expansion',
      value: undefined,
      currency: undefined,
      stage: undefined,
      probability: undefined,
      expected_close: undefined,
      contact_id: undefined,
      notes: undefined,
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({ ...backendDeal, organization_id: null });

    await updateDeal('deal-1', { organizationId: '   ' });

    expect(invokeMock).toHaveBeenCalledWith(
      'update_deal',
      expect.objectContaining({
        id: 'deal-1',
        organization_id: null,
      }),
    );
  });

  it('maps linkDealToOrganization to link_deal_to_organization', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await expect(linkDealToOrganization('deal-1', ' org-1 ')).resolves.toMatchObject({
      id: 'deal-1',
      organizationId: 'org-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('link_deal_to_organization', {
      deal_id: 'deal-1',
      organization_id: 'org-1',
    });
  });

  it('maps add/list/remove deal contact commands', async () => {
    invokeMock.mockResolvedValueOnce(backendDealContact);

    await expect(
      addDealContact('deal-1', 'contact-1', {
        role: ' Decision maker ',
        isPrimary: true,
      }),
    ).resolves.toEqual({
      id: 'deal-contact-1',
      dealId: 'deal-1',
      contactId: 'contact-1',
      role: 'Decision maker',
      isPrimary: true,
      createdAt: '2026-06-24T08:30:00Z',
      deletedAt: null,
      deviceId: 'device-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('add_deal_contact', {
      deal_id: 'deal-1',
      contact_id: 'contact-1',
      role: 'Decision maker',
      is_primary: true,
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce([backendDealContact]);

    await expect(listDealContacts('deal-1')).resolves.toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith('list_deal_contacts', {
      deal_id: 'deal-1',
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({ ...backendDealContact, deleted_at: '2026-06-24T10:00:00Z' });

    await expect(removeDealContact('deal-1', 'contact-1')).resolves.toMatchObject({
      deletedAt: '2026-06-24T10:00:00Z',
    });
    expect(invokeMock).toHaveBeenCalledWith('remove_deal_contact', {
      deal_id: 'deal-1',
      contact_id: 'contact-1',
    });
  });
});
