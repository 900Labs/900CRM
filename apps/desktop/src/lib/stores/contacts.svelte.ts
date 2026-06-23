/**
 * src/lib/stores/contacts.ts — Contact state management for 900CRM.
 *
 * All state uses Svelte 5 $state runes.
 *
 * @module stores/contacts
 */

import {
  listContacts,
  createContact,
  updateContact,
  deleteContact,
  searchContacts,
  linkContactToOrganization,
} from '$lib/api/contacts';
import type { Contact, CreateContactPayload, UpdateContactPayload, ListContactsParams } from '$lib/api/contacts';
import { runLoadingAction, runSavingAction, runStoreAction } from './actionRunner';
import { uiStore } from './ui';

const notifier = {
  success: (message: string) => uiStore.toastSuccess(message),
  error: (message: string) => uiStore.toastError(message),
};

// ─────────────────────────────────────────────────────────────────────────────
// ContactStore
// ─────────────────────────────────────────────────────────────────────────────

class ContactStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** Current page of contacts. */
  contacts = $state<Contact[]>([]);

  /** Currently selected/viewed contact. */
  selectedContact = $state<Contact | null>(null);

  /** Total count (for pagination). */
  total = $state<number>(0);

  /** Current page number (1-based). */
  page = $state<number>(1);

  /** Page size. */
  pageSize = $state<number>(50);

  /** Active filter state. */
  filters = $state<ListContactsParams>({
    sortBy: 'name',
    sortDir: 'asc',
    pageSize: 50,
    page: 1,
  });

  /** Whether a load is in progress. */
  isLoading = $state<boolean>(false);

  /** Whether a save is in progress. */
  isSaving = $state<boolean>(false);

  /** Inline search results. */
  searchResults = $state<Contact[]>([]);

  /** Whether a search is in progress. */
  isSearching = $state<boolean>(false);

  // ── Derived ────────────────────────────────────────────────────────────────

  /** Total number of pages. */
  totalPages = $derived(Math.ceil(this.total / this.pageSize));

  /** Whether there are more pages. */
  hasNextPage = $derived(this.page < this.totalPages);

  /** Whether there is a previous page. */
  hasPrevPage = $derived(this.page > 1);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load the contacts list with the current filter/page state.
   */
  async loadContacts(): Promise<void> {
    await runLoadingAction({
      setLoading: (value) => {
        this.isLoading = value;
      },
      notifier,
      errorMessage: 'Failed to load contacts',
      action: async () => {
        const result = await listContacts(this.filters);
        this.contacts = result.contacts;
        this.total = result.total;
        this.page = result.page;
      },
    });
  }

  /**
   * Create a new contact and refresh the list.
   *
   * @param data  Contact creation payload
   * @returns     The created Contact
   */
  async createContact(data: CreateContactPayload): Promise<Contact> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Contact created',
      errorMessage: 'Failed to create contact',
      action: () => createContact(data),
      onSuccess: (contact) => {
        this.contacts = [contact, ...this.contacts];
        if (this.contacts.length > this.pageSize) {
          this.contacts = this.contacts.slice(0, this.pageSize);
        }
        this.total += 1;
      },
    });
  }

  /**
   * Update an existing contact.
   *
   * @param id    Contact UUID
   * @param data  Fields to update
   * @returns     Updated Contact
   */
  async updateContact(id: string, data: UpdateContactPayload): Promise<Contact> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Contact updated',
      errorMessage: 'Failed to update contact',
      action: () => updateContact(id, data),
      onSuccess: (contact) => {
        this.contacts = this.contacts.map((c) => (c.id === id ? contact : c));

        if (this.selectedContact?.id === id) {
          this.selectedContact = contact;
        }
      },
    });
  }

  async linkContactToOrganization(
    contactId: string,
    organizationId: string | null
  ): Promise<Contact> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Contact organization updated',
      errorMessage: 'Failed to link organization',
      action: () => linkContactToOrganization(contactId, organizationId),
      onSuccess: (contact) => {
        this.contacts = this.contacts.map((existing) =>
          existing.id === contactId ? contact : existing
        );
        if (this.selectedContact?.id === contactId) {
          this.selectedContact = contact;
        }
      },
    });
  }

  /**
   * Soft-delete a contact and refresh the list.
   *
   * @param id  Contact UUID
   */
  async deleteContact(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      successMessage: 'Contact deleted',
      errorMessage: 'Failed to delete contact',
      action: () => deleteContact(id),
      onSuccess: () => {
        this.contacts = this.contacts.filter((c) => c.id !== id);

        if (this.selectedContact?.id === id) {
          this.selectedContact = null;
        }

        this.total = Math.max(0, this.total - 1);
      },
    });
  }

  /**
   * Search contacts by query string (used for inline search results).
   *
   * @param query  Search string
   */
  async searchContacts(query: string): Promise<void> {
    if (!query.trim()) {
      this.searchResults = [];
      return;
    }

    this.isSearching = true;
    await runStoreAction({
      notifier,
      busyFlag: (value) => {
        this.isSearching = value;
      },
      errorMessage: 'Failed to search contacts',
      action: async () => {
        this.searchResults = await searchContacts(query);
      },
      onError: () => {
        this.searchResults = [];
      },
    });
  }

  /**
   * Set the selected contact.
   *
   * @param contact  Contact to select, or null to clear
   */
  selectContact(contact: Contact | null): void {
    this.selectedContact = contact;
  }

  /**
   * Update filter/sort/pagination state and reload.
   *
   * @param updates  Partial filter changes
   */
  async setFilters(updates: Partial<ListContactsParams>): Promise<void> {
    this.filters = { ...this.filters, ...updates, page: 1 };
    await this.loadContacts();
  }

  /** Go to the next page. */
  async nextPage(): Promise<void> {
    if (!this.hasNextPage) return;
    this.filters = { ...this.filters, page: this.page + 1 };
    await this.loadContacts();
  }

  /** Go to the previous page. */
  async prevPage(): Promise<void> {
    if (!this.hasPrevPage) return;
    this.filters = { ...this.filters, page: this.page - 1 };
    await this.loadContacts();
  }
}

/** Singleton contacts store. */
export const contactStore = new ContactStore();
