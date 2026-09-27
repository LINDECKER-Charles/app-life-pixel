import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { finiteAnimationsEnded } from './animations';
import { EditorPage } from './editor-page';

/** WCAG 2.2 AA, the target of AGENTS.md, and every level below it. */
const WCAG_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const BLOCKING_IMPACTS = new Set(['serious', 'critical']);

/** The serious and critical violations axe finds on the page as it is now, one line each. */
async function blockingViolations(page: Page): Promise<string[]> {
  // A dialog fading in has the contrast of its opacity: axe waits for its end, as the hosted
  // suite's does (e2e/hosted/accessibility.ts).
  await page.waitForFunction(finiteAnimationsEnded);
  const results = await new AxeBuilder({ page }).withTags(WCAG_TAGS).analyze();
  return results.violations
    .filter((violation) => BLOCKING_IMPACTS.has(violation.impact ?? ''))
    .map(
      (violation) =>
        `${violation.id} (${violation.impact}): ${violation.help} — ` +
        violation.nodes.map((node) => node.target.join(' ')).join(', '),
    );
}

/** The inspector's panels, one at a time behind a tab below 75 rem (editor.md, U1). */
const INSPECTOR_TABS = ['Palette', 'Layers', 'Preview'] as const;
/** A phone's viewport: the editor's single-column layout, below 48 rem. */
const PHONE = { width: 390, height: 844 };

// axe on every screen of the editor path (editor.md, U6): no serious or critical violation.
test.describe('accessibility', () => {
  test('the welcome of a first visit', async ({ page }) => {
    await new EditorPage(page).goto();
    expect(await blockingViolations(page)).toEqual([]);
  });

  test.describe('with an animation open', () => {
    let editor: EditorPage;

    test.beforeEach(async ({ page }) => {
      editor = new EditorPage(page);
      await editor.open();
      await editor.create();
    });

    test('the editor', async ({ page }) => {
      expect(await blockingViolations(page)).toEqual([]);
    });

    test('the export dialog', async ({ page }) => {
      await editor.button('Export').click();
      await expect(editor.exportDialog).toBeVisible();
      await editor.expectEveryFormatSized();
      expect(await blockingViolations(page)).toEqual([]);
    });

    test('the shortcuts dialog', async ({ page }) => {
      await editor.button('Keyboard shortcuts').click();
      await expect(editor.shortcutsDialog).toBeVisible();
      await expect(page.getByRole('listitem').filter({ hasText: 'Zoom in' })).toBeVisible();
      expect(await blockingViolations(page)).toEqual([]);
    });

    test('the settings', async ({ page }) => {
      await page.getByRole('link', { name: 'Settings' }).click();
      await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
      // Ionic animates the pages' change: axe runs once the editor has left the screen.
      await expect(page.locator('lp-editor-page')).toBeHidden();
      expect(await blockingViolations(page)).toEqual([]);
    });
  });

  test.describe('at 390 px', () => {
    test.use({ viewport: PHONE });

    test('the welcome', async ({ page }) => {
      await new EditorPage(page).goto();
      expect(await blockingViolations(page)).toEqual([]);
    });

    test('the editor, with each inspector panel', async ({ page }) => {
      const editor = new EditorPage(page);
      await editor.open();
      await editor.create();
      const tabs = page.getByRole('tablist', { name: 'Panels' });
      for (const name of INSPECTOR_TABS) {
        await test.step(`the ${name} panel`, async () => {
          await tabs.getByRole('tab', { name, exact: true }).click();
          await expect(page.getByRole('tabpanel', { name, exact: true })).toBeVisible();
          expect(await blockingViolations(page)).toEqual([]);
        });
      }
    });

    test('the export dialog', async ({ page }) => {
      const editor = new EditorPage(page);
      await editor.open();
      await editor.create();
      await editor.button('Export').click();
      await expect(editor.exportDialog).toBeVisible();
      await editor.expectEveryFormatSized();
      expect(await blockingViolations(page)).toEqual([]);
    });
  });
});
