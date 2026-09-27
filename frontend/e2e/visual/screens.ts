import { expect, type Page } from '@playwright/test';
import { relative, resolve } from 'node:path';
import { OUTPUT_DIR, VIEWPORTS, type Viewport } from './visual-matrix';

/** The page a route shows: Ionic keeps the previous ones in the outlet, hidden. */
export const ROUTED_PAGE = 'ion-router-outlet > .ion-page:not(.ion-page-hidden)';

/** Waits until layout, fonts and two frames have settled after a change of size or screen. */
export async function settle(page: Page): Promise<void> {
  await page.evaluate(async () => {
    await document.fonts.ready;
    await new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
  });
}

/** Opens a routed page and waits for its level-1 heading, when it has one. */
export async function openRoute(page: Page, path: string): Promise<void> {
  await page.goto(path);
  const heading = page.locator(ROUTED_PAGE).getByRole('heading', { level: 1 }).first();
  await expect(heading)
    .toBeVisible({ timeout: 10_000 })
    .catch(() => undefined);
  await settle(page);
}

/**
 * Captures what the screen shows at every width of the matrix, then restores the widest one;
 * returns the files' paths relative to the output directory.
 */
export async function captureAtEveryWidth(
  page: Page,
  folder: string,
  screen: string,
): Promise<string[]> {
  const paths: string[] = [];
  for (const viewport of VIEWPORTS) {
    paths.push(await captureAt(page, `${folder}/${screen}`, viewport));
  }
  await page.setViewportSize(VIEWPORTS[0]);
  await settle(page);
  return paths;
}

/**
 * Captures what the screen shows at one width, once settled, as `<screen>-<width>.png` under the
 * output directory; returns the file's path relative to it.
 */
export async function captureAt(page: Page, screen: string, viewport: Viewport): Promise<string> {
  await page.setViewportSize(viewport);
  await settle(page);
  const path = resolve(OUTPUT_DIR, `${screen}-${viewport.width}.png`);
  await page.screenshot({ path, animations: 'disabled', caret: 'hide' });
  return relative(OUTPUT_DIR, path).replaceAll('\\', '/');
}
