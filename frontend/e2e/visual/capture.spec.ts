import { expect, test, type Page } from '@playwright/test';
import { measureScroll, measureStage } from './measurements';
import { captureAtEveryWidth, openRoute } from './screens';
import { VisualEditor } from './visual-editor';
import { LANGUAGES, THEMES, VIEWPORTS, writeJson, type Language } from './visual-matrix';

// C12 (docs/plans, "visual" project): screenshots of the real app, engine included, for review.
// Nothing compares pixels: a run fails only when a page throws an uncaught error.

/** The editor's canvas stage is measured on common laptop screens. */
const STAGE_VIEWPORTS = [
  { width: 1366, height: 768 },
  { width: 1024, height: 768 },
];
/** A small phone, on which every routed page must scroll down to its end. */
const PHONE = { width: 390, height: 640 };
/** Routed pages reachable without the hosted stack: without `ion-content`, then with it. */
const SCROLLED_PATHS = [
  '/sign-in',
  '/sign-up',
  '/reset-password',
  '/verify-email',
  '/library',
  '/settings',
  '/legal/terms',
  '/visual-review-missing-page',
];

test.describe.configure({ mode: 'parallel' });

/** Collects the page's uncaught errors and console errors from now on. */
function watchErrors(page: Page): { pageErrors: string[]; consoleErrors: string[] } {
  const errors = { pageErrors: [] as string[], consoleErrors: [] as string[] };
  page.on('pageerror', (error) => errors.pageErrors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.consoleErrors.push(message.text());
  });
  return errors;
}

async function captureJourney(page: Page, language: Language, folder: string): Promise<string[]> {
  const captures: string[] = [];
  const capture = async (screen: string): Promise<void> => {
    captures.push(...(await captureAtEveryWidth(page, folder, screen)));
  };
  await openRoute(page, '/settings');
  await capture('settings');
  await openRoute(page, '/visual-review-missing-page');
  await capture('not-found');

  const editor = new VisualEditor(page, language);
  if ((await editor.open()) === 'welcome') {
    await capture('editor-welcome');
    await editor.startFromWelcome();
  }
  await capture('editor-new-animation');
  await editor.create();
  await editor.drawTwoFrames();
  await capture('editor');
  await editor.openExport();
  await capture('export');
  return captures;
}

for (const { code, locale } of LANGUAGES) {
  for (const theme of THEMES) {
    test.describe(`${code}, ${theme} theme`, () => {
      test.use({ locale, colorScheme: theme, reducedMotion: 'reduce', viewport: VIEWPORTS[0] });

      test('captures every screen at every width', async ({ page }) => {
        const errors = watchErrors(page);
        const folder = `screens/${code}-${theme}`;
        let captures: string[] = [];
        try {
          captures = await captureJourney(page, code, folder);
        } finally {
          const applied = await page.evaluate(() => ({
            language: document.documentElement.lang,
            dark: matchMedia('(prefers-color-scheme: dark)').matches,
          }));
          writeJson(`${folder}/report.json`, { code, theme, applied, captures, ...errors });
        }
        expect(errors.pageErrors, 'uncaught errors on the page').toEqual([]);
      });
    });
  }
}

test.describe('measurements', () => {
  test.use({ locale: 'en-US', colorScheme: 'light', reducedMotion: 'reduce' });

  test('measures the canvas stage and the scrolling of routed pages', async ({ page }) => {
    const errors = watchErrors(page);
    // Leaving the editor with unsaved work asks first (D37): the measurements leave it anyway.
    page.on('dialog', (dialog) => void dialog.accept());
    const editor = new VisualEditor(page, 'en');
    await page.setViewportSize(STAGE_VIEWPORTS[0]);
    if ((await editor.open()) === 'welcome') await editor.startFromWelcome();
    await editor.create();
    const stage = [];
    for (const viewport of STAGE_VIEWPORTS) stage.push(await measureStage(editor, viewport));

    await page.setViewportSize(PHONE);
    const scrolling = [];
    for (const path of SCROLLED_PATHS) scrolling.push(await measureScroll(page, path));

    writeJson('measurements.json', { stage, phone: PHONE, scrolling, ...errors });
    expect(errors.pageErrors, 'uncaught errors on the page').toEqual([]);
  });
});
