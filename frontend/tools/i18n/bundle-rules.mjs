// Catalogues are served, never bundled (docs/i18n.md): a bundled catalogue shows as its keys used
// as object properties with a text value — `"common.close":"Close"` —, quoted or JSON-escaped.

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function entryPattern(key) {
  const quote = String.raw`\\?["'\x60]`;
  // The value must be a JSON-like string literal (double or single quoted, optionally
  // JSON-escaped): a backtick there is a template literal, not a catalogue value — as in a
  // minified ternary between two key literals, e.g. `` `a.b`:`c.d` ``.
  const valueQuote = String.raw`\\?["']`;
  return new RegExp(`${quote}${escapeRegExp(key)}${quote}\\s*:\\s*${valueQuote}`);
}

/** Returns, for each script, the catalogue keys it carries as entries of a catalogue. */
export function findBundledEntries(scripts, keys) {
  const patterns = keys.map((key) => ({ key, pattern: entryPattern(key) }));
  return scripts.flatMap(({ name, text }) =>
    patterns.filter(({ pattern }) => pattern.test(text)).map(({ key }) => `${name}: ${key}`),
  );
}
