import { readdirSync, readFileSync } from 'node:fs';

import { expect, test } from '@playwright/test';
import type { Locator } from '@playwright/test';

import type { LifePixelElement } from '../life-pixel.js';
import { PlayerPage } from './fixtures/player-page';

/**
 * The real exports `crates/compiler` writes and checks byte for byte, each beside
 * `<stem>.expected.json`: what it must show, from the rendering of `life-pixel-core`.
 */
const GOLDEN = new URL('../../crates/compiler/tests/golden/', import.meta.url);
const EXPORT_EXTENSION = '.wasm';
const EXPECTED_EXTENSION = '.expected.json';
const ID = 'golden';
/**
 * How far the clock jumps between two looks for `tagend`: frames last up to 65,535 ms, and the
 * loader passes at most 1,000 ms to one `tick`.
 */
const STEP_MS = 1000;
const POLL_MS = 20;
/** What the loader's own warnings start with, as opposed to the browser's advice. */
const LOADER_WARNING = '<life-pixel>';

interface ExpectedRange {
  readonly name: string;
  readonly first: number;
  readonly last: number;
  readonly loop: 'loop' | 'once';
}

interface ExpectedFrame {
  readonly durationMs: number;
  readonly rgba: string;
}

interface Expected {
  readonly title: string;
  readonly width: number;
  readonly height: number;
  readonly tags: readonly ExpectedRange[];
  readonly frames: readonly ExpectedFrame[];
}

interface Golden {
  readonly stem: string;
  readonly body: Uint8Array;
  readonly expected: Expected;
}

function golden(stem: string): Golden {
  const read = (extension: string) => readFileSync(new URL(`${stem}${extension}`, GOLDEN));
  const expected: Expected = JSON.parse(read(EXPECTED_EXTENSION).toString('utf8'));
  return { stem, body: read(EXPORT_EXTENSION), expected };
}

const GOLDENS = readdirSync(GOLDEN)
  .filter((name) => name.endsWith(EXPORT_EXTENSION))
  .sort()
  .map((name) => golden(name.slice(0, -EXPORT_EXTENSION.length)));

/** The ranges an element plays: each tag, or without tags the whole animation, which loops. */
function ranges({ expected }: Golden): readonly ExpectedRange[] {
  if (expected.tags.length) return expected.tags;
  return [{ name: '', first: 0, last: expected.frames.length - 1, loop: 'loop' }];
}

async function open(player: PlayerPage, { stem, body }: Golden, attributes: string) {
  const element = `<life-pixel id="${ID}" src="/${stem}.wasm" ${attributes}></life-pixel>`;
  await player.open(element, { [`/${stem}.wasm`]: { body } });
  await expect.poll(() => player.events()).toContain(`${ID}:load`);
  return player.page.locator('life-pixel');
}

/**
 * The element's canvas, and `frame` put on a canvas of the same size: both read back through
 * `getImageData`, so that the browser's rounding of partial alpha applies to both alike.
 */
function shownAndExpected(element: Locator, frame: ExpectedFrame) {
  return element.evaluate((host: LifePixelElement, hex: string) => {
    const canvas = host.shadowRoot?.querySelector('canvas');
    if (!canvas) return null;
    const { width, height } = canvas;
    const pixels = Uint8ClampedArray.from(hex.match(/../g) ?? [], (byte) => parseInt(byte, 16));
    const scratch = document.createElement('canvas');
    Object.assign(scratch, { width, height });
    scratch.getContext('2d')?.putImageData(new ImageData(pixels, width, height), 0, 0);
    const read = (source: HTMLCanvasElement) =>
      [...(source.getContext('2d')?.getImageData(0, 0, width, height).data ?? [])].join();
    return { size: [width, height], shown: read(canvas), expected: read(scratch) };
  }, frame.rgba);
}

async function expectFrame(element: Locator, expected: Expected, frame: number): Promise<void> {
  const canvas = await shownAndExpected(element, expected.frames[frame]);
  expect(canvas?.size, `frame ${frame}`).toEqual([expected.width, expected.height]);
  expect(canvas?.shown, `frame ${frame}`).toBe(canvas?.expected);
}

async function tagendCount(player: PlayerPage): Promise<number> {
  return (await player.events()).filter((event) => event === `${ID}:tagend`).length;
}

for (const exported of GOLDENS) {
  const { stem, expected } = exported;

  test(`${stem}: shows every frame of every range as core renders it`, async ({ page }) => {
    const player = new PlayerPage(page);
    const element = await open(player, exported, 'autoplay="false"');

    await expect(page.getByRole('img')).toHaveAccessibleName(expected.title);
    for (const range of ranges(exported)) {
      if (range.name) {
        await element.evaluate((host: LifePixelElement, tag) => (host.tag = tag), range.name);
      }
      for (let frame = range.first; frame <= range.last; frame++) {
        const offset = frame - range.first;
        await element.evaluate((host: LifePixelElement, to) => host.seek(to), offset);
        await expectFrame(element, expected, frame);
      }
    }
    expect(player.warnings.filter((warning) => warning.startsWith(LOADER_WARNING))).toEqual([]);
  });

  for (const range of ranges(exported)) {
    const name = range.name || 'the whole animation';
    const end = range.loop === 'loop' ? 'loops' : 'stops on its last frame';

    test(`${stem}: plays ${name} to its end, then ${end}`, async ({ page }) => {
      const player = new PlayerPage(page);
      const element = await open(player, exported, range.name ? `tag="${range.name}"` : '');
      await expectFrame(element, expected, range.first);

      // One animation frame per jump, which ticks the whole step.
      const advance = async () => {
        await page.clock.fastForward(STEP_MS);
        return tagendCount(player);
      };
      await expect.poll(advance, { intervals: [POLL_MS] }).toBeGreaterThanOrEqual(1);

      const isPlaying = await element.evaluate((host: LifePixelElement) => host.playing);
      expect(isPlaying).toBe(range.loop === 'loop');
      if (range.loop === 'once') await expectFrame(element, expected, range.last);
    });
  }
}
