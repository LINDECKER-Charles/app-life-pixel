// Catalogues are served, never bundled (docs/i18n.md): a bundled catalogue shows as its keys used
// as object properties with a text value — `"common.close":"Close"` —, quoted or JSON-escaped.

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function entryPattern(key) {
  const quote = String.raw`\\?["'\x60]`;
  return new RegExp(`${quote}${escapeRegExp(key)}${quote}\\s*:\\s*${quote}`);
}

/** Returns, for each script, the catalogue keys it carries as entries of a catalogue. */
export function findBundledEntries(scripts, keys) {
  const patterns = keys.map((key) => ({ key, pattern: entryPattern(key) }));
  return scripts.flatMap(({ name, text }) =>
    patterns.filter(({ pattern }) => pattern.test(text)).map(({ key }) => `${name}: ${key}`),
  );
}
