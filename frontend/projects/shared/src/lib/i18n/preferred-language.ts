import { InjectionToken } from '@angular/core';

/** Returns a language code a person chose earlier, or `undefined` when there is none. */
export type PreferredLanguageSource = () => Promise<string | undefined> | string | undefined;

/**
 * Where a person's language may already be known, asked in the order provided, before the
 * browser's languages: the account's (hosted) and the desktop settings'. Each adds its own
 * provider: `{ provide: PREFERRED_LANGUAGE, useValue: source, multi: true }`.
 */
export const PREFERRED_LANGUAGE = new InjectionToken<readonly PreferredLanguageSource[]>(
  'PREFERRED_LANGUAGE',
);
