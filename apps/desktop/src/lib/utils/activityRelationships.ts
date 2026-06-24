import type { Activity, ActivityLink } from '$lib/api/activities';
import { addActivityLink, listActivityLinks } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import { listContacts } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import { listDeals } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';
import { listOrganizations } from '$lib/api/organizations';
import { contactDisplayName } from '$lib/utils/dealRelationships';

export const ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE = 500;

export interface ActivityRelationshipLookups {
  contacts: Contact[];
  organizations: Organization[];
  deals: Deal[];
}

export interface ActivityRelationshipItem {
  id: string;
  label: string;
}

export interface ActivityRelationshipLabels {
  contacts: ActivityRelationshipItem[];
  organizations: ActivityRelationshipItem[];
  deals: ActivityRelationshipItem[];
}

export interface ActivityRelationshipSelection {
  contactId?: string | null;
  organizationId?: string | null;
  dealId?: string | null;
}

export type ActivityLinkIndex = Record<string, ActivityLink[]>;

function uniqueIds(ids: (string | null | undefined)[]): string[] {
  return Array.from(
    new Set(
      ids
        .map((id) => id?.trim())
        .filter((id): id is string => Boolean(id))
    )
  );
}

function linkedIds(
  links: ActivityLink[],
  entityType: ActivityLink['entityType'],
): string[] {
  return links
    .filter((link) => link.entityType === entityType && !link.deletedAt)
    .map((link) => link.entityId);
}

export function deriveActivityRelationshipLabels(
  activity: Pick<Activity, 'contactId' | 'contactName' | 'dealId' | 'dealName'>,
  links: ActivityLink[],
  lookups: ActivityRelationshipLookups,
): ActivityRelationshipLabels {
  const contactIds = uniqueIds([...linkedIds(links, 'contact'), activity.contactId]);
  const organizationIds = uniqueIds(linkedIds(links, 'organization'));
  const dealIds = uniqueIds([...linkedIds(links, 'deal'), activity.dealId]);

  return {
    contacts: contactIds.map((id) => {
      const contact = lookups.contacts.find((candidate) => candidate.id === id);
      return {
        id,
        label: contact ? contactDisplayName(contact) : activity.contactId === id ? activity.contactName ?? id : id,
      };
    }),
    organizations: organizationIds.map((id) => {
      const organization = lookups.organizations.find((candidate) => candidate.id === id);
      return {
        id,
        label: organization?.name.trim() || id,
      };
    }),
    deals: dealIds.map((id) => {
      const deal = lookups.deals.find((candidate) => candidate.id === id);
      return {
        id,
        label: deal?.name.trim() || (activity.dealId === id ? activity.dealName ?? id : id),
      };
    }),
  };
}

export async function loadActivityRelationshipContacts(): Promise<Contact[]> {
  const contacts: Contact[] = [];
  let page = 1;
  let total = 0;

  do {
    const result = await listContacts({
      page,
      pageSize: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
      sortBy: 'name',
      sortDir: 'asc',
    });

    contacts.push(...result.contacts);
    total = result.total;

    if (result.contacts.length === 0) {
      break;
    }

    page += 1;
  } while (contacts.length < total);

  return contacts;
}

export async function loadActivityRelationshipLookups(): Promise<ActivityRelationshipLookups> {
  const [contacts, organizations, deals] = await Promise.all([
    loadActivityRelationshipContacts(),
    listOrganizations(),
    listDeals({ sortBy: 'name', sortDir: 'asc' }),
  ]);

  return { contacts, organizations, deals };
}

export async function loadActivityLinkIndex(activityIds: string[]): Promise<ActivityLinkIndex> {
  const uniqueActivityIds = uniqueIds(activityIds);
  const entries = await Promise.all(
    uniqueActivityIds.map(async (activityId) => [
      activityId,
      await listActivityLinks(activityId),
    ] as const)
  );

  return Object.fromEntries(entries);
}

export async function addSelectedActivityLinks(
  activityId: string,
  selection: ActivityRelationshipSelection,
): Promise<void> {
  const additions = [
    selection.contactId ? addActivityLink(activityId, 'contact', selection.contactId) : null,
    selection.organizationId ? addActivityLink(activityId, 'organization', selection.organizationId) : null,
    selection.dealId ? addActivityLink(activityId, 'deal', selection.dealId) : null,
  ].filter((addition): addition is Promise<ActivityLink> => addition !== null);

  if (additions.length === 0) {
    return;
  }

  await Promise.all(additions);
}
