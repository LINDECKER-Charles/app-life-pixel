import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { EditorPage } from './editor-page';

/** WCAG 2.2 AA, the target of AGENTS.md, and every level below it. */
const WCAG_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const BLOCKING_IMPACTS = new Set(['serious', 'critical']);

/** The serious and critical violations axe finds on the page as it is now, one line each. */
async function blockingViolations(page: Page): Promise<string[]> {
  const results = await new AxeBuilder({ page }).withTags(WCAG_TAGS).analyze();
  return results.violations
    .filter((violation) => BLOCKING_IMPACTS.has(violation.impact ?? ''))
    .map(
      (violation) =>
        `${violation.id} (${violation.impact}): ${violation.help} — ` +
        violation.nodes.map((node) => node.target.join(' ')).join(', '),
    );
}

// axe on every screen of the editor path (editor.md, U6): no serious or critical violation.
test.describe('accessibility', () => {
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
