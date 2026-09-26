import { expect, type Page } from '@playwright/test';

/**
 * Waits for the end of the app's page transition. `ion-router-outlet` keeps the page it leaves
 * in the DOM, shown until the transition ends and hidden after: until then, a control of both
 * pages — the same form, or a second editor — matches twice.
 */
export async function expectSettled(page: Page): Promise<void> {
  await expect(page.locator('ion-router-outlet > .ion-page:not(.ion-page-hidden)')).toHaveCount(1);
}
