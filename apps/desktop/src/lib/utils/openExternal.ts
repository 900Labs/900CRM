import { open } from '@tauri-apps/plugin-shell';
import { validateUrl } from './validators';

const ALLOWED_PROTOCOLS = new Set(['http:', 'https:']);

/**
 * Open a user-supplied URL in the OS handler after an allow-list check.
 * mailto: stays on the email helper. javascript: and file: are rejected.
 */
export async function openExternalUrl(raw: string): Promise<void> {
  const value = raw.trim();
  const result = validateUrl(value);
  if (!result.valid) {
    throw new Error(result.error ?? 'Invalid URL');
  }

  const parsed = new URL(value);
  if (!ALLOWED_PROTOCOLS.has(parsed.protocol)) {
    throw new Error('URL must use http or https');
  }

  await open(parsed.toString());
}
