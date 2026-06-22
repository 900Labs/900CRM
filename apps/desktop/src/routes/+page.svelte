<script lang="ts">
  /**
   * +page.svelte — Hash route renderer for Tauri shell mode.
   */

  import { onMount } from 'svelte';
  import Dashboard from './Dashboard.svelte';
  import Contacts from './Contacts.svelte';
  import Organizations from './Organizations.svelte';
  import Pipeline from './Pipeline.svelte';
  import Activities from './Activities.svelte';
  import Settings from './Settings.svelte';
  import ContactDetail from './ContactDetail.svelte';

  let route = $state('/');
  let contactId = $state<string | null>(null);

  function parseRoute(path: string) {
    const clean = path.startsWith('/') ? path : `/${path}`;
    const normalized = clean.replace(/\/+$/, '') || '/';

    if (normalized === '/contacts') {
      route = '/contacts';
      contactId = null;
      return;
    }

    if (normalized.startsWith('/contacts/')) {
      const id = normalized.split('/')[2] ?? '';
      route = '/contacts/:id';
      contactId = id || null;
      return;
    }

    if (normalized === '/organizations') {
      route = '/organizations';
      contactId = null;
      return;
    }

    if (normalized === '/pipeline') {
      route = '/pipeline';
      contactId = null;
      return;
    }

    if (normalized === '/activities') {
      route = '/activities';
      contactId = null;
      return;
    }

    if (normalized === '/settings') {
      route = '/settings';
      contactId = null;
      return;
    }

    route = '/';
    contactId = null;
  }

  onMount(() => {
    const syncFromHash = () => {
      const hashPath = window.location.hash.replace(/^#/, '') || '/';
      parseRoute(hashPath);
    };

    syncFromHash();
    window.addEventListener('hashchange', syncFromHash);

    return () => {
      window.removeEventListener('hashchange', syncFromHash);
    };
  });
</script>

{#if route === '/contacts/:id' && contactId}
  <ContactDetail {contactId} />
{:else if route === '/contacts'}
  <Contacts />
{:else if route === '/organizations'}
  <Organizations />
{:else if route === '/pipeline'}
  <Pipeline />
{:else if route === '/activities'}
  <Activities />
{:else if route === '/settings'}
  <Settings />
{:else}
  <Dashboard />
{/if}
