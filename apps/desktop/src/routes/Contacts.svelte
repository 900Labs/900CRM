<script lang="ts">
  /**
   * Contacts.svelte — Contacts list view for 900CRM.
   *
   * Features: DataTable with search, type filter, sort, pagination.
   * Toolbar with Add Contact, Import/Export buttons.
   * Click row to navigate to ContactDetail.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { contactStore } from '$lib/stores/contacts';
  import { uiStore } from '$lib/stores/ui';
  import type { Contact } from '$lib/api/contacts';
  import type { Column } from '$lib/components/DataTable.svelte';
  import { formatFullName, formatDate } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';
  import DataTable from '$lib/components/DataTable.svelte';
  import ImportExport from '$lib/components/ImportExport.svelte';

  // ── State ───────────────────────────────────────────────────────────────────

  let searchQuery = $state('');
  let typeFilter = $state<'' | 'person' | 'org'>('');
  let showImportExport = $state(false);
  let selectedContact = $state<Contact | null>(null);
  let searchTimer: ReturnType<typeof setTimeout> | undefined;

  // ── Column definitions ─────────────────────────────────────────────────────

  const columns: Column<Contact>[] = [
    {
      key: 'name',
      label: t('contacts.name'),
      sortable: true,
      render: (c) => formatFullName((c as Contact).firstName, (c as Contact).lastName),
    },
    {
      key: 'organization',
      label: t('contacts.organization'),
      sortable: true,
      render: (c) => (c as Contact).organization ?? '—',
    },
    {
      key: 'email',
      label: t('contacts.email'),
      render: (c) => (c as Contact).email ?? '—',
    },
    {
      key: 'phone',
      label: t('contacts.phone'),
      render: (c) => (c as Contact).phone ?? '—',
    },
    {
      key: 'type',
      label: t('contacts.type'),
      sortable: true,
      render: (c) => t(`contacts.${(c as Contact).type}`),
    },
    {
      key: 'createdAt',
      label: t('contacts.created'),
      sortable: true,
      render: (c) => formatDate((c as Contact).createdAt, settingsStore.dateFormat as 'MMM D, YYYY'),
    },
  ];

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(async () => {
    await contactStore.loadContacts();
  });

  // ── Handlers ───────────────────────────────────────────────────────────────

  function handleSearchInput(e: Event) {
    searchQuery = (e.target as HTMLInputElement).value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      contactStore.setFilters({ search: searchQuery, page: 1 });
    }, 300);
  }

  async function handleTypeFilter(type: '' | 'person' | 'org') {
    typeFilter = type;
    await contactStore.setFilters({ type: type || undefined, page: 1 });
  }

  function handleRowClick(row: unknown) {
    const contact = row as Contact;
    contactStore.selectContact(contact);
    selectedContact = contact;
    // Navigate to detail view (hash routing)
    window.location.hash = `/contacts/${contact.id}`;
  }

  async function handleSort(key: string, dir: 'asc' | 'desc') {
    await contactStore.setFilters({
      sortBy: key as 'name' | 'createdAt' | 'updatedAt',
      sortDir: dir,
    });
  }
</script>

<div class="page-content contacts-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('contacts.title')}</h1>
    <div class="toolbar">
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addContact')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('contacts.addContact')}
      </button>
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => showImportExport = true}
        type="button"
      >
        {t('common.import')} / {t('common.export')}
      </button>
    </div>
  </div>

  <!-- Filters -->
  <div class="contacts-filters">
    <div class="search-wrap">
      <svg class="filter-search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        class="input filter-search selectable"
        type="search"
        placeholder={t('contacts.search')}
        value={searchQuery}
        oninput={handleSearchInput}
        aria-label={t('contacts.search')}
      />
    </div>

    <div class="type-filters" role="group" aria-label={t('contacts.type')}>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'person', label: t('contacts.person') },
        { value: 'org', label: t('contacts.org') },
      ] as option (option.value)}
        <button
          class="filter-chip"
          class:active={typeFilter === option.value}
          onclick={() => handleTypeFilter(option.value as '' | 'person' | 'org')}
          type="button"
        >
          {option.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- Table -->
  <div class="contacts-table card">
    <DataTable
      columns={columns as Column[]}
      rows={contactStore.contacts}
      loading={contactStore.isLoading}
      total={contactStore.total}
      page={contactStore.page}
      pageSize={contactStore.pageSize}
      emptyTitle={t('contacts.noContacts')}
      emptyDescription={t('contacts.noContactsDesc')}
      emptyIcon="contacts"
      emptyActionLabel={t('contacts.addContact')}
      onemptyaction={() => uiStore.openModal('addContact')}
      onrowclick={handleRowClick}
      onnextpage={() => contactStore.nextPage()}
      onprevpage={() => contactStore.prevPage()}
    />
  </div>
</div>

<!-- Import/Export Modal -->
<ImportExport bind:open={showImportExport} />

<style>
  .contacts-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .contacts-filters {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    flex-wrap: wrap;
  }

  .search-wrap {
    position: relative;
    flex: 1;
    min-width: 200px;
    max-width: 360px;
  }

  .filter-search-icon {
    position: absolute;
    inset-inline-start: var(--space-4);
    top: 50%;
    transform: translateY(-50%);
    color: var(--icon-muted);
    pointer-events: none;
  }

  .filter-search {
    padding-inline-start: var(--space-10);
    height: 34px;
  }

  .filter-search::-webkit-search-cancel-button {
    display: none;
  }

  .type-filters {
    display: flex;
    gap: var(--space-2);
  }

  .filter-chip {
    padding: var(--space-2) var(--space-4);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background-color: transparent;
    border: var(--border-width) solid var(--border-default);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .filter-chip.active {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .contacts-table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
