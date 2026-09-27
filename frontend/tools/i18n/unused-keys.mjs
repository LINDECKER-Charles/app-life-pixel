// Keys no source file names literally. Error codes and emails are keyed at run time or by the
// server, so their prefixes are left out. A report, not a failure: a key stays while a supported
// app still uses it (docs/i18n.md).
const RUNTIME_PREFIXES = ['errors.', 'email.'];

/** Returns the keys of the catalogue that appear in none of the source texts. */
export function findUnusedKeys(keys, sourceTexts) {
  return keys
    .filter((key) => !RUNTIME_PREFIXES.some((prefix) => key.startsWith(prefix)))
    .filter((key) => !sourceTexts.some((text) => text.includes(key)));
}
