import { describe, expect, it } from 'vitest';

import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';
import { contactDisplayName, deriveDealRelationshipLabels } from '$lib/utils/dealRelationships';

function contact(overrides: Partial<Contact>): Contact {
  return {
    id: 'contact-1',
    firstName: 'Amina',
    lastName: 'Khan',
    email: 'amina@example.com',
    phone: null,
    organization: null,
    type: 'person',
    tags: [],
    notes: null,
    website: null,
    address: null,
    createdAt: '2026-06-24T08:00:00Z',
    updatedAt: '2026-06-24T08:00:00Z',
    deletedAt: null,
    ...overrides,
  };
}

function deal(overrides: Partial<Deal>): Deal {
  return {
    id: 'deal-1',
    name: 'Clinic expansion',
    value: 12000,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: null,
    contactId: null,
    organizationId: null,
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-06-24T08:00:00Z',
    updatedAt: '2026-06-24T08:00:00Z',
    ...overrides,
  };
}

const organization: Organization = {
  id: 'org-1',
  name: 'Nairobi Health',
  email: null,
  phone: null,
  website: null,
  addressLine1: null,
  addressLine2: null,
  city: null,
  region: null,
  country: null,
  postalCode: null,
  source: null,
  description: null,
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

describe('deal relationship labels', () => {
  it('formats contact selector labels with stable fallbacks', () => {
    expect(contactDisplayName(contact({ firstName: ' Amina ', lastName: ' Khan ' }))).toBe('Amina Khan');
    expect(contactDisplayName(contact({ firstName: '', lastName: '', email: 'owner@example.com' }))).toBe('owner@example.com');
    expect(contactDisplayName(contact({ id: 'contact-fallback', firstName: '', lastName: '', email: null }))).toBe('contact-fallback');
  });

  it('derives primary contact and organization names from loaded frontend lookups', () => {
    expect(
      deriveDealRelationshipLabels(
        deal({ contactId: 'contact-1', organizationId: 'org-1' }),
        [contact({})],
        [organization],
      ),
    ).toEqual({
      primaryContactName: 'Amina Khan',
      organizationName: 'Nairobi Health',
    });
  });

  it('falls back to backend contactName when the contact lookup is not loaded', () => {
    expect(
      deriveDealRelationshipLabels(
        deal({ contactId: 'missing-contact', contactName: 'Legacy Name' }),
        [],
        [],
      ),
    ).toEqual({
      primaryContactName: 'Legacy Name',
      organizationName: null,
    });
  });
});
