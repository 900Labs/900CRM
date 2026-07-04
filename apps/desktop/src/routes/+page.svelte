<script lang="ts">
  /**
   * +page.svelte — Hash route renderer for Tauri shell mode.
   */

  import { browser } from '$app/environment';
  import Dashboard from './Dashboard.svelte';
  import Contacts from './Contacts.svelte';
  import Organizations from './Organizations.svelte';
  import Pipeline from './Pipeline.svelte';
  import Activities from './Activities.svelte';
  import Settings from './Settings.svelte';
  import ContactDetail from './ContactDetail.svelte';
  import AuditLog from './AuditLog.svelte';
  import PendingActions from './PendingActions.svelte';
  import { currentHashPath, installHashRouteSync } from '$lib/utils/hashRouter';

  interface ParsedRoute {
    route: string;
    contactId: string | null;
  }

  function parseRoutePath(path: string): ParsedRoute {
    const clean = path.startsWith('/') ? path : `/${path}`;
    const normalized = clean.replace(/\/+$/, '') || '/';

    if (normalized === '/contacts') {
      return { route: '/contacts', contactId: null };
    }

    if (normalized.startsWith('/contacts/')) {
      const id = normalized.split('/')[2] ?? '';
      return { route: '/contacts/:id', contactId: id || null };
    }

    if (normalized === '/organizations') {
      return { route: '/organizations', contactId: null };
    }

    if (normalized === '/pipeline') {
      return { route: '/pipeline', contactId: null };
    }

    if (normalized === '/activities') {
      return { route: '/activities', contactId: null };
    }

    if (normalized === '/settings') {
      return { route: '/settings', contactId: null };
    }

    if (normalized === '/audit-log') {
      return { route: '/audit-log', contactId: null };
    }

    if (normalized === '/pending-actions') {
      return { route: '/pending-actions', contactId: null };
    }

    return { route: '/', contactId: null };
  }

  function readHashRoute(): ParsedRoute {
    if (!browser) {
      return { route: '/', contactId: null };
    }

    return parseRoutePath(currentHashPath());
  }

  const initialRoute = readHashRoute();

  let route = $state(initialRoute.route);
  let contactId = $state<string | null>(initialRoute.contactId);
  let routeSyncInitialized = false;

  function parseRoute(path: string) {
    const parsed = parseRoutePath(path);
    route = parsed.route;
    contactId = parsed.contactId;
  }

  $effect(() => {
    if (!browser || routeSyncInitialized) {
      return;
    }

    routeSyncInitialized = true;

    const syncFromHash = () => {
      parseRoute(currentHashPath());
    };
    const removeRouteSync = installHashRouteSync(syncFromHash);

    syncFromHash();
    window.addEventListener('hashchange', syncFromHash);

    return () => {
      removeRouteSync();
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
{:else if route === '/audit-log'}
  <AuditLog />
{:else if route === '/pending-actions'}
  <PendingActions />
{:else}
  <Dashboard />
{/if}
