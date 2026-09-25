import { expect, test } from '@playwright/test';

import type { LifePixelElement } from '../life-pixel.js';
import { BLINK, MASCOT, RED, WHITE } from './fixtures/animations';
import { fakeExport } from './fixtures/fake-export';
import { PlayerPage } from './fixtures/player-page';

test('draws the first frame at the native size and fires load', async ({ page }) => {
  const player = new PlayerPage(page);
  await player.open('<life-pixel id="mascot" src="/mascot.wasm"></life-pixel>', {
    '/mascot.wasm': { body: await fakeExport(MASCOT) },
  });

  await expect.poll(() => player.events()).toEqual(['mascot:load']);
  const frame = await page.evaluate(() => {
    const canvas = document.querySelector('life-pixel')?.shadowRoot?.querySelector('canvas');
    const image = canvas?.getContext('2d')?.getImageData(0, 0, canvas.width, canvas.height);
    return image && { width: image.width, height: image.height, pixels: [...image.data] };
  });
  expect(frame?.width).toBe(8);
  expect(frame?.height).toBe(6);
  expect(frame?.pixels).toEqual(Array.from({ length: 8 * 6 }, () => RED).flat());
});

test('falls back to arrayBuffer when the server does not send application/wasm', async ({
  page,
}) => {
  const player = new PlayerPage(page);
  await player.open('<life-pixel id="blink" src="/blink.wasm"></life-pixel>', {
    '/blink.wasm': { body: await fakeExport(BLINK), contentType: 'application/octet-stream' },
  });

  await expect.poll(() => player.events()).toEqual(['blink:load']);
  expect(await player.shownColor('blink')).toEqual(WHITE);
});

test('fetches and compiles an export once for two elements', async ({ page }) => {
  const player = new PlayerPage(page);
  await player.open(
    '<life-pixel id="first" src="/blink.wasm"></life-pixel>' +
      '<life-pixel id="second" src="blink.wasm"></life-pixel>',
    { '/blink.wasm': { body: await fakeExport(BLINK) } },
  );

  await expect
    .poll(async () => (await player.events()).sort())
    .toEqual(['first:load', 'second:load']);
  expect(player.fetches.get('/blink.wasm')).toBe(1);
  expect(await player.shownColor('second')).toEqual(WHITE);
});

test('fires error and draws nothing for an unknown ABI', async ({ page }) => {
  const player = new PlayerPage(page);
  await player.open('<life-pixel id="future" src="/future.wasm"></life-pixel>', {
    '/future.wasm': { body: await fakeExport(BLINK, 2) },
  });

  await expect.poll(() => player.events()).toEqual(['future:error']);
  expect(await player.shownColor('future')).toBeNull();
  expect(player.warnings).toEqual([expect.stringContaining('ABI version 2')]);
});

test('fires error and names the status when the player refuses the payload', async ({ page }) => {
  const player = new PlayerPage(page);
  await player.open('<life-pixel id="broken" src="/broken.wasm"></life-pixel>', {
    '/broken.wasm': { body: await fakeExport({ ...BLINK, magic: 'GIF8' }) },
  });

  await expect.poll(() => player.events()).toEqual(['broken:error']);
  expect(await player.shownColor('broken')).toBeNull();
  expect(player.warnings).toEqual([expect.stringContaining('status 1')]);
});

test('loads the new export when src changes', async ({ page }) => {
  const player = new PlayerPage(page);
  await player.open('<life-pixel id="swap" src="/mascot.wasm"></life-pixel>', {
    '/mascot.wasm': { body: await fakeExport(MASCOT) },
    '/blink.wasm': { body: await fakeExport(BLINK) },
  });
  await expect.poll(() => player.events()).toEqual(['swap:load']);

  await page
    .locator('life-pixel')
    .evaluate((element: LifePixelElement) => (element.src = '/blink.wasm'));

  await expect.poll(() => player.events()).toEqual(['swap:load', 'swap:load']);
  expect(await page.locator('life-pixel').getAttribute('src')).toBe('/blink.wasm');
  expect(await player.shownColor('swap')).toEqual(WHITE);
});
