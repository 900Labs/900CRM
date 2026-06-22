<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { organizationStore } from '$lib/stores/organizations';
  import { settingsStore } from '$lib/stores/settings';
  import type { Organization } from '$lib/api/organizations';
  import { formatDate } from '$lib/utils/formatters';

  let searchQuery = $state('');
  let editingId = $state<string | null>(null);
  let name = $state('');
  let email = $state('');
  let phone = $state('');
  let website = $state('');
  let city = $state('');
  let country = $state('');
  let description = $state('');
  let formError = $state<string | null>(null);

  const filteredOrganizations = $derived(
    organizationStore.organizations.filter((organization) => {
      const query = searchQuery.trim().toLowerCase();
      if (!query) return true;
      return [organization.name, organization.email, organization.phone, organization.city, organization.country]
        .filter((value): value is string => Boolean(value))
        .some((value) => value.toLowerCase().includes(query));
    })
  );

  onMount(async () => {
    await organizationStore.loadOrganizations();
  });

  function resetForm() {
    editingId = null;
    name = '';
    email = '';
    phone = '';
    website = '';
    city = '';
    country = '';
    description = '';
    formError = null;
  }

  function editOrganization(organization: Organization) {
    editingId = organization.id;
    name = organization.name;
    email = organization.email ?? '';
    phone = organization.phone ?? '';
    website = organization.website ?? '';
    city = organization.city ?? '';
    country = organization.country ?? '';
    description = organization.description ?? '';
    formError = null;
  }

  async function saveOrganization() {
    if (!name.trim()) {
      formError = t('organizations.nameRequired');
      return;
    }

    formError = null;
    const payload = {
      name: name.trim(),
      email: email.trim() || null,
      phone: phone.trim() || null,
      website: website.trim() || null,
      addressLine1: null,
      addressLine2: null,
      city: city.trim() || null,
      region: null,
      country: country.trim() || null,
      postalCode: null,
      description: description.trim() || null,
    };

    if (editingId) {
      await organizationStore.updateOrganization(editingId, payload);
    } else {
      await organizationStore.createOrganization(payload);
    }
    resetForm();
  }

  async function deleteOrganization(id: string) {
    await organizationStore.deleteOrganization(id);
    if (editingId === id) {
      resetForm();
    }
  }
</script>

