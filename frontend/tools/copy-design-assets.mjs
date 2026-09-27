// Copies the design system's production illustrations, pixel identity and bundled Nunito font
// with its licence into the app's public folder, so that `ng serve`, a
// static build and the desktop app serve them at /design-system/. Angular refuses asset inputs
// outside the workspace, and the assets stay in one place, in design-system/assets/.
import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE = fileURLToPath(new URL('../../design-system/assets/', import.meta.url));
const TARGET = fileURLToPath(new URL('../projects/app/public/design-system/', import.meta.url));

/** The files the app uses, relative to design-system/assets/. */
export const DESIGN_ASSETS = [
  'mark.svg',
  'pip.svg',
  'sprout.svg',
  'illustrations/pip-atelier.webp',
  'illustrations/pip-library.svg',
  'illustrations/pip-export.svg',
  'illustrations/missing-frame.svg',
  'illustrations/appearance/light.svg',
  'illustrations/appearance/dark.svg',
  'illustrations/appearance/system.svg',
  'fonts/nunito-variable.ttf',
  'fonts/OFL.txt',
];

/**
 * Replaces `target` with a copy of every asset of `source`, and returns the files written. Fails
 * before touching `target` when an asset is missing, so a stale copy never hides a lost source.
 */
export function copyDesignAssets(source, target) {
  const missing = DESIGN_ASSETS.filter((asset) => !existsSync(join(source, asset)));
  if (missing.length > 0) {
    throw new Error(`copy-design-assets: missing in ${source}: ${missing.join(', ')}`);
  }
  rmSync(target, { recursive: true, force: true });
  return DESIGN_ASSETS.map((asset) => {
    const file = join(target, asset);
    mkdirSync(dirname(file), { recursive: true });
    cpSync(join(source, asset), file);
    return file;
  });
}

if (process.argv[1] !== undefined && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  copyDesignAssets(SOURCE, TARGET);
}
