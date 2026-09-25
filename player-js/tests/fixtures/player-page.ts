import { readFile } from 'node:fs/promises';
import type { Page, Route } from '@playwright/test';

/** An origin the tests never resolve: `page.route` answers every request. */
export const ORIGIN = 'http://life-pixel.test';
const LOADER = new URL('../../life-pixel.js', import.meta.url);
const WASM_TYPE = 'application/wasm';
const ELEMENT_EVENTS = ['load', 'error', 'tagend'];
const NOT_FOUND = 404;

/** A file of the page's origin: an export, or anything a test needs. */
export interface ServedFile {
  readonly body: Uint8Array;
  readonly contentType?: string;
}

declare global {
  interface Window {
    /** The events of every `<life-pixel>`, as `<id>:<type>`, in order. */
    lifePixelEvents: string[];
  }
}

/**
 * A page that loads the committed build, `life-pixel.js`, with the files given, and records the
 * elements' events and warnings. Its clock is paused: only `advance` moves playback.
 */
export class PlayerPage {
  /** How many times each path was fetched. */
  readonly fetches = new Map<string, number>();
  /** The `console.warn` messages of the page. */
  readonly warnings: string[] = [];

  constructor(readonly page: Page) {
    page.on('console', (message) => {
      if (message.type() === 'warning') this.warnings.push(message.text());
    });
  }

  /** Opens a page whose body is `body`, serving `files` by path. */
  async open(body: string, files: Record<string, ServedFile>): Promise<void> {
    const loader = await readFile(LOADER);
    await this.page.clock.install({ time: 0 });
    await this.page.clock.pauseAt(1000);
    await this.page.addInitScript(recordEvents, ELEMENT_EVENTS);
    await this.page.route(`${ORIGIN}/**`, (route) => this.#serve(route, { body, loader, files }));
    await this.page.goto(`${ORIGIN}/`);
  }

  /** Moves the page's clock, running the animation frames that fall within. */
  advance(ms: number): Promise<void> {
    return this.page.clock.runFor(ms);
  }

  events(): Promise<string[]> {
    return this.page.evaluate(() => window.lifePixelEvents);
  }

  /** The RGBA of the top-left pixel an element shows, or `null` when it shows nothing. */
  shownColor(id: string): Promise<number[] | null> {
    return this.page.evaluate((elementId) => {
      const canvas = document.getElementById(elementId)?.shadowRoot?.querySelector('canvas');
      if (!canvas?.width || !canvas.height) return null;
      return [...(canvas.getContext('2d')?.getImageData(0, 0, 1, 1).data ?? [])];
    }, id);
  }

  #serve(route: Route, site: { body: string; loader: Buffer; files: Record<string, ServedFile> }) {
    const path = new URL(route.request().url()).pathname;
    this.fetches.set(path, (this.fetches.get(path) ?? 0) + 1);
    if (path === '/') return route.fulfill({ contentType: 'text/html', body: page(site.body) });
    if (path === '/life-pixel.js') {
      return route.fulfill({ contentType: 'text/javascript', body: site.loader });
    }
    const file = site.files[path];
    if (!file) return route.fulfill({ status: NOT_FOUND });
    const body = Buffer.from(file.body);
    return route.fulfill({ contentType: file.contentType ?? WASM_TYPE, body });
  }
}

function page(body: string): string {
  return `<!doctype html><html lang="en"><head><title>life-pixel</title>
<script type="module" src="/life-pixel.js"></script></head><body>${body}</body></html>`;
}

function recordEvents(types: string[]): void {
  window.lifePixelEvents = [];
  for (const type of types) {
    // Capturing on the document sees the elements' events, which do not bubble.
    document.addEventListener(
      type,
      (event) => {
        const target = event.target;
        if (target instanceof Element && target.localName === 'life-pixel') {
          window.lifePixelEvents.push(`${target.id}:${type}`);
        }
      },
      true,
    );
  }
}
