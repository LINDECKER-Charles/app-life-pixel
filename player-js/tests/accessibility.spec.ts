import { expect, test } from '@playwright/test';

import { MASCOT } from './fixtures/animations';
import { fakeExport } from './fixtures/fake-export';
import { PlayerPage } from './fixtures/player-page';

async function openMascot(page: PlayerPage['page'], attributes: string): Promise<PlayerPage> {
  const player = new PlayerPage(page);
  await player.open(`<life-pixel id="mascot" src="/mascot.wasm" ${attributes}></life-pixel>`, {
    '/mascot.wasm': { body: await fakeExport(MASCOT) },
  });
  await expect.poll(() => player.events()).toContain('mascot:load');
  return player;
}

test('is an image named by alt', async ({ page }) => {
  await openMascot(page, 'alt="The mascot waving"');

  await expect(page.getByRole('img')).toHaveAccessibleName('The mascot waving');
});

test('is an image named by the animation title without alt', async ({ page }) => {
  await openMascot(page, '');

  await expect(page.getByRole('img')).toHaveAccessibleName('The mascot');
});

test('alt="" hides a decorative animation from assistive technologies', async ({ page }) => {
  await openMascot(page, 'alt=""');

  await expect(page.locator('life-pixel')).toHaveAttribute('aria-hidden', 'true');
  await expect(page.getByRole('img')).toHaveCount(0);
});

test('takes its role when connected, not when created', async ({ page }) => {
  await openMascot(page, '');

  const roles = await page.evaluate(() => {
    const element = document.createElement('life-pixel');
    const created = element.getAttribute('role');
    document.body.append(element);
    return { created, connected: element.getAttribute('role') };
  });

  expect(roles).toEqual({ created: null, connected: 'img' });
});
