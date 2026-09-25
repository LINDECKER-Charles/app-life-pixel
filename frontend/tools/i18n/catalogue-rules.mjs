// The rules every catalogue of i18n/ follows (docs/i18n.md). Each check returns the list of
// problems it finds, as sentences; an empty list means the rule holds.
import { parse } from '@messageformat/parser';

const KEY_PATTERN = /^[a-z0-9_]+(\.[a-z0-9_]+)+$/;
const EMAIL_PREFIX = 'email.';
const SIMPLE_TOKEN_TYPES = new Set(['content', 'argument']);

/** The canonical text of a catalogue: sorted flat keys, one per line, two-space indentation. */
export function formatCatalogue(catalogue) {
  const sorted = Object.fromEntries(Object.entries(catalogue).sort(([a], [b]) => compare(a, b)));
  return `${JSON.stringify(sorted, null, 2)}\n`;
}

/** Checks that a catalogue's text is a flat object of messages with valid, sorted keys. */
export function checkCatalogueText(fileName, text) {
  let catalogue;
  try {
    catalogue = JSON.parse(text);
  } catch (error) {
    return [`${fileName}: not valid JSON (${error.message})`];
  }
  if (catalogue === null || typeof catalogue !== 'object' || Array.isArray(catalogue)) {
    return [`${fileName}: not an object of messages`];
  }
  const entries = Object.entries(catalogue);
  const problems = [
    ...entries.filter(([, value]) => typeof value !== 'string').map(([key]) => `not flat: ${key}`),
    ...entries.filter(([key]) => !KEY_PATTERN.test(key)).map(([key]) => `invalid key: ${key}`),
  ];
  if (!isSorted(entries.map(([key]) => key))) {
    problems.push('keys are not sorted');
  } else if (problems.length === 0 && text !== formatCatalogue(catalogue)) {
    problems.push('not one key per line with two-space indentation, or a key is repeated');
  }
  return problems.map((problem) => `${fileName}: ${problem}`);
}

/** Checks that a catalogue holds exactly the keys of the source catalogue. */
export function checkSameKeys(fileName, catalogue, sourceCatalogue) {
  const keys = new Set(Object.keys(catalogue));
  const sourceKeys = new Set(Object.keys(sourceCatalogue));
  const missing = [...sourceKeys].filter((key) => !keys.has(key));
  const extra = [...keys].filter((key) => !sourceKeys.has(key));
  return [
    ...missing.map((key) => `${fileName}: missing ${key}`),
    ...extra.map((key) => `${fileName}: ${key} is not in the source catalogue`),
  ];
}

/** Checks that every message parses as ICU MessageFormat for the catalogue's language. */
export function checkMessages(fileName, code, catalogue) {
  const options = {
    cardinal: new Intl.PluralRules(code).resolvedOptions().pluralCategories,
    ordinal: new Intl.PluralRules(code, { type: 'ordinal' }).resolvedOptions().pluralCategories,
  };
  return Object.entries(catalogue).flatMap(([key, message]) => {
    try {
      const tokens = parse(message, options);
      return key.startsWith(EMAIL_PREFIX) && !tokens.every(isSimpleToken)
        ? [`${fileName}: ${key} uses more than simple {name} arguments`]
        : [];
    } catch (error) {
      return [`${fileName}: ${key} does not parse (${error.message.split('\n')[0]})`];
    }
  });
}

/** Checks that `languages.json` lists each catalogue exactly once, with a name. */
export function checkLanguageList(languages, catalogueCodes) {
  if (!Array.isArray(languages)) {
    return ['languages.json: not a list'];
  }
  const listed = languages.map((language) => language?.code);
  const problems = languages
    .filter((language) => typeof language?.code !== 'string' || typeof language?.name !== 'string')
    .map((language) => `languages.json: needs a code and a name: ${JSON.stringify(language)}`);
  const repeated = listed.filter((code, index) => listed.indexOf(code) !== index);
  return [
    ...problems,
    ...repeated.map((code) => `languages.json: ${code} is listed twice`),
    ...catalogueCodes
      .filter((code) => !listed.includes(code))
      .map((code) => `languages.json: misses ${code}, whose catalogue exists`),
    ...listed
      .filter((code) => typeof code === 'string' && !catalogueCodes.includes(code))
      .map((code) => `languages.json: lists ${code}, which has no catalogue`),
  ];
}

function isSimpleToken(token) {
  return SIMPLE_TOKEN_TYPES.has(token.type);
}

function isSorted(keys) {
  return keys.every((key, index) => index === 0 || compare(keys[index - 1], key) < 0);
}

function compare(a, b) {
  if (a === b) {
    return 0;
  }
  return a < b ? -1 : 1;
}
