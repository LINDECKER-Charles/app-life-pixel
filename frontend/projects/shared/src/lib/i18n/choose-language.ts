/** The source language of the catalogues, and the fallback of every choice. */
export const SOURCE_LANGUAGE = 'en';

/**
 * Picks the first candidate that has a catalogue — exactly, else by its primary subtag, so that
 * `fr-CA` picks `fr` — and falls back to the source language.
 */
export function chooseLanguage(
  candidates: readonly (string | undefined)[],
  availableCodes: readonly string[],
): string {
  for (const candidate of candidates) {
    const match = candidate === undefined ? undefined : matchAvailable(candidate, availableCodes);
    if (match !== undefined) {
      return match;
    }
  }
  return SOURCE_LANGUAGE;
}

function matchAvailable(candidate: string, availableCodes: readonly string[]): string | undefined {
  const wanted = candidate.toLowerCase();
  const primary = wanted.split('-')[0];
  return (
    availableCodes.find((code) => code.toLowerCase() === wanted) ??
    availableCodes.find((code) => code.toLowerCase() === primary)
  );
}
