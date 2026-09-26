import assert from 'node:assert/strict';
import { readFile, readdir, realpath, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPOSITORY = fileURLToPath(new URL('../../', import.meta.url));
const DESIGN_ROOT = path.join(REPOSITORY, 'design-system');
const PREVIEW_ROOT = path.join(DESIGN_ROOT, 'preview');
const KEY_PREFIX = 'design_system.';
const TEXT_RATIO = 4.5;
const CONTROL_RATIO = 3;
const SURFACES = ['bg', 'surface', 'surface-soft', 'surface-raised'];
const SEMANTICS = ['success', 'warning', 'danger', 'info'];
const COUNTS = { contrasts: 0, keys: 0, references: 0 };

async function text(filename) {
  return readFile(filename, 'utf8');
}

function isContained(root, filename) {
  const relative = path.relative(root, filename);
  return relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    if (entry.name.startsWith('.')) continue;
    const filename = path.join(directory, entry.name);
    assert(!entry.isSymbolicLink(), `Unexpected symbolic link: ${filename}`);
    if (entry.isDirectory()) files.push(...(await listFiles(filename)));
    else files.push(filename);
  }
  return files;
}

function colourBlock(css, theme) {
  const selector = `:root\\[data-theme=['"]${theme}['"]\\]\\s*\\{([^}]+)\\}`;
  const match = css.match(new RegExp(selector));
  assert(match, `Missing explicit ${theme} theme`);
  return Object.fromEntries(
    [...match[1].matchAll(/--lp-([a-z-]+):\s*(#[\da-f]{6})\s*;/gi)].map((entry) => [
      entry[1],
      entry[2],
    ]),
  );
}

function luminance(hex) {
  assert(/^#[\da-f]{6}$/i.test(hex ?? ''), `Expected six-digit colour, received ${hex}`);
  const channels = hex
    .slice(1)
    .match(/../g)
    .map((part) => Number.parseInt(part, 16) / 255);
  const linear = channels.map((value) =>
    value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
  );
  return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}

function contrast(tokens, pair, theme) {
  const [foreground, background, threshold] = pair;
  const values = [luminance(tokens[foreground]), luminance(tokens[background])];
  const ratio = (Math.max(...values) + 0.05) / (Math.min(...values) + 0.05);
  const label = `${theme}: ${foreground} on ${background}`;
  assert(ratio >= threshold, `${label} is ${ratio.toFixed(2)}:1; requires ${threshold}:1`);
  COUNTS.contrasts += 1;
}

function contrastPairs() {
  const pairs = SURFACES.flatMap((background) => [
    ['text', background, TEXT_RATIO],
    ['text-muted', background, TEXT_RATIO],
    ['focus', background, CONTROL_RATIO],
    ['control-border', background, CONTROL_RATIO],
    ['accent', background, TEXT_RATIO],
  ]);
  pairs.push(['on-accent', 'accent', TEXT_RATIO], ['on-accent', 'accent-hover', TEXT_RATIO]);
  pairs.push(['accent-text', 'accent-soft', TEXT_RATIO]);
  pairs.push(['disabled-text', 'disabled-bg', TEXT_RATIO]);
  for (const semantic of SEMANTICS) {
    pairs.push([semantic, `${semantic}-bg`, TEXT_RATIO]);
    pairs.push([semantic, 'surface', TEXT_RATIO]);
  }
  return pairs;
}

async function checkColours() {
  const css = await text(path.join(DESIGN_ROOT, 'tokens', 'tokens.css'));
  const light = colourBlock(css, 'light');
  const dark = { ...light, ...colourBlock(css, 'dark') };
  for (const [theme, tokens] of Object.entries({ light, dark })) {
    for (const pair of contrastPairs()) contrast(tokens, pair, theme);
  }
}

async function readCatalogues() {
  const languages = await Promise.all(
    ['en', 'fr'].map(async (language) => {
      const catalogue = JSON.parse(await text(path.join(REPOSITORY, 'i18n', `${language}.json`)));
      assert(Object.values(catalogue).every((value) => typeof value === 'string'));
      return catalogue;
    }),
  );
  assert.deepEqual(
    Object.keys(languages[0]).sort(),
    Object.keys(languages[1]).sort(),
    'English and French catalogue keys differ',
  );
  const available = JSON.parse(await text(path.join(REPOSITORY, 'i18n', 'languages.json')));
  assert(Array.isArray(available), 'The language list must be an array');
  assert(['en', 'fr'].every((code) => available.some((language) => language.code === code)));
  return languages[0];
}

function referencedKeys(source) {
  const attributes = /data-(?:i18n(?:-[\w-]+)?|feedback)=["']([^"']+)["']/g;
  const calls = /\b(?:t|translate|announce|notify)\(\s*["']([\w.-]+)["']/g;
  return [...source.matchAll(attributes), ...source.matchAll(calls)]
    .map((match) => match[1])
    .filter((key) => /^[a-z][\w.-]*$/.test(key));
}

async function checkTranslations(files) {
  const catalogue = await readCatalogues();
  const used = new Set();
  for (const filename of files.filter((file) => /\.(?:html|m?js)$/.test(file))) {
    for (const shortKey of referencedKeys(await text(filename))) {
      const key = shortKey.startsWith(KEY_PREFIX) ? shortKey : `${KEY_PREFIX}${shortKey}`;
      assert(Object.hasOwn(catalogue, key), `Missing translation ${key} in ${filename}`);
      assert(catalogue[key].trim().length > 0, `Empty translation: ${key}`);
      used.add(key);
    }
  }
  assert(used.size > 0, 'No preview translation references were checked');
  COUNTS.keys = used.size;
}

function assetReferences(source, extension) {
  if (extension === '.html') {
    return [...source.matchAll(/\b(?:src|href)=["']([^"']+)["']/g)].map((match) => match[1]);
  }
  if (extension === '.css') {
    return [...source.matchAll(/url\(\s*["']?([^"')\s]+)["']?\s*\)/g)].map((match) => match[1]);
  }
  return [...source.matchAll(/\bfrom\s+["'](\.[^"']+)["']/g)].map((match) => match[1]);
}

function resolveAsset(filename, reference) {
  if (/^(?:[a-z]+:|#|\/\/)/i.test(reference)) return null;
  const pathname = decodeURIComponent(reference.split(/[?#]/)[0]);
  if (!pathname) return null;
  // Section fragments are inserted into index.html, so their URLs share its document base.
  const htmlBase = path.join(PREVIEW_ROOT, 'index.html');
  const base = filename.endsWith('.html') ? htmlBase : filename;
  return pathname.startsWith('/')
    ? path.resolve(REPOSITORY, `.${pathname}`)
    : path.resolve(path.dirname(base), pathname);
}

async function checkReference(filename, reference) {
  const target = resolveAsset(filename, reference);
  if (!target) return;
  assert(isContained(REPOSITORY, target), `Asset escapes repository: ${reference}`);
  let canonical;
  try {
    canonical = await realpath(target);
  } catch {
    assert.fail(`Missing asset ${reference} referenced in ${filename}`);
  }
  assert(isContained(REPOSITORY, canonical), `Asset symlink escapes repository: ${reference}`);
  assert((await stat(canonical)).isFile(), `Asset is not a file: ${reference}`);
  COUNTS.references += 1;
}

async function checkAssets(files) {
  for (const filename of files) {
    const extension = path.extname(filename);
    if (!['.html', '.css', '.js', '.mjs'].includes(extension)) continue;
    const references = assetReferences(await text(filename), extension);
    for (const reference of references) await checkReference(filename, reference);
  }
}

async function checkInstructions() {
  const instructions = await Promise.all(
    ['AGENTS.md', 'CLAUDE.md'].map((filename) => readFile(path.join(REPOSITORY, filename))),
  );
  assert(instructions[0].equals(instructions[1]), 'AGENTS.md and CLAUDE.md differ');
}

async function main() {
  const files = await listFiles(DESIGN_ROOT);
  const previewFiles = files.filter((filename) => isContained(PREVIEW_ROOT, filename));
  await checkColours();
  await checkTranslations(previewFiles);
  await checkAssets(files);
  await checkInstructions();
  console.log(`PASS: ${COUNTS.contrasts} contrast pairs, ${COUNTS.keys} translated preview keys,`);
  console.log(`${COUNTS.references} local asset references; catalogues and instructions match.`);
  console.log('Static checks only; dynamic keys and browser accessibility need manual review.');
}

await main().catch((error) => {
  console.error(`FAIL: ${error.message}`);
  process.exitCode = 1;
});
