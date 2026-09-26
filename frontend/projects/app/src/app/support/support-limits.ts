import type { SupportCategory, SupportRequestStatus } from 'shared';

/**
 * `core::limits`, transcribed for the support form (support-admin.md, H9): like
 * `ACCOUNT_LIMITS`, it spares the support pages the editor's wasm engine.
 */
export const SUPPORT_LIMITS = {
  messageMaxChars: 5_000,
  screenshotMaxBytes: 5_242_880,
} as const;

/** The screenshot's limit as the form states it: 5 MB. */
export const SCREENSHOT_MAX_MEGABYTES = SUPPORT_LIMITS.screenshotMaxBytes / (1024 * 1024);

/** The media types the server decodes; the file picker offers only these. */
export const SCREENSHOT_TYPES: readonly string[] = ['image/png', 'image/jpeg'];

// Literal keys, so that the i18n check sees each one used.
/** The label of each category, in the order the form lists them. */
export const CATEGORY_LABELS: Readonly<Record<SupportCategory, string>> = {
  bug: 'support.category.bug',
  account: 'support.category.account',
  billing: 'support.category.billing',
  data_protection: 'support.category.data_protection',
  abuse: 'support.category.abuse',
  other: 'support.category.other',
};

/** The label of each status. */
export const STATUS_LABELS: Readonly<Record<SupportRequestStatus, string>> = {
  new: 'support.status.new',
  in_progress: 'support.status.in_progress',
  waiting_for_user: 'support.status.waiting_for_user',
  resolved: 'support.status.resolved',
  closed: 'support.status.closed',
};

/** The characters of `text` once trimmed, as the server counts them: code points. */
export function messageLength(text: string): number {
  return [...text.trim()].length;
}

/** Why `file` cannot be sent as a screenshot, as an i18n key; `undefined` when it can. */
export function screenshotRefusal(file: File): string | undefined {
  if (!SCREENSHOT_TYPES.includes(file.type)) {
    return 'support.new.screenshot_type';
  }
  if (file.size > SUPPORT_LIMITS.screenshotMaxBytes) {
    return 'support.new.screenshot_size';
  }
  return undefined;
}

/** `iso` as a date and time of `language`. */
export function formatDate(iso: string, language: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium', timeStyle: 'short' }).format(
    new Date(iso),
  );
}
