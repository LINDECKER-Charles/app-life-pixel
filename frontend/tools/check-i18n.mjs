// Checks the catalogues of i18n/ against docs/i18n.md: exactly the keys of en.json everywhere,
// flat sorted keys, ICU messages that parse, simple arguments in emails, and a languages.json
// that lists exactly the catalogues. Reports the keys no source file names.
import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import {
  checkCatalogueText,
  checkLanguageList,
  checkMessages,
  checkSameKeys,
} from './i18n/catalogue-rules.mjs';
import { checkLegalPages, LEGAL_PAGES } from './i18n/legal-rules.mjs';
import { findUnusedKeys } from './i18n/unused-keys.mjs';

const FRONTEND_DIR = new URL('../', import.meta.url);
const I18N_DIR = new URL('../../i18n/', import.meta.url);
const LEGAL_DIR = new URL('legal/', I18N_DIR);
const LANGUAGES_FILE = 'languages.json';
const SOURCE_CODE = 'en';

function readCatalogues() {
  return readdirSync(I18N_DIR)
    .filter((name) => name.endsWith('.json') && name !== LANGUAGES_FILE)
    .map((fileName) => ({
      fileName,
      code: fileName.slice(0, -'.json'.length),
      text: readFileSync(new URL(fileName, I18N_DIR), 'utf8'),
    }));
}

function checkAll(catalogues) {
  const shapeProblems = catalogues.flatMap(({ fileName, text }) =>
    checkCatalogueText(fileName, text),
  );
  if (shapeProblems.length > 0) {
    return shapeProblems;
  }
  const parsed = catalogues.map((file) => ({ ...file, messages: JSON.parse(file.text) }));
  const source = parsed.find(({ code }) => code === SOURCE_CODE);
  if (source === undefined) {
    return [`${SOURCE_CODE}.json: the source catalogue is missing`];
  }
  const languages = JSON.parse(readFileSync(new URL(LANGUAGES_FILE, I18N_DIR), 'utf8'));
  return [
    ...checkLanguageList(
      languages,
      parsed.map(({ code }) => code),
    ),
    ...parsed.flatMap(({ fileName, messages }) =>
      checkSameKeys(fileName, messages, source.messages),
    ),
    ...parsed.flatMap(({ fileName, code, messages }) => checkMessages(fileName, code, messages)),
    ...checkLegalPages(readLegalPages(parsed.map(({ code }) => code))),
  ];
}

function readLegalPages(codes) {
  return Object.fromEntries(
    codes.map((code) => [
      code,
      Object.fromEntries(LEGAL_PAGES.map((page) => [page, readLegalPage(code, page)])),
    ]),
  );
}

function readLegalPage(code, page) {
  try {
    return readFileSync(new URL(`${code}/${page}.md`, LEGAL_DIR), 'utf8');
  } catch {
    return undefined;
  }
}

function readSourceTexts() {
  return execFileSync('git', ['ls-files', '--', 'projects'], {
    cwd: FRONTEND_DIR,
    encoding: 'utf8',
  })
    .split('\n')
    .filter((file) => /\.(ts|html)$/.test(file) && !file.endsWith('.spec.ts'))
    .map((file) => readFileSync(new URL(file, FRONTEND_DIR), 'utf8'));
}

function reportUnusedKeys(sourceCatalogue) {
  let texts;
  try {
    texts = readSourceTexts();
  } catch {
    console.log('Source files not listed by Git: unused keys not reported.');
    return;
  }
  const unused = findUnusedKeys(Object.keys(JSON.parse(sourceCatalogue.text)), texts);
  if (unused.length > 0) {
    console.log(`Keys no source file names yet (kept while an app may use them):`);
    unused.forEach((key) => console.log(`  ${key}`));
  }
}

const catalogues = readCatalogues();
const problems = checkAll(catalogues);
if (problems.length > 0) {
  problems.forEach((problem) => console.error(problem));
  process.exit(1);
}
reportUnusedKeys(catalogues.find(({ code }) => code === SOURCE_CODE));
console.log(`i18n: ${catalogues.length} catalogues checked.`);
