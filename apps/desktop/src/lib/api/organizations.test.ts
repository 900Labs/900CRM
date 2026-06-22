import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createOrganization,
  deleteOrganization,
  listOrganizations,
  updateOrganization,
  type CreateOrganizationPayload,
} from './organizations';

type BackendOrganization = {
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
};

function sampleOrganization(
  overrides: Partial<BackendOrganization> = {}
): BackendOrganization {
  return {
    id: 'org-1',
    name: 'Acme Health',
    email: 'hello@acme.example',
    phone: '+123456',
    website: 'https://acme.example',
    address_line1: 'Dock 4',
    address_line2: null,
    city: 'Lagos',
    region: null,
    country: 'NG',
    postal_code: null,
    source: 'desktop',
    description: 'Regional partner',
    created_at: '2026-06-01T00:00:00Z',
    updated_at: '2026-06-02T00:00:00Z',
    deleted_at: null,
    ...overrides,
  };
}

describe('organizations api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createOrganization payload to backend fields', async () => {
    const payload: CreateOrganizationPayload = {
      name: 'Acme Health',
      email: 'hello@acme.example',
      phone: '+123456',
      website: 'https://acme.example',
      addressLine1: 'Dock 4',
      addressLine2: null,
      city: 'Lagos',
      region: null,
      country: 'NG',
      postalCode: null,
      description: 'Regional partner',
    };
    invokeMock.mockResolvedValue(sampleOrganization());

    const organization = await createOrganization(payload);

    expect(invokeMock).toHaveBeenCalledWith('create_organization', {
      name: 'Acme Health',
      email: 'hello@acme.example',
      phone: '+123456',
      website: 'https://acme.example',
      address_line1: 'Dock 4',
      address_line2: null,
      city: 'Lagos',
      region: null,
      country: 'NG',
      postal_code: null,
      description: 'Regional partner',
    });
    expect(organization.name).toBe('Acme Health');
  });

  it('maps listOrganizations response', async () => {
    invokeMock.mockResolvedValue([
      sampleOrganization(),
      sampleOrganization({ id: 'org-2', name: 'Beta Clinic', email: '' }),
    ]);

    const organizations = await listOrganizations();

    expect(invokeMock).toHaveBeenCalledWith('list_organizations');
    expect(organizations).toHaveLength(2);
    expect(organizations[1]?.email).toBeNull();
  });

  it('maps updateOrganization and deleteOrganization', async () => {
    invokeMock.mockResolvedValue(sampleOrganization({ name: 'Acme Group' }));

    const organization = await updateOrganization('org-1', {
      name: 'Acme Group',
      description: 'Updated',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_organization', {
      id: 'org-1',
      name: 'Acme Group',
      description: 'Updated',
    });
    expect(organization.name).toBe('Acme Group');

    invokeMock.mockResolvedValue(undefined);
    await deleteOrganization('org-1');
    expect(invokeMock).toHaveBeenLastCalledWith('delete_organization', { id: 'org-1' });
  });

  it('maps explicit null and blank update fields as clears', async () => {
    invokeMock.mockResolvedValue(
      sampleOrganization({
        email: null,
        phone: null,
        website: null,
        city: null,
        country: null,
        description: null,
      })
    );

    const organization = await updateOrganization('org-1', {
      email: null,
      phone: '',
      website: '   ',
      city: null,
      country: '',
      description: '',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_organization', {
      id: 'org-1',
      email: null,
      phone: null,
      website: null,
      city: null,
      country: null,
      description: null,
    });
    expect(organization.email).toBeNull();
    expect(organization.description).toBeNull();
  });
});