<div class="page-content organizations-page">
  <div class="page-header">
    <h1 class="page-title">{t('organizations.title')}</h1>
  </div>

  <div class="organizations-layout">
    <section class="organization-form" aria-labelledby="organization-form-title">
      <h2 class="section-title" id="organization-form-title">
        {editingId ? t('organizations.editOrganization') : t('organizations.addOrganization')}
      </h2>

      <div class="form-grid">
        <div class="form-group form-group--full">
          <label class="form-label" for="organization-name">{t('organizations.name')}</label>
          <input id="organization-name" class="input" bind:value={name} />
        </div>
        <div class="form-group">
          <label class="form-label" for="organization-email">{t('organizations.email')}</label>
          <input id="organization-email" class="input" type="email" bind:value={email} />
        </div>
        <div class="form-group">
          <label class="form-label" for="organization-phone">{t('organizations.phone')}</label>
          <input id="organization-phone" class="input" bind:value={phone} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="organization-website">{t('organizations.website')}</label>
          <input id="organization-website" class="input" type="url" bind:value={website} />
        </div>
        <div class="form-group">
          <label class="form-label" for="organization-city">{t('organizations.city')}</label>
          <input id="organization-city" class="input" bind:value={city} />
        </div>
        <div class="form-group">
          <label class="form-label" for="organization-country">{t('organizations.country')}</label>
          <input id="organization-country" class="input" bind:value={country} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="organization-description">{t('organizations.description')}</label>
          <textarea
            id="organization-description"
            class="input textarea"
            rows="3"
            bind:value={description}
          ></textarea>
        </div>
      </div>

      {#if formError}
        <p class="field-error">{formError}</p>
      {/if}

      <div class="form-actions">
        {#if editingId}
          <button class="btn btn-secondary btn-sm" type="button" onclick={resetForm}>
            {t('common.cancel')}
          </button>
        {/if}
        <button
          class="btn btn-primary btn-sm"
          type="button"
          onclick={saveOrganization}
          disabled={organizationStore.isSaving}
        >
          {organizationStore.isSaving ? t('common.loading') : t('common.save')}
        </button>
      </div>
    </section>

    <section class="organizations-list" aria-labelledby="organizations-list-title">
      <div class="list-header">
        <h2 class="section-title" id="organizations-list-title">{t('organizations.list')}</h2>
        <input
          class="input search-input"
          type="search"
          bind:value={searchQuery}
          placeholder={t('organizations.search')}
          aria-label={t('organizations.search')}
        />
      </div>

      {#if organizationStore.isLoading}
        <div class="list-state">{t('common.loading')}</div>
      {:else if filteredOrganizations.length === 0}
        <div class="list-state">{t('organizations.noOrganizations')}</div>
      {:else}
        <div class="table-wrap">
          <table class="organizations-table">
            <thead>
              <tr>
                <th>{t('organizations.name')}</th>
                <th>{t('organizations.contact')}</th>
                <th>{t('organizations.location')}</th>
                <th>{t('common.updated')}</th>
                <th>{t('common.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredOrganizations as organization (organization.id)}
                <tr>
                  <td>
                    <button
                      class="link-button"
                      type="button"
                      onclick={() => editOrganization(organization)}
                    >
                      {organization.name}
                    </button>
                  </td>
                  <td>{organization.email ?? organization.phone ?? '—'}</td>
                  <td>{[organization.city, organization.country].filter(Boolean).join(', ') || '—'}</td>
                  <td>{formatDate(organization.updatedAt, settingsStore.dateFormat as 'MMM D, YYYY')}</td>
                  <td>
                    <div class="row-actions">
                      <button
                        class="btn btn-secondary btn-xs"
                        type="button"
                        onclick={() => editOrganization(organization)}
                      >
                        {t('common.edit')}
                      </button>
                      <button
                        class="btn btn-danger btn-xs"
                        type="button"
                        onclick={() => deleteOrganization(organization.id)}
                      >
                        {t('common.delete')}
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .organizations-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .organizations-layout {
    display: grid;
    grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
    gap: var(--space-6);
    align-items: start;
  }

  .organization-form,
  .organizations-list {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-card);
    padding: var(--space-5);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
    margin-top: var(--space-4);
  }

  .form-group--full {
    grid-column: 1 / -1;
  }

  .form-actions,
  .row-actions,
  .list-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .form-actions {
    justify-content: flex-end;
    margin-top: var(--space-4);
  }

  .list-header {
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .search-input {
    width: min(280px, 100%);
  }

  .table-wrap {
    overflow: auto;
  }

  .organizations-table {
    width: 100%;
    border-collapse: collapse;
  }

  .organizations-table th,
  .organizations-table td {
    padding: var(--space-3);
    border-bottom: var(--border-width) solid var(--border-subtle);
    text-align: left;
    vertical-align: middle;
    white-space: nowrap;
  }

  .organizations-table th {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .link-button {
    border: 0;
    background: transparent;
    color: var(--text-accent);
    font: inherit;
    font-weight: var(--weight-medium);
    padding: 0;
    cursor: pointer;
  }

  .list-state {
    min-height: 160px;
    display: grid;
    place-items: center;
    color: var(--text-secondary);
  }

  .field-error {
    color: var(--color-danger);
    font-size: var(--text-sm);
    margin-top: var(--space-3);
  }

  @media (max-width: 900px) {
    .organizations-layout {
      grid-template-columns: 1fr;
    }

    .list-header {
      align-items: stretch;
      flex-direction: column;
    }
  }
</style>
