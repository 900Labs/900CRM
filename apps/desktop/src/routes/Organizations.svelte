<script lang="ts">
  /**
   * Organizations.svelte - First-class organization list and command surface.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { organizationStore } from '$lib/stores/organizations';
  import { uiStore } from '$lib/stores/ui';
  import type { Organization } from '$lib/api/organizations';
  import { formatDate } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';
  import Modal from '$lib/components/Modal.svelte';
  import EntityNotesPanel from '$lib/components/EntityNotesPanel.svelte';
  import EntityTagsPanel from '$lib/components/EntityTagsPanel.svelte';

  interface OrganizationFormState {
    name: string;
    email: string;
    phone: string;
    website: string;
    addressLine1: string;
    addressLine2: string;
    city: string;
    region: string;
    country: string;
    postalCode: string;
    description: string;
  }

  let searchQuery = $state('');
  let formOpen = $state(false);
  let linkOpen = $state(false);
  let notesTagsOpen = $state(false);
  let editingOrganization = $state<Organization | null>(null);
  let linkOrganization = $state<Organization | null>(null);
  let notesTagsOrganization = $state<Organization | null>(null);
  let linkContactId = $state('');
  let form = $state<OrganizationFormState>(emptyOrganizationForm());

  const filteredOrganizations = $derived(
    organizationStore.organizations.filter((organization) => organizationMatches(organization, searchQuery)),
  );

  onMount(async () => {
    await organizationStore.loadOrganizations();
  });

  function emptyOrganizationForm(): OrganizationFormState {
    return {
      name: '',
      email: '',
      phone: '',
      website: '',
      addressLine1: '',
      addressLine2: '',
      city: '',
      region: '',
      country: '',
      postalCode: '',
      description: '',
    };
  }

  function organizationMatches(organization: Organization, query: string): boolean {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return true;

    return [
      organization.name,
      organization.email,
      organization.phone,
      organization.website,
      organization.city,
      organization.region,
      organization.country,
      organization.description,
    ]
      .filter((value): value is string => Boolean(value))
      .some((value) => value.toLowerCase().includes(normalized));
  }

  function organizationLocation(organization: Organization): string {
    const parts = [organization.city, organization.region, organization.country]
      .map((part) => part?.trim())
      .filter((part): part is string => Boolean(part));

    return parts.length > 0 ? parts.join(', ') : '—';
  }

  function blankToNull(value: string): string | null {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }

  function openCreateForm() {
    editingOrganization = null;
    form = emptyOrganizationForm();
    formOpen = true;
  }

  function openEditForm(organization: Organization) {
    editingOrganization = organization;
    form = {
      name: organization.name,
      email: organization.email ?? '',
      phone: organization.phone ?? '',
      website: organization.website ?? '',
      addressLine1: organization.addressLine1 ?? '',
      addressLine2: organization.addressLine2 ?? '',
      city: organization.city ?? '',
      region: organization.region ?? '',
      country: organization.country ?? '',
      postalCode: organization.postalCode ?? '',
      description: organization.description ?? '',
    };
    formOpen = true;
  }

  function closeForm() {
    formOpen = false;
    editingOrganization = null;
    form = emptyOrganizationForm();
  }

  async function submitOrganization() {
    if (!form.name.trim()) {
      uiStore.toastError(t('common.fieldRequired', { field: t('organizations.name') }));
      return;
    }

    const payload = {
      name: form.name.trim(),
      email: blankToNull(form.email),
      phone: blankToNull(form.phone),
      website: blankToNull(form.website),
      addressLine1: blankToNull(form.addressLine1),
      addressLine2: blankToNull(form.addressLine2),
      city: blankToNull(form.city),
      region: blankToNull(form.region),
      country: blankToNull(form.country),
      postalCode: blankToNull(form.postalCode),
      description: blankToNull(form.description),
    };

    if (editingOrganization) {
      await organizationStore.updateOrganization(editingOrganization.id, payload);
    } else {
      await organizationStore.createOrganization(payload);
    }

    closeForm();
  }

  async function deleteOrganization(organization: Organization) {
    const confirmed = window.confirm(t('organizations.confirmDelete', { name: organization.name }));
    if (!confirmed) return;

    await organizationStore.deleteOrganization(organization.id);
  }

  function openNotesTags(organization: Organization) {
    notesTagsOrganization = organization;
    notesTagsOpen = true;
  }

  function closeNotesTags() {
    notesTagsOpen = false;
    notesTagsOrganization = null;
  }

  function openLinkContact(organization: Organization) {
    linkOrganization = organization;
    linkContactId = '';
    linkOpen = true;
  }

  function closeLinkContact() {
    linkOpen = false;
    linkOrganization = null;
    linkContactId = '';
  }

  async function submitLinkContact(organizationId: string | null) {
    const contactId = linkContactId.trim();
    if (!contactId) {
      uiStore.toastError(t('organizations.linkContactRequired'));
      return;
    }

    await organizationStore.linkContactToOrganization(contactId, organizationId);
    closeLinkContact();
  }
</script>

<div class="page-content organizations-page">
  <div class="page-header">
    <h1 class="page-title">{t('organizations.title')}</h1>
    <button class="btn btn-primary btn-sm" type="button" onclick={openCreateForm}>
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M6 1v10M1 6h10"/>
      </svg>
      {t('organizations.addOrganization')}
    </button>
  </div>

  <div class="organizations-filters">
    <div class="search-wrap">
      <svg class="filter-search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        class="input filter-search selectable"
        type="search"
        bind:value={searchQuery}
        placeholder={t('organizations.search')}
        aria-label={t('organizations.search')}
      />
    </div>
  </div>

  <div class="organizations-table card">
    <div class="table-scroll">
      <table class="organization-list" aria-label={t('organizations.title')}>
        <thead>
          <tr>
            <th>{t('organizations.name')}</th>
            <th>{t('organizations.email')}</th>
            <th>{t('organizations.phone')}</th>
            <th>{t('organizations.website')}</th>
            <th>{t('organizations.location')}</th>
            <th>{t('organizations.created')}</th>
            <th>{t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#if organizationStore.isLoading}
            {#each Array(6) as _, index (index)}
              <tr class="skeleton-row">
                <td colspan="7"><div class="skeleton table-skeleton-cell"></div></td>
              </tr>
            {/each}
          {:else if filteredOrganizations.length === 0}
            <tr>
              <td colspan="7" class="empty-cell">
                <div class="empty-state">
                  <h2>{t('organizations.noOrganizations')}</h2>
                  <p>{t('organizations.noOrganizationsDesc')}</p>
                  <button class="btn btn-primary btn-sm" type="button" onclick={openCreateForm}>
                    {t('organizations.addOrganization')}
                  </button>
                </div>
              </td>
            </tr>
          {:else}
            {#each filteredOrganizations as organization (organization.id)}
              <tr>
                <td>
                  <div class="organization-name-cell">
                    <span class="organization-name">{organization.name}</span>
                    {#if organization.description}
                      <span class="organization-description">{organization.description}</span>
                    {/if}
                  </div>
                </td>
                <td>{organization.email ?? '—'}</td>
                <td>{organization.phone ?? '—'}</td>
                <td>
                  {#if organization.website}
                    <a href={organization.website} target="_blank" rel="noreferrer">{organization.website}</a>
                  {:else}
                    —
                  {/if}
                </td>
                <td>{organizationLocation(organization)}</td>
                <td>{formatDate(organization.createdAt, settingsStore.dateFormat as 'MMM D, YYYY')}</td>
                <td>
                  <div class="row-actions">
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openEditForm(organization)}>
                      {t('common.edit')}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openNotesTags(organization)}>
                      {t('organizations.manageNotesTags')}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openLinkContact(organization)}>
                      {t('organizations.linkContact')}
                    </button>
                    <button class="btn btn-ghost btn-sm danger-action" type="button" onclick={() => deleteOrganization(organization)}>
                      {t('common.delete')}
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<Modal
  bind:open={formOpen}
  title={editingOrganization ? t('organizations.editOrganization') : t('organizations.addOrganization')}
  size="lg"
  onclose={closeForm}
>
  <form class="organization-form" onsubmit={(event) => { event.preventDefault(); void submitOrganization(); }}>
    <div class="form-grid">
      <label class="form-group">
        <span class="form-label">{t('organizations.name')}</span>
        <input class="input selectable" bind:value={form.name} required />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.email')}</span>
        <input class="input selectable" type="email" bind:value={form.email} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.phone')}</span>
        <input class="input selectable" bind:value={form.phone} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.website')}</span>
        <input class="input selectable" type="url" bind:value={form.website} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.addressLine1')}</span>
        <input class="input selectable" bind:value={form.addressLine1} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.addressLine2')}</span>
        <input class="input selectable" bind:value={form.addressLine2} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.city')}</span>
        <input class="input selectable" bind:value={form.city} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.region')}</span>
        <input class="input selectable" bind:value={form.region} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.country')}</span>
        <input class="input selectable" bind:value={form.country} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.postalCode')}</span>
        <input class="input selectable" bind:value={form.postalCode} />
      </label>
      <label class="form-group form-group-full">
        <span class="form-label">{t('organizations.description')}</span>
        <textarea class="textarea selectable" rows="3" bind:value={form.description}></textarea>
      </label>
    </div>

    <div class="modal-form-actions">
      <button class="btn btn-ghost" type="button" onclick={closeForm}>{t('common.cancel')}</button>
      <button class="btn btn-primary" type="submit" disabled={organizationStore.isSaving}>
        {editingOrganization ? t('organizations.updateOrganization') : t('organizations.createOrganization')}
      </button>
    </div>
  </form>
</Modal>

<Modal
  bind:open={notesTagsOpen}
  title={notesTagsOrganization ? t('organizations.notesTagsTitle', { name: notesTagsOrganization.name }) : t('organizations.manageNotesTags')}
  size="lg"
  onclose={closeNotesTags}
>
  {#if notesTagsOrganization}
    <div class="organization-entity-panels">
      <section class="organization-entity-section" aria-labelledby="organization-tags-heading">
        <h2 class="modal-section-title" id="organization-tags-heading">{t('entityTags.title')}</h2>
        <EntityTagsPanel entityType="organization" entityId={notesTagsOrganization.id} />
      </section>

      <section class="organization-entity-section" aria-labelledby="organization-notes-heading">
        <h2 class="modal-section-title" id="organization-notes-heading">{t('entityNotes.title')}</h2>
        <EntityNotesPanel entityType="organization" entityId={notesTagsOrganization.id} />
      </section>
    </div>
  {/if}
</Modal>

<Modal
  bind:open={linkOpen}
  title={t('organizations.contactLink')}
  size="sm"
  onclose={closeLinkContact}
>
  <form class="link-form" onsubmit={(event) => { event.preventDefault(); void submitLinkContact(linkOrganization?.id ?? null); }}>
    <p class="link-description">{t('organizations.linkContactDesc')}</p>
    <label class="form-group">
      <span class="form-label">{t('organizations.contactId')}</span>
      <input class="input selectable" bind:value={linkContactId} required />
    </label>
    <div class="modal-form-actions link-actions">
      <button class="btn btn-ghost" type="button" onclick={closeLinkContact}>{t('common.cancel')}</button>
      <button
        class="btn btn-secondary"
        type="button"
        onclick={() => submitLinkContact(null)}
        disabled={organizationStore.isLinkingContact}
      >
        {t('organizations.clearContactLink')}
      </button>
      <button class="btn btn-primary" type="submit" disabled={organizationStore.isLinkingContact || !linkOrganization}>
        {t('organizations.linkContact')}
      </button>
    </div>
  </form>
</Modal>

<style>
  .organizations-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .organizations-filters {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    flex-wrap: wrap;
  }

  .search-wrap {
    position: relative;
    flex: 1;
    min-width: 220px;
    max-width: 380px;
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
    height: 34px;
    padding-inline-start: var(--space-10);
  }

  .organizations-table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .table-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .organization-list {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .organization-list th,
  .organization-list td {
    padding: var(--space-3) var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
  }

  .organization-list th {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    background-color: var(--surface-subtle);
  }

  .organization-name-cell {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 180px;
  }

  .organization-name {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .organization-description {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    max-width: 280px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .danger-action {
    color: var(--color-danger-600);
  }

  .empty-cell {
    padding: var(--space-10) var(--space-4);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    min-height: 260px;
    color: var(--text-secondary);
    text-align: center;
  }

  .empty-state h2 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  .empty-state p {
    margin: 0;
  }

  .organization-form,
  .link-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
  }

  .form-group-full {
    grid-column: 1 / -1;
  }

  .modal-form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .organization-entity-panels {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .organization-entity-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-block-end: var(--space-8);
    border-block-end: var(--border-width) solid var(--border-subtle);
  }

  .organization-entity-section:last-child {
    padding-block-end: 0;
    border-block-end: none;
  }

  .modal-section-title {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
  }

  .link-description {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .link-actions {
    justify-content: space-between;
  }

  @media (max-width: 720px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
