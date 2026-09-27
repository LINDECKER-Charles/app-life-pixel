// Merges the translation fragments of agents working in parallel into the catalogues
// (i18n-pending/README.md): each i18n-pending/<lot>/<code>.json adds or replaces keys of
// i18n/<code>.json, then the lot's removed.txt drops keys from every catalogue; lots apply in
// name order. The catalogues are written sorted, in the format of catalogue-rules.mjs, and the
// fragments deleted — unless `--keep`. Without any fragment, nothing changes.
import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { formatCatalogue } from './i18n/catalogue-rules.mjs';

const REPOSITORY = fileURLToPath(new URL('../../', import.meta.url));
const PENDING_DIR = 'i18n-pending';
const I18N_DIR = 'i18n';
const REMOVED_FILE = 'removed.txt';
const LANGUAGES_FILE = 'languages.json';
const KEEP_FLAG = '--keep';

/** The fragments waiting in i18n-pending/, one per lot folder, in name order. */
export function readFragments(repository) {
  const pending = join(repository, PENDING_DIR);
  if (!existsSync(pending)) return [];
  return readdirSync(pending, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort()
    .map((lot) => readFragment(join(pending, lot), lot))
    .filter((fragment) => fragment.files.length > 0);
}

function readFragment(folder, lot) {
  const files = readdirSync(folder).filter(
    (name) => name.endsWith('.json') || name === REMOVED_FILE,
  );
  const messages = Object.fromEntries(
    files
      .filter((name) => name.endsWith('.json'))
      .map((name) => [
        name.slice(0, -'.json'.length),
        readMessages(join(folder, name), `${PENDING_DIR}/${lot}/${name}`),
      ]),
  );
  const removed = files.includes(REMOVED_FILE) ? readRemoved(join(folder, REMOVED_FILE)) : [];
  return { lot, folder, files, messages, removed };
}

function readMessages(path, label) {
  let value;
  try {
    value = JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    throw new Error(`${label}: not valid JSON (${error.message})`, { cause: error });
  }
  const isObject = value !== null && typeof value === 'object' && !Array.isArray(value);
  if (!isObject || Object.values(value).some((message) => typeof message !== 'string')) {
    throw new Error(`${label}: not a flat object of messages`);
  }
  return value;
}

/** The keys of a removed.txt: one per line; blank lines and `#` comments are skipped. */
function readRemoved(path) {
  return readFileSync(path, 'utf8')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line !== '' && !line.startsWith('#'));
}

/** The catalogues of i18n/, by language code. */
function readCatalogues(repository) {
  const folder = join(repository, I18N_DIR);
  return Object.fromEntries(
    readdirSync(folder)
      .filter((name) => name.endsWith('.json') && name !== LANGUAGES_FILE)
      .map((name) => [
        name.slice(0, -'.json'.length),
        readMessages(join(folder, name), `${I18N_DIR}/${name}`),
      ]),
  );
}

/** Applies the fragments, in order, to copies of the catalogues. */
export function applyFragments(catalogues, fragments) {
  const merged = Object.fromEntries(
    Object.entries(catalogues).map(([code, messages]) => [code, { ...messages }]),
  );
  for (const { lot, messages, removed } of fragments) {
    for (const [code, added] of Object.entries(messages)) {
      if (!(code in merged)) {
        throw new Error(`${PENDING_DIR}/${lot}/${code}.json: there is no ${I18N_DIR}/${code}.json`);
      }
      Object.assign(merged[code], added);
    }
    for (const catalogue of Object.values(merged)) {
      for (const key of removed) delete catalogue[key];
    }
  }
  return merged;
}

/** Every message the fragments set, in order: one entry per language and key. */
function settings(fragments) {
  return fragments.flatMap(({ lot, messages }) =>
    Object.entries(messages).flatMap(([code, added]) =>
      Object.entries(added).map(([key, message]) => ({ lot, code, key, message })),
    ),
  );
}

/** Keys two lots set to different messages: the later lot wins, which deserves a look. */
export function findConflicts(fragments) {
  const seen = new Map();
  const conflicts = [];
  for (const { lot, code, key, message } of settings(fragments)) {
    const earlier = seen.get(`${code}:${key}`);
    if (earlier && earlier.message !== message) {
      conflicts.push(`${code}.json: ${key} set by ${earlier.lot}, replaced by ${lot}`);
    }
    seen.set(`${code}:${key}`, { lot, message });
  }
  return conflicts;
}

/** Keys a lot translates in some of its languages but not in all of them. */
export function findUnpaired({ lot, messages }) {
  const codes = Object.keys(messages);
  const keys = new Set(codes.flatMap((code) => Object.keys(messages[code])));
  return [...keys].flatMap((key) => {
    const missing = codes.filter((code) => !(key in messages[code]));
    return missing.length === 0 ? [] : [`${lot}: ${key} is missing from ${missing.join(', ')}`];
  });
}

function deleteFragment({ folder, files }) {
  for (const name of files) rmSync(join(folder, name));
  if (readdirSync(folder).length === 0) rmSync(folder, { recursive: true });
}

/**
 * Merges every fragment of `repository` into its catalogues and, unless `keep`, deletes them.
 * Returns the lots merged and the warnings worth a look.
 */
export function mergeFragments(repository, { keep = false } = {}) {
  const fragments = readFragments(repository);
  if (fragments.length === 0) return { lots: [], warnings: [] };
  const merged = applyFragments(readCatalogues(repository), fragments);
  for (const [code, messages] of Object.entries(merged)) {
    writeFileSync(join(repository, I18N_DIR, `${code}.json`), formatCatalogue(messages));
  }
  if (!keep) fragments.forEach(deleteFragment);
  const warnings = [...findConflicts(fragments), ...fragments.flatMap(findUnpaired)];
  return { lots: fragments.map(({ lot }) => lot), warnings };
}

function main(argv) {
  if (argv.some((argument) => argument !== KEEP_FLAG)) {
    console.error(`usage: node tools/merge-i18n-fragments.mjs [${KEEP_FLAG}]`);
    return 2;
  }
  const keep = argv.includes(KEEP_FLAG);
  const { lots, warnings } = mergeFragments(REPOSITORY, { keep });
  for (const warning of warnings) console.warn(`merge-i18n-fragments: warning: ${warning}`);
  if (lots.length === 0) {
    console.log(`merge-i18n-fragments: no fragment in ${PENDING_DIR}/, nothing to do.`);
  } else {
    const kept = keep ? ', fragments kept' : ', fragments deleted';
    console.log(`merge-i18n-fragments: merged ${lots.join(', ')}${kept}.`);
  }
  return 0;
}

if (process.argv[1] !== undefined && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(`merge-i18n-fragments: ${error.message}`);
    process.exitCode = 1;
  }
}
