import { expect, test } from '@playwright/test';

import type { LifePixelElement } from '../life-pixel.js';
import { BLUE, GREEN, MASCOT, RED, WHITE } from './fixtures/animations';
import { fakeExport } from './fixtures/fake-export';
import { PlayerPage } from './fixtures/player-page';

const FRAME_MS = 100;
const POLL_MS = 20;

async function openMascot(page: PlayerPage['page'], attributes: string): Promise<PlayerPage> {
  const player = new PlayerPage(page);
  await player.open(`<life-pixel id="mascot" src="/mascot.wasm" ${attributes}></life-pixel>`, {
    '/mascot.wasm': { body: await fakeExport(MASCOT) },
  });
  await expect.poll(() => player.events()).toContain('mascot:load');
  return player;
}

/** Advances the clock a frame at a time until `mascot` fired `count` tagend events. */
async function playUntilTagends(player: PlayerPage, count: number): Promise<void> {
  await expect
    .poll(
      async () => {
        await player.advance(FRAME_MS);
        return tagendCount(player);
      },
      { intervals: [POLL_MS] },
    )
    .toBeGreaterThanOrEqual(count);
}

async function tagendCount(player: PlayerPage): Promise<number> {
  return (await player.events()).filter((event) => event === 'mascot:tagend').length;
}

function isPlaying(page: PlayerPage['page']): Promise<boolean> {
  return page.locator('life-pixel').evaluate((element: LifePixelElement) => element.playing);
}

test('plays the tag the attribute names', async ({ page }) => {
  const player = await openMascot(page, 'tag="jump"');

  expect(await player.shownColor('mascot')).toEqual(BLUE);
});

test('plays the tag set through the property, reflected on the attribute', async ({ page }) => {
  const player = await openMascot(page, '');
  expect(await player.shownColor('mascot')).toEqual(RED);

  await page.locator('life-pixel').evaluate((element: LifePixelElement) => (element.tag = 'jump'));

  expect(await page.locator('life-pixel').getAttribute('tag')).toBe('jump');
  expect(await player.shownColor('mascot')).toEqual(BLUE);
});

test('plays the first tag and warns for an unknown tag', async ({ page }) => {
  const player = await openMascot(page, 'tag="fly"');

  expect(await player.shownColor('mascot')).toEqual(RED);
  expect(player.warnings).toEqual([expect.stringContaining('fly')]);
});

test('fires tagend once, then stops, after a range played once', async ({ page }) => {
  const player = await openMascot(page, 'tag="jump"');

  await playUntilTagends(player, 1);
  await player.advance(10 * FRAME_MS);

  expect(await tagendCount(player)).toBe(1);
  expect(await isPlaying(page)).toBe(false);
  expect(await player.shownColor('mascot')).toEqual(WHITE);
});

test('fires tagend on every loop of a looping range', async ({ page }) => {
  const player = await openMascot(page, 'tag="idle"');

  await playUntilTagends(player, 3);

  expect(await isPlaying(page)).toBe(true);
});

test('plays the next tag when tagend sets it', async ({ page }) => {
  const player = await openMascot(page, 'tag="jump"');
  await page
    .locator('life-pixel')
    .evaluate((element: LifePixelElement) =>
      element.addEventListener('tagend', () => (element.tag = 'idle'), { once: true }),
    );

  await playUntilTagends(player, 1);

  expect(await isPlaying(page)).toBe(true);
  expect(await page.locator('life-pixel').getAttribute('tag')).toBe('idle');
  await playUntilTagends(player, 2);
});

test('loop makes a range played once loop', async ({ page }) => {
  const player = await openMascot(page, 'tag="jump" loop');

  await playUntilTagends(player, 2);

  expect(await isPlaying(page)).toBe(true);
});

test('loop="false" plays a looping range once', async ({ page }) => {
  const player = await openMascot(page, 'tag="idle" loop="false"');

  await playUntilTagends(player, 1);
  await player.advance(10 * FRAME_MS);

  expect(await tagendCount(player)).toBe(1);
  expect(await isPlaying(page)).toBe(false);
  expect(await player.shownColor('mascot')).toEqual(GREEN);
});
