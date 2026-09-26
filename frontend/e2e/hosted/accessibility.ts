import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

/** WCAG 2.2 AA, the target of AGENTS.md, and every level below it. */
const WCAG_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const BLOCKING_IMPACTS = new Set(['serious', 'critical']);

/**
 * axe on `page` as it is now — a screen, or a dialog open over one —: no serious or critical
 * violation (support-admin.md, H17). A failure lists each violation with the elements it found.
 */
export async function expectAccessible(page: Page, screen: string): Promise<void> {
  await test.step(`axe: ${screen}`, async () => {
    // A page or a dialog fading in has the contrast of its opacity: axe waits for its end.
    await page.waitForFunction(() =>
      document
        .getAnimations()
        .every(
          (animation) =>
            animation.playState !== 'running' ||
            animation.effect?.getTiming().iterations === Infinity,
        ),
    );
    const results = await new AxeBuilder({ page }).withTags(WCAG_TAGS).analyze();
    const violations = results.violations
      .filter((violation) => BLOCKING_IMPACTS.has(violation.impact ?? ''))
      .map(
        (violation) =>
          `${violation.id} (${violation.impact}): ${violation.help} — ` +
          violation.nodes
            .map((node) => `${node.target.join(' ')} (${node.any[0]?.message ?? node.html})`)
            .join(', '),
      );
    expect.soft(violations, `serious or critical axe violations on ${screen}`).toEqual([]);
  });
}
