<script lang="ts">
  /**
   * SearchBar.svelte — Global search with debounce and results dropdown.
   *
   * Debounces input by 300ms, calls the unified search command, and shows
   * an inline dropdown with type badges per result.
   */

  import { t } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';
  import { searchContacts } from '$lib/api/contacts';
  import type { SearchResult } from '$lib/stores/ui';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    placeholder = '',
    onselectresult,
  }: {
    placeholder?: string;
    onselectresult?: (result: SearchResult) => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────

  let inputEl: HTMLInputElement | undefined;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  // ── Helpers ────────────────────────────────────────────────────────────────

  function handleInput(e: Event) {
    const query = (e.target as HTMLInputElement).value;
    uiStore.setSearchQuery(query);

    clearTimeout(debounceTimer);

    if (!query.trim()) {
      uiStore.clearSearch();
      return;
    }

    uiStore.isSearching = true;
    debounceTimer = setTimeout(() => performSearch(query), 300);
  }

  async function performSearch(query: string) {
    try {
      // Search contacts (expand to deals/activities as needed)
      const contacts = await searchContacts(query);
      const results: SearchResult[] = contacts.slice(0, 8).map((c) => ({
        id: c.id,
        type: 'contact' as const,
        title: [c.firstName, c.lastName].filter(Boolean).join(' ') || 'Unknown',
        subtitle: c.email || c.organization || '',
      }));
      uiStore.setSearchResults(results);
    } catch {
      uiStore.setSearchResults([]);
    }
  }

  function handleSelect(result: SearchResult) {
    onselectresult?.(result);
    uiStore.clearSearch();
    if (inputEl) inputEl.value = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      uiStore.closeSearch();
      inputEl?.blur();
    }
  }

  function handleBlur() {
    setTimeout(() => uiStore.closeSearch(), 200);
  }

  function typeLabel(type: string): string {
    const map: Record<string, string> = {
      contact:  t('contacts.title'),
      deal:     t('deals.title'),
      activity: t('activities.title'),
    };
    return map[type] ?? type;
  }

  function typeBadgeClass(type: string): string {
    const map: Record<string, string> = {
      contact:  'badge-primary',
      deal:     'badge-success',
      activity: 'badge-warning',
    };
    return map[type] ?? 'badge-neutral';
  }
</script>

<div class="search-bar">
  <div class="search-input-wrap">
    <!-- Search icon -->
    <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
      <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
    </svg>

    <input
      class="input search-input selectable"
      bind:this={inputEl}
      type="search"
      placeholder={placeholder || t('common.search')}
      aria-label={t('common.search')}
      oninput={handleInput}
      onkeydown={handleKeyDown}
      onblur={handleBlur}
      autocomplete="off"
      spellcheck={false}
    />

    {#if uiStore.isSearching}
      <svg class="search-spinner animate-spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
    {/if}
  </div>

  <!-- Results dropdown -->
  {#if uiStore.searchOpen && uiStore.searchResults.length > 0}
    <ul class="search-results" role="listbox" aria-label={t('common.search') + ' results'}>
      {#each uiStore.searchResults as result (result.id)}
        <li
          class="search-result-item"
          role="option"
          aria-selected="false"
          onmousedown|preventDefault={() => handleSelect(result)}
        >
          <div class="search-result-text">
            <span class="search-result-title">{result.title}</span>
            {#if result.subtitle}
              <span class="search-result-subtitle">{result.subtitle}</span>
            {/if}
          </div>
          <span class="badge {typeBadgeClass(result.type)}">{typeLabel(result.type)}</span>
        </li>
      {/each}
    </ul>
  {:else if uiStore.searchOpen && uiStore.searchQuery && !uiStore.isSearching}
    <div class="search-no-results">
      {t('common.noResults')}
    </div>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    width: 280px;
  }

  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    inset-inline-start: var(--space-4);
    color: var(--icon-muted);
    pointer-events: none;
    flex-shrink: 0;
  }

  .search-input {
    padding-inline-start: var(--space-10);
    padding-inline-end: var(--space-8);
    height: 32px;
    font-size: var(--text-sm);
    background-color: var(--surface-input);
  }

  /* Remove native search clear button */
  .search-input::-webkit-search-cancel-button {
    display: none;
  }

  .search-spinner {
    position: absolute;
    inset-inline-end: var(--space-4);
    color: var(--icon-muted);
    pointer-events: none;
  }

  .search-results {
    position: absolute;
    top: calc(100% + var(--space-2));
    inset-inline-start: 0;
    width: 100%;
    min-width: 320px;
    background-color: var(--surface-modal);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-xl);
    z-index: var(--z-tooltip);
    overflow: hidden;
    list-style: none;
    animation: scale-in var(--duration-fast) var(--ease-out) forwards;
    transform-origin: top center;
  }

  .search-result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .search-result-item:hover {
    background-color: var(--surface-hover);
  }

  .search-result-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .search-result-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-result-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-no-results {
    padding: var(--space-6) var(--space-5);
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    text-align: center;
  }
</style>
