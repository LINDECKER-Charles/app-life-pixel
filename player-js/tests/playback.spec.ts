import { expect, test, type Page } from '@playwright/test';

import type { LifePixelElement } from '../life-pixel.js';
import { BLINK, BLUE, MASCOT, WHITE } from './fixtures/animations';
import { fakeExport } from './fixtures/fake-export';
import { PlayerPage } from './fixtures/player-page';

const FRAME_MS = 100;
const POLL_MS = 20;
const LONG_MS = 1000;

async function openBlink(page: Page, body: string): Promise<PlayerPage> {
  const player = new PlayerPage(page);
  await player.open(body, { '/blink.wasm': { body: await fakeExport(BLINK) } });
  return player;
}

/** Advances the clock until element `id` shows `color`. */
async function playUntilShown(player: PlayerPage, id: string, color: readonly number[]) {
  await expect
    .poll(
      async () => {
        await player.advance(FRAME_MS);
        return player.shownColor(id);
      },
      { intervals: [POLL_MS] },
    )
    .toEqual(color);
}

/** Checks that element `id` keeps the frame it shows for a long while. */
async function expectStill(player: PlayerPage, id: string): Promise<void> {
  const shown = await player.shownColor(id);
  for (let elapsed = 0; elapsed < LONG_MS; elapsed += FRAME_MS) {
    await player.advance(FRAME_MS);
    expect(await player.shownColor(id)).toEqual(shown);
  }
}

/** Resolves once `id` intersects the viewport, or stops to: the element's observer saw it too. */
function waitForIntersection(page: Page, id: string, isIntersecting: boolean): Promise<void> {
  return page.evaluate(
    ([elementId, expected]) =>
      new Promise<void>((resolve) => {
        const observer = new IntersectionObserver((entries) => {
          if (entries.some((entry) => entry.isIntersecting === expected)) {
            observer.disconnect();
            resolve();
          }
        });
        const element = document.getElementById(elementId as string);
        if (element) observer.observe(element);
      }),
    [id, isIntersecting] as const,
  );
}

function setHidden(page: Page, isHidden: boolean): Promise<void> {
  return page.evaluate((hidden) => {
    Object.defineProperty(document, 'hidden', { configurable: true, get: () => hidden });
    document.dispatchEvent(new Event('visibilitychange'));
  }, isHidden);
}

function isPlaying(page: Page, id: string): Promise<boolean> {
  return page.locator(`#${id}`).evaluate((element: LifePixelElement) => element.playing);
}

test('pause() keeps the frame shown, play() resumes and seek() shows a frame', async ({ page }) => {
  const player = await openBlink(page, '<life-pixel id="blink" src="/blink.wasm"></life-pixel>');
  const blink = page.locator('#blink');
  await playUntilShown(player, 'blink', BLUE);

  await blink.evaluate((element: LifePixelElement) => element.pause());
  expect(await isPlaying(page, 'blink')).toBe(false);
  await expectStill(player, 'blink');

  await blink.evaluate((element: LifePixelElement) => element.seek(0));
  expect(await player.shownColor('blink')).toEqual(WHITE);

  await blink.evaluate((element: LifePixelElement) => element.play());
  expect(await isPlaying(page, 'blink')).toBe(true);
  await playUntilShown(player, 'blink', BLUE);
});

test('pauses while off-screen', async ({ page }) => {
  const player = await openBlink(
    page,
    '<life-pixel id="near" src="/blink.wasm"></life-pixel>' +
      '<div style="height: 300vh"></div>' +
      '<life-pixel id="far" src="/blink.wasm"></life-pixel>',
  );
  await expect.poll(async () => (await player.events()).sort()).toEqual(['far:load', 'near:load']);
  await waitForIntersection(page, 'far', false);
  await playUntilShown(player, 'near', BLUE);
  expect(await player.shownColor('far')).toEqual(WHITE);

  await page.locator('#far').scrollIntoViewIfNeeded();
  await waitForIntersection(page, 'far', true);
  await playUntilShown(player, 'far', BLUE);

  await page.evaluate(() => window.scrollTo(0, 0));
  await waitForIntersection(page, 'far', false);
  await expectStill(player, 'far');
});

test('pauses while the tab is hidden', async ({ page }) => {
  const player = await openBlink(page, '<life-pixel id="blink" src="/blink.wasm"></life-pixel>');
  await playUntilShown(player, 'blink', BLUE);

  await setHidden(page, true);
  await expectStill(player, 'blink');

  await setHidden(page, false);
  await playUntilShown(player, 'blink', WHITE);
  await playUntilShown(player, 'blink', BLUE);
});

test('autoplay="false" waits for play()', async ({ page }) => {
  const player = await openBlink(
    page,
    '<life-pixel id="blink" src="/blink.wasm" autoplay="false"></life-pixel>',
  );
  await expect.poll(() => player.events()).toContain('blink:load');

  await expectStill(player, 'blink');
  expect(await isPlaying(page, 'blink')).toBe(false);

  await page.locator('#blink').evaluate((element: LifePixelElement) => element.play());
  await playUntilShown(player, 'blink', BLUE);
});

test.describe('with reduced motion preferred', () => {
  test.beforeEach(({ page }) => page.emulateMedia({ reducedMotion: 'reduce' }));

  test('shows the first frame of the range without playing; play() plays', async ({ page }) => {
    const player = new PlayerPage(page);
    await player.open('<life-pixel id="mascot" src="/mascot.wasm" tag="jump"></life-pixel>', {
      '/mascot.wasm': { body: await fakeExport(MASCOT) },
    });
    await expect.poll(() => player.events()).toContain('mascot:load');

    await expectStill(player, 'mascot');
    expect(await player.shownColor('mascot')).toEqual(BLUE);
    expect(await isPlaying(page, 'mascot')).toBe(false);

    await page.locator('#mascot').evaluate((element: LifePixelElement) => element.play());
    await playUntilShown(player, 'mascot', WHITE);
  });

  test('motion="always" plays', async ({ page }) => {
    const player = await openBlink(
      page,
      '<life-pixel id="blink" src="/blink.wasm" motion="always"></life-pixel>',
    );

    await playUntilShown(player, 'blink', BLUE);
    expect(await isPlaying(page, 'blink')).toBe(true);
  });
});
