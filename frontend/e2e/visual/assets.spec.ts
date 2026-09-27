import { expect, test, type Page } from '@playwright/test';
import { AssetCaptures } from './asset-captures';
import { AssetLibrary } from './asset-library';
import { label } from './labels';
import { openRoute } from './screens';
import { VisualEditor } from './visual-editor';
import { LANGUAGES, THEMES, VIEWPORTS, type Language } from './visual-matrix';

const LIBRARY_ILLUSTRATION = 'lp-animation-list lp-empty-state';
const EXPORT_ILLUSTRATION = 'lp-export-download-done';

test.describe.configure({ mode: 'parallel' });

async function captureLibrary(
  page: Page,
  language: Language,
  captures: AssetCaptures,
): Promise<void> {
  const library = new AssetLibrary(page, language);
  await library.install();
  await openRoute(page, '/library');
  await expect(page.getByText(label(language, 'library.projects.empty'))).toBeVisible();
  await expect(page.locator(LIBRARY_ILLUSTRATION)).toBeVisible();
  await captures.everyWidth('library-empty', LIBRARY_ILLUSTRATION);
  library.populate();
  await openRoute(page, '/library');
  await expect(page.getByRole('link', { name: library.projectName, exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: library.animationName, exact: true })).toBeVisible();
  await captures.everyWidth('library-populated');
  await openRoute(page, library.projectPath);
  await expect(page.getByRole('heading', { name: library.projectName })).toBeVisible();
  await expect(page.getByRole('link', { name: library.animationName, exact: true })).toBeVisible();
  await captures.everyWidth('library-project');
  library.emptyProject();
  await openRoute(page, library.projectPath);
  await expect(page.locator(LIBRARY_ILLUSTRATION)).toBeVisible();
  await captures.everyWidth('library-project-empty', LIBRARY_ILLUSTRATION);
}

async function finishExport(page: Page, language: Language): Promise<void> {
  const editor = new VisualEditor(page, language);
  if ((await editor.open()) === 'welcome') await editor.startFromWelcome();
  await editor.create();
  await editor.drawTwoFrames();
  await editor.openExport();
  // Ionic's dialog role lives in its shadow root; the table is projected through a slot.
  const gifRow = page
    .getByRole('table', { name: label(language, 'export.table.caption') })
    .getByRole('row')
    .filter({ has: page.getByRole('rowheader', { name: /^GIF\b/ }) });
  const button = gifRow.getByRole('button', { name: label(language, 'export.table.download') });
  await expect(button).toBeEnabled();
  const [download] = await Promise.all([page.waitForEvent('download'), button.click()]);
  expect(await download.failure()).toBeNull();
  await expect(page.locator(EXPORT_ILLUSTRATION)).toBeVisible();
  await expect(page.locator(EXPORT_ILLUSTRATION)).toContainText(
    label(language, 'export.download.done', { format: 'GIF' }),
  );
}

for (const { code, locale } of LANGUAGES) {
  for (const theme of THEMES) {
    test.describe(`assets: ${code}, ${theme} theme`, () => {
      test.use({ locale, colorScheme: theme, reducedMotion: 'reduce', viewport: VIEWPORTS[0] });

      test('captures empty and populated libraries and projects', async ({ page }) => {
        const captures = new AssetCaptures(page, `assets/${code}-${theme}/library`);
        try {
          await captureLibrary(page, code, captures);
        } finally {
          captures.report();
        }
      });

      test('captures the illustration after a successful export', async ({ page }) => {
        const captures = new AssetCaptures(page, `assets/${code}-${theme}/export`);
        try {
          await finishExport(page, code);
          await captures.everyWidth('export-done', EXPORT_ILLUSTRATION);
        } finally {
          captures.report();
        }
      });
    });
  }
}

test('serves the declared SVG, ICO and Apple touch identity icons', async ({ page, request }) => {
  await page.goto('/');
  const svg = page.locator('link[rel="icon"][type="image/svg+xml"]');
  const ico = page.locator('link[rel="icon"][type="image/x-icon"]');
  const apple = page.locator('link[rel="apple-touch-icon"]');
  await expect(svg).toHaveAttribute('href', 'favicon.svg');
  await expect(ico).toHaveAttribute('sizes', '16x16 32x32 48x48');
  await expect(apple).toHaveAttribute('sizes', '180x180');
  const svgResponse = await request.get('/favicon.svg');
  expect(svgResponse.ok()).toBe(true);
  expect(svgResponse.headers()['content-type']).toContain('image/svg+xml');
  expect(await svgResponse.text()).toContain('shape-rendering="crispEdges"');
  const icoResponse = await request.get('/favicon.ico');
  expect(icoResponse.ok()).toBe(true);
  const icoBytes = await icoResponse.body();
  expect(icoBytes.readUInt16LE(2)).toBe(1);
  expect(icoBytes.readUInt16LE(4)).toBe(3);
  const appleResponse = await request.get('/apple-touch-icon.png');
  expect(appleResponse.ok()).toBe(true);
  const appleBytes = await appleResponse.body();
  expect(appleBytes.readUInt32BE(16)).toBe(180);
  expect(appleBytes.readUInt32BE(20)).toBe(180);
});
