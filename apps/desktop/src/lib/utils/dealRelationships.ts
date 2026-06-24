import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';

export interface DealRelationshipLabels {
  primaryContactName: string | null;
  organizationName: string | null;
}

export function contactDisplayName(contact: Pick<Contact, 'id' | 'firstName' | 'lastName' | 'email'>): string {
  const fullName = [contact.firstName, contact.lastName]
    .map((part) => part.trim())
    .filter(Boolean)
    .join(' ');

  return fullName || contact.email?.trim() || contact.id;
}

export function deriveDealRelationshipLabels(
  deal: Pick<Deal, 'contactId' | 'contactName' | 'organizationId'>,
  contacts: Pick<Contact, 'id' | 'firstName' | 'lastName' | 'email'>[],
  organizations: Pick<Organization, 'id' | 'name'>[],
): DealRelationshipLabels {
  const contact = deal.contactId
    ? contacts.find((candidate) => candidate.id === deal.contactId)
    : null;
  const organization = deal.organizationId
    ? organizations.find((candidate) => candidate.id === deal.organizationId)
    : null;

  return {
    primaryContactName: contact ? contactDisplayName(contact) : deal.contactName,
    organizationName: organization?.name.trim() || null,
  };
}
