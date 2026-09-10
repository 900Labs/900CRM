<script lang="ts">
  /**
   * BrandMark.svelte — 900 Labs mark used in the sidebar and any in-app chrome.
   * Matches apps/desktop/src-tauri/icons/app-icon.svg.
   */
  let { size = 28 }: { size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 28 28"
  fill="none"
  aria-hidden="true"
>
  <rect width="28" height="28" rx="6" fill="var(--color-primary)" />
  <path
    fill="#ffffff"
    fill-rule="evenodd"
    d="M14 4.6a6.2 6.2 0 1 1 0 12.4 6.2 6.2 0 0 1 0-12.4zm0 3.1a3.1 3.1 0 1 0 0 6.2 3.1 3.1 0 0 0 0-6.2z"
  />
  <rect x="17.1" y="10.8" width="2.7" height="11" rx="1.35" fill="#ffffff" />
  <circle cx="14" cy="10.8" r="1.55" fill="#A84B2F" />
</svg>
