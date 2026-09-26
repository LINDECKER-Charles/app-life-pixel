import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { Language } from './visual-matrix';

const I18N_DIR = resolve(__dirname, '../../../i18n');
const catalogues = new Map<Language, Record<string, string>>();

function catalogue(language: Language): Record<string, string> {
  let messages = catalogues.get(language);
  if (!messages) {
    messages = JSON.parse(readFileSync(resolve(I18N_DIR, `${language}.json`), 'utf8'));
    catalogues.set(language, messages ?? {});
  }
  return messages ?? {};
}

/**
 * The text a key shows in `language`, read from the catalogues of `i18n/` so that the same
 * journey runs in every language. Simple arguments — `{name}` or `{name, number}` — are replaced.
 */
export function label(
  language: Language,
  key: string,
  values: Readonly<Record<string, string | number>> = {},
): string {
  const message = catalogue(language)[key];
  if (message === undefined) throw new Error(`No message ${key} in ${language}.json`);
  return message.replace(/\{\s*(\w+)\s*(?:,\s*number\s*)?\}/g, (whole, name: string) =>
    name in values ? String(values[name]) : whole,
  );
}
