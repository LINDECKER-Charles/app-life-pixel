import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

/** Where the captures and the measurements go: ignored by Git (`.gitignore` beside this file). */
export const OUTPUT_DIR = resolve(__dirname, 'output');

export type Language = 'en' | 'fr';
export type Theme = 'light' | 'dark';

/** The widths every screen is captured at, each with a height typical of such a screen. */
export const VIEWPORTS = [
  { width: 1440, height: 900 },
  { width: 1024, height: 768 },
  { width: 768, height: 1024 },
  { width: 390, height: 844 },
] as const;

/**
 * The languages, chosen as a visitor's browser chooses them: the web keeps no preference (D37),
 * so the app picks the catalogue from `navigator.languages`, which Playwright's `locale` sets.
 */
export const LANGUAGES = [
  { code: 'en', locale: 'en-US' },
  { code: 'fr', locale: 'fr-FR' },
] as const satisfies readonly { code: Language; locale: string }[];

/**
 * The themes, through the default "system" preference: the tokens follow `prefers-color-scheme`,
 * which Playwright's `colorScheme` emulates.
 */
export const THEMES: readonly Theme[] = ['light', 'dark'];

/** Writes `value` as formatted JSON under the output directory, creating its folders. */
export function writeJson(relativePath: string, value: unknown): string {
  const path = resolve(OUTPUT_DIR, relativePath);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
  return path;
}
