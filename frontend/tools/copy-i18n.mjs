// Copies the catalogues of ../i18n into the app's public folder, so that `ng serve`, a static
// build and the desktop app serve them at /i18n/. Angular refuses asset inputs outside the
// workspace, and the catalogues stay in one place at the root of the repository.
import { cpSync, rmSync } from 'node:fs';

const SOURCE = new URL('../../i18n/', import.meta.url);
const TARGET = new URL('../projects/app/public/i18n/', import.meta.url);

rmSync(TARGET, { recursive: true, force: true });
cpSync(SOURCE, TARGET, { recursive: true });
