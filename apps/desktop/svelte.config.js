import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    // Static adapter for Tauri — builds to 'build/' directory.
    adapter: adapter({
      fallback: 'index.html',
      strict: false,
    }),
  },
};

export default config;
