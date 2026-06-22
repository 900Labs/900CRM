/**
 * src/lib/i18n/bootstrap.ts — Runtime locale bootstrap and persistence hooks.
 *
 * Keeps environment-bound startup concerns separate from translation core logic.
 */

import { availableLocales, primeFallbackLocale, setLocale, type LocaleCode } from './index';

let initPromise: Promise<void> | null = null;

function isLocaleCode(code: string): code is LocaleCode {
  return availableLocales.some((locale) => locale.code === code);
}

/**
 * Detect the preferred locale:
 *   1. localStorage persisted preference
 *   2. navigator language preference
 *   3. English fallback
 */
function detectInitialLocale(): LocaleCode {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem('900crm-locale');
    if (saved && isLocaleCode(saved)) {
      return saved;
    }
  }

  if (typeof navigator !== 'undefined') {
    for (const lang of navigator.languages ?? [navigator.language]) {
      const code = lang.split('-')[0].toLowerCase();
      if (isLocaleCode(code)) {
        return code;
      }
    }
  }

  return 'en';
}

/**
 * Explicit i18n startup hook.
 * Call once from app bootstrap before rendering route content.
 */
export function initI18n(): Promise<void> {
  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    await primeFallbackLocale();
    await setLocale(detectInitialLocale());
  })();

  return initPromise;
}

/**
 * Persist the user locale choice and apply it.
 */
export async function chooseLocale(code: LocaleCode | string): Promise<void> {
  if (!isLocaleCode(code)) {
    throw new Error(`Unsupported locale: ${code}`);
  }

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('900crm-locale', code);
  }
  await setLocale(code);
}
