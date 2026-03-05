<script lang="ts">
  /**
   * Modal.svelte — Reusable modal dialog component for 900CRM.
   *
   * Features:
   *   - Overlay backdrop with click-to-close
   *   - Escape key handler
   *   - Focus trap (Tab/Shift+Tab cycle within modal)
   *   - Slots: header, body (default), footer
   *   - Configurable width
   *   - RTL support via CSS logical properties
   *
   * @example
   * <Modal open={isOpen} title="Add Contact" on:close={() => isOpen = false}>
   *   <div slot="body">…content…</div>
   *   <div slot="footer">
   *     <button class="btn btn-secondary">Cancel</button>
   *     <button class="btn btn-primary">Save</button>
   *   </div>
   * </Modal>
   */

  import { t } from '$lib/i18n';
  import { onMount, onDestroy } from 'svelte';

  // ── Props ──────────────────────────────────────────────────────────────────

  /** Whether the modal is visible. */
  let {
    open = $bindable(false),
    title = '',
    width = '560px',
    closeOnBackdrop = true,
    closeOnEscape = true,
    onclose,
  }: {
    open?: boolean;
    title?: string;
    width?: string;
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    onclose?: () => void;
  } = $props();

  // ── Refs ───────────────────────────────────────────────────────────────────

  let modalEl: HTMLDivElement | undefined;
  let previousFocus: Element | null = null;

  // ── Helpers ────────────────────────────────────────────────────────────────

  function close() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      close();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && closeOnEscape) {
      close();
      return;
    }

    // Focus trap
    if (e.key === 'Tab' && modalEl) {
      const focusable = modalEl.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );
      const first = focusable[0];
      const last  = focusable[focusable.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    }
  }

  // ── Lifecycle ──────────────────────────────────────────────────────────────

  $effect(() => {
    if (open) {
      previousFocus = document.activeElement;
      // Focus the first focusable element inside the modal
      requestAnimationFrame(() => {
        const first = modalEl?.querySelector<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), [tabindex="0"]'
        );
        first?.focus();
      });
    } else if (previousFocus instanceof HTMLElement) {
      previousFocus.focus();
      previousFocus = null;
    }
  });

  onMount(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if open}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    onclick={handleBackdropClick}
  >
    <div
      class="modal"
      style="width: {width}; max-width: calc(100vw - 2rem);"
      bind:this={modalEl}
    >
      <!-- Header -->
      <div class="modal-header">
        {#if title}
          <span class="modal-title">{title}</span>
        {/if}
        {@render header?.()}
        <button
          class="icon-btn modal-close"
          onclick={close}
          aria-label={t('common.close')}
          type="button"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M12 4L4 12M4 4l8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        {@render children?.()}
      </div>

      <!-- Footer -->
      {#if footer}
        <div class="modal-footer">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}

{#snippet header()}<!-- optional header slot override -->{/snippet}
{#snippet footer()}<!-- optional footer slot -->{/snippet}

<style>
  .modal-close {
    margin-inline-start: auto;
    flex-shrink: 0;
  }
</style>
