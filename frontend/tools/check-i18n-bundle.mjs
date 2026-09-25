// Checks the app's build: no catalogue is compiled into its JavaScript, and every catalogue is
// served as a file at /i18n/. Run after `npm run build`.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { findBundledEntries } from './i18n/bundle-rules.mjs';

const I18N_DIR = new URL('../../i18n/', import.meta.url);
const BROWSER_DIR = new URL('../dist/app/browser/', import.meta.url);

if (!existsSync(BROWSER_DIR)) {
  console.error('dist/app/browser/ is missing: run `npm run build` first.');
  process.exit(1);
}
const catalogueFiles = readdirSync(I18N_DIR).filter((name) => name.endsWith('.json'));
const keys = catalogueFiles
  .filter((name) => name !== 'languages.json')
  .flatMap((name) => Object.keys(JSON.parse(readFileSync(new URL(name, I18N_DIR), 'utf8'))));
const scripts = readdirSync(BROWSER_DIR, { recursive: true })
  .filter((name) => name.endsWith('.js'))
  .map((name) => ({ name, text: readFileSync(new URL(name, BROWSER_DIR), 'utf8') }));

const problems = [
  ...catalogueFiles
    .filter((name) => !existsSync(new URL(`i18n/${name}`, BROWSER_DIR)))
    .map((name) => `i18n/${name} is not served by the build`),
  ...findBundledEntries(scripts, [...new Set(keys)]).map((entry) => `bundled catalogue: ${entry}`),
];
if (problems.length > 0) {
  problems.forEach((problem) => console.error(problem));
  process.exit(1);
}
console.log(
  `i18n: no catalogue in ${scripts.length} scripts; ${catalogueFiles.length} files served.`,
);
