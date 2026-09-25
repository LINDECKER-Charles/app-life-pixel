import { expect, type Page, type Route } from '@playwright/test';
import { BLINK_TAG, DURATIONS_MS, EXPECTED_FRAMES, type Pixel } from './drawing';
import type { WasmExport } from './editor-page';

/** An origin nothing resolves: `page.route` answers every request of the player's page. */
const ORIGIN = 'http://life-pixel.test';
const ELEMENT_ID = 'animation';
const EVENTS = ['load', 'error', 'tagend'];
/** Well short of the blink frame's duration, then well past it. */
const BEFORE_BLINK_ENDS_MS = DURATIONS_MS[1] - 100;
const AFTER_BLINK_ENDS_MS = 200;

declare global {
  interface Window {
    /** The `<life-pixel>` element's events, in order. */
    playerEvents: string[];
  }
}

/**
 * A page of someone's site playing the editor's WASM export with its loader, both downloaded
 * from the export dialog and served through `page.route`. Its clock is paused: only `advance`
 * moves playback, so frame durations are checked exactly.
 */
export class PlayerPage {
  constructor(readonly page: Page) {}

  async open(files: WasmExport): Promise<void> {
    const html =
      '<!doctype html><html lang="en"><head><title>Player</title>' +
      '<script type="module" src="/life-pixel.js"></script></head><body>' +
      `<life-pixel id="${ELEMENT_ID}" src="/${files.animation.name}"></life-pixel></body></html>`;
    await this.page.clock.install({ time: 0 });
    await this.page.clock.pauseAt(1000);
    await this.page.addInitScript(recordEvents, EVENTS);
    await this.page.route(`${ORIGIN}/**`, (route) => serve(route, html, files));
    await this.page.goto(`${ORIGIN}/`);
    await expect.poll(() => this.events()).toContain('load');
    await this.whenVisible();
  }

  events(): Promise<string[]> {
    return this.page.evaluate(() => window.playerEvents);
  }

  async setTag(tag: string): Promise<void> {
    await this.page.evaluate(
      ({ id, name }) => document.getElementById(id)?.setAttribute('tag', name),
      { id: ELEMENT_ID, name: tag },
    );
  }

  advance(ms: number): Promise<void> {
    return this.page.clock.runFor(ms);
  }

  /** The RGBA the player's canvas shows at a pixel, read back with `getImageData`. */
  shown(pixel: Pixel): Promise<number[]> {
    return this.page.evaluate(
      ({ id, x, y }) => {
        const canvas = document.getElementById(id)?.shadowRoot?.querySelector('canvas');
        const data = canvas?.getContext('2d')?.getImageData(x, y, 1, 1).data;
        return data ? [...data] : [];
      },
      { id: ELEMENT_ID, x: pixel.x, y: pixel.y },
    );
  }

  async expectFrame(index: number): Promise<void> {
    for (const { pixel, rgba } of EXPECTED_FRAMES[index]) {
      await expect
        .poll(() => this.shown(pixel), {
          message: `frame ${index + 1}, pixel ${pixel.x},${pixel.y}`,
        })
        .toEqual([...rgba]);
    }
  }

  /** Resolves once the element is on screen, when it starts requesting animation frames. */
  private whenVisible(): Promise<void> {
    return this.page.evaluate(
      (id) =>
        new Promise<void>((resolve) => {
          const element = document.getElementById(id);
          if (!element) throw new Error('No <life-pixel> on the page.');
          const observer = new IntersectionObserver((entries) => {
            if (!entries.some((entry) => entry.isIntersecting)) return;
            observer.disconnect();
            resolve();
          });
          observer.observe(element);
        }),
      ELEMENT_ID,
    );
  }
}

/**
 * Plays the export: the default tag, `idle`, shows frame 1; `blink` shows frame 2, then ends
 * once — `tagend` — after its own duration, staying on frame 2.
 */
export async function expectPlayback(player: PlayerPage): Promise<void> {
  await player.expectFrame(0);
  await player.setTag(BLINK_TAG);
  await player.expectFrame(1);
  const before = (await player.events()).length;
  await player.advance(BEFORE_BLINK_ENDS_MS);
  expect((await player.events()).slice(before)).not.toContain('tagend');
  await player.advance(AFTER_BLINK_ENDS_MS);
  expect((await player.events()).slice(before)).toEqual(['tagend']);
  await player.expectFrame(1);
  expect(await player.events()).not.toContain('error');
}

function serve(route: Route, html: string, files: WasmExport): Promise<void> {
  const path = new URL(route.request().url()).pathname;
  if (path === '/') return route.fulfill({ contentType: 'text/html', body: html });
  if (path === `/${files.loader.name}`) {
    return route.fulfill({ contentType: 'text/javascript', body: files.loader.bytes });
  }
  if (path === `/${files.animation.name}`) {
    return route.fulfill({ contentType: 'application/wasm', body: files.animation.bytes });
  }
  return route.fulfill({ status: 404 });
}

function recordEvents(types: string[]): void {
  window.playerEvents = [];
  for (const type of types) {
    // The element's events do not bubble: capturing on the document sees them.
    document.addEventListener(
      type,
      (event) => {
        const target = event.target;
        if (target instanceof Element && target.localName === 'life-pixel') {
          window.playerEvents.push(type);
        }
      },
      true,
    );
  }
}
