/**
 * src/lib/utils/formatters.ts — Formatting helpers for 900CRM.
 *
 * All functions are pure and locale-aware. No side effects.
 *
 * @module utils/formatters
 */

// ─────────────────────────────────────────────────────────────────────────────
// Currency
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Format a numeric value as a currency string.
 *
 * @param value    The numeric amount
 * @param currency ISO 4217 currency code (e.g. 'USD', 'EUR', 'KES')
 * @param locale   BCP 47 locale string (e.g. 'en-US', 'ar-SA')
 * @returns        Formatted currency string (e.g. '$1,234.56')
 */
export function formatCurrency(
  value: number,
  currency = 'USD',
  locale = 'en-US'
): string {
  try {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency,
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    }).format(value);
  } catch {
    return `${currency} ${value.toFixed(2)}`;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dates
// ─────────────────────────────────────────────────────────────────────────────

/** Supported date format tokens. */
export type DateFormat = 'YYYY-MM-DD' | 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'MMM D, YYYY';

/**
 * Format an ISO date string according to the specified format.
 *
 * @param iso     ISO 8601 date string or timestamp
 * @param format  Target format string
 * @param locale  BCP 47 locale for month names etc.
 * @returns       Formatted date string, or empty string on invalid input
 */
export function formatDate(
  iso: string | null | undefined,
  format: DateFormat = 'YYYY-MM-DD',
  locale = 'en-US'
): string {
  if (!iso) return '';

  const date = new Date(iso);
  if (isNaN(date.getTime())) return '';

  const year  = date.getFullYear();
  const month = date.getMonth() + 1;
  const day   = date.getDate();
  const pad   = (n: number) => String(n).padStart(2, '0');

  switch (format) {
    case 'YYYY-MM-DD':
      return `${year}-${pad(month)}-${pad(day)}`;
    case 'DD/MM/YYYY':
      return `${pad(day)}/${pad(month)}/${year}`;
    case 'MM/DD/YYYY':
      return `${pad(month)}/${pad(day)}/${year}`;
    case 'MMM D, YYYY': {
      try {
        return date.toLocaleDateString(locale, {
          month: 'short',
          day: 'numeric',
          year: 'numeric',
        });
      } catch {
        return `${pad(month)}/${pad(day)}/${year}`;
      }
    }
    default:
      return `${year}-${pad(month)}-${pad(day)}`;
  }
}

/**
 * Format an ISO timestamp as a relative time string (e.g. "3 hours ago").
 *
 * Falls back to absolute date if the timestamp is older than 7 days.
 *
 * @param iso  ISO 8601 string or timestamp
 * @returns    Human-readable relative time string
 */
export function formatRelativeTime(iso: string | null | undefined): string {
  if (!iso) return '';

  const date = new Date(iso);
  if (isNaN(date.getTime())) return '';

  const now   = Date.now();
  const diff  = now - date.getTime(); // ms
  const secs  = Math.floor(diff / 1000);
  const mins  = Math.floor(secs / 60);
  const hours = Math.floor(mins / 60);
  const days  = Math.floor(hours / 24);

  if (secs < 60)        return 'just now';
  if (mins < 60)        return `${mins}m ago`;
  if (hours < 24)       return `${hours}h ago`;
  if (days === 1)       return 'yesterday';
  if (days < 7)         return `${days}d ago`;
  if (days < 30)        return `${Math.floor(days / 7)}w ago`;
  if (days < 365)       return `${Math.floor(days / 30)}mo ago`;
  return `${Math.floor(days / 365)}y ago`;
}

// ─────────────────────────────────────────────────────────────────────────────
// Phone
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Format a phone number string.
 * Currently does basic formatting; extend with libphonenumber for full support.
 *
 * @param phone    Raw phone string
 * @param country  ISO 3166-1 alpha-2 country code (e.g. 'US', 'KE')
 * @returns        Formatted phone string, or original if unrecognized
 */
export function formatPhone(phone: string | null | undefined, _country = 'US'): string {
  if (!phone) return '';

  // Strip all non-digits
  const digits = phone.replace(/\D/g, '');

  // US: (XXX) XXX-XXXX
  if (digits.length === 10) {
    return `(${digits.slice(0, 3)}) ${digits.slice(3, 6)}-${digits.slice(6)}`;
  }

  // US with country code: +1 (XXX) XXX-XXXX
  if (digits.length === 11 && digits[0] === '1') {
    return `+1 (${digits.slice(1, 4)}) ${digits.slice(4, 7)}-${digits.slice(7)}`;
  }

  // International: return with + prefix
  if (digits.length > 10) {
    return `+${digits}`;
  }

  return phone;
}

// ─────────────────────────────────────────────────────────────────────────────
// Names / Initials
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Extract up to 2-character initials from a contact's name.
 *
 * @param firstName  First name (or full name if lastName is omitted)
 * @param lastName   Last name (optional)
 * @returns          1-2 uppercase initials (e.g. 'JD', 'A')
 */
export function formatInitials(firstName = '', lastName = ''): string {
  const first = firstName.trim();
  const last  = lastName.trim();

  if (first && last) {
    return (first[0] + last[0]).toUpperCase();
  }

  if (first) {
    const parts = first.split(/\s+/);
    if (parts.length >= 2) {
      return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }
    return first[0].toUpperCase();
  }

  return '?';
}

/**
 * Format a full name from first and last.
 */
export function formatFullName(firstName = '', lastName = ''): string {
  return [firstName, lastName].filter(Boolean).join(' ').trim() || 'Unknown';
}

// ─────────────────────────────────────────────────────────────────────────────
// Numbers
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Format a large number with compact suffix (K, M, B).
 * e.g. 1500000 → '1.5M'
 */
export function formatCompactNumber(value: number): string {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000)     return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000)         return `${(value / 1_000).toFixed(1)}K`;
  return String(value);
}

/**
 * Format a percentage (0–100).
 * e.g. 75 → '75%'
 */
export function formatPercent(value: number, decimals = 0): string {
  return `${value.toFixed(decimals)}%`;
}
