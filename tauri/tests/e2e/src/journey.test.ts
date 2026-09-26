import assert from 'node:assert/strict';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { setTimeout as sleep } from 'node:timers/promises';
import { Key, type WebElement } from 'selenium-webdriver';
import { cliAnimations } from './cli-animations.ts';
import { debugBuild } from './debug-build.ts';
import { DesktopSession } from './desktop-session.ts';
import { KeyboardCursor, type Pixel } from './keyboard-cursor.ts';
import { type Name, Screen } from './screen.ts';

/** The animation the journey draws, and the project it goes into. */
const TITLE = 'Desktop journey';
const PROJECT = 'Desktop tests';
const SIZE = 16;
/** Three black pixels with the pencil, in the first colour, then one more after the first save. */
const STROKE: readonly Pixel[] = [
  { x: 1, y: 1 },
  { x: 2, y: 1 },
  { x: 3, y: 1 },
];
const CHANGE: Pixel = { x: 8, y: 8 };

/** The first start builds the webview's caches: slower than any later wait. */
const APP_START_TIMEOUT_MS = 60_000;
const EXPORT_TIMEOUT_MS = 30_000;
/** How long a shortcut's dialog may take to show before the shortcut is pressed again. */
const SHORTCUT_ANSWER_MS = 2_000;
const SHORTCUT_TIMEOUT_MS = 20_000;
const EXPORT_POLL_INTERVAL_MS = 250;
/** How every GIF begins: `GIF87a` or `GIF89a`. */
const GIF_SIGNATURE = 'GIF8';
/** Where a failed run leaves a screenshot and the page's source, for CI to keep. */
const RESULTS_FOLDER = path.resolve(import.meta.dirname, '../test-results');

/** The journey's steps, on one app, each a person's action followed by what they see. */
class Journey {
  readonly #session: DesktopSession;
  readonly #screen: Screen;

  constructor(session: DesktopSession) {
    this.#session = session;
    this.#screen = new Screen(session.driver);
  }

  async startsOnANewAnimation(): Promise<void> {
    await this.#screen.find('textbox', 'Title', { timeoutMs: APP_START_TIMEOUT_MS });
  }

  async createsTheAnimation(): Promise<void> {
    await this.#screen.type('textbox', 'Title', TITLE);
    await this.#screen.type('spinbutton', 'Width', String(SIZE));
    await this.#screen.type('spinbutton', 'Height', String(SIZE));
    await this.#screen.waitUntilEnabled(await this.#screen.find('button', 'Create'));
    await this.#screen.act('button', 'Create', (button) => button.click());
    await this.#screen.waitUntilGone('textbox', 'Title');
    await this.#screen.find('button', 'Frame 1');
  }

  async drawsWithTheKeyboard(): Promise<void> {
    const cursor = new KeyboardCursor(await this.#pencilOnCanvas());
    for (const pixel of STROKE) await cursor.applyAt(pixel);
    await this.#screen.waitUntilEnabled(await this.#screen.find('button', 'Undo'));
  }

  async savesIntoANewProject(): Promise<void> {
    await this.#pressOnCanvas(Key.chord(Key.CONTROL, 's'));
    await this.#screen.type('textbox', 'Project name', PROJECT);
    await this.#screen.waitUntilEnabled(await this.#screen.find('button', 'Save here'));
    await this.#screen.act('button', 'Save here', (button) => button.click());
    await this.#screen.find('status', 'Saved');
  }

  /** A second save sends back the version the first one answered: a lost one would conflict. */
  async changesAndSavesAgain(): Promise<void> {
    await new KeyboardCursor(await this.#pencilOnCanvas()).applyAt(CHANGE);
    await this.#screen.waitUntilGone('status', 'Saved');
    await this.#pressOnCanvas(Key.chord(Key.CONTROL, 's'));
    await this.#screen.find('status', 'Saved');
    assert.deepEqual(await this.#screen.queryAll('button', 'Overwrite'), []);
  }

  async findsItInTheLibrary(): Promise<void> {
    await this.#screen.act('link', 'Library', (link) => link.click());
    await this.#screen.find('link', PROJECT);
    await this.#screen.act('link', TITLE, (link) => link.click());
    await this.#screen.find('application', 'Drawing canvas');
    await this.#screen.find('button', 'Frame 1');
  }

  /** Ctrl+E, as the Export button is an Ionic one, which WebKitGTK's WebDriver cannot click. */
  async exportsAGif(): Promise<void> {
    const shortcut = Key.chord(Key.CONTROL, 'e');
    const table = await this.#pressUntilShown(
      shortcut,
      'table',
      'Every export format, with its size',
    );
    const row = await this.#rowOf(table, /^GIF\b/);
    const download = await this.#screen.find('button', 'Download', { within: row });
    await this.#screen.waitUntilEnabled(download);
    await download.click();
    const gif = await waitForFile(this.#session.folders.exports, '.gif');
    const bytes = await readFile(gif);
    assert.equal(bytes.subarray(0, GIF_SIGNATURE.length).toString('latin1'), GIF_SIGNATURE);
  }

  async isListedByTheBundledCli(cli: string): Promise<void> {
    const animations = await cliAnimations(cli, this.#session.folders.library);
    const listed = animations.map(({ title, width, height, frameCount }) => ({
      title,
      width,
      height,
      frameCount,
    }));
    assert.deepEqual(listed, [{ title: TITLE, width: SIZE, height: SIZE, frameCount: 1 }]);
  }

  /** Keeps a screenshot and the page's source of the window as it is. */
  async keepEvidence(): Promise<void> {
    const driver = this.#session.driver;
    await mkdir(RESULTS_FOLDER, { recursive: true });
    const screenshot = await driver.takeScreenshot();
    await writeFile(path.join(RESULTS_FOLDER, 'failure.png'), screenshot, 'base64');
    await writeFile(path.join(RESULTS_FOLDER, 'failure.html'), await driver.getPageSource());
  }

  /** The canvas, focused, with the pencil — `b` — pressed. */
  async #pencilOnCanvas(): Promise<WebElement> {
    const canvas = await this.#pressOnCanvas('b');
    const pencil = await this.#screen.act('button', 'Pencil', () => Promise.resolve());
    await this.#screen.waitUntilPressed(pencil);
    return canvas;
  }

  /** Presses `keys` on the canvas of the editor shown. */
  #pressOnCanvas(keys: string): Promise<WebElement> {
    return this.#screen.act('application', 'Drawing canvas', (canvas) => canvas.sendKeys(keys));
  }

  /**
   * Presses `keys` on the canvas until `role` named `name` shows: the editor takes no shortcut
   * until Ionic's move to its page ends, so a person presses again.
   */
  async #pressUntilShown(keys: string, role: string, name: Name): Promise<WebElement> {
    const deadline = Date.now() + SHORTCUT_TIMEOUT_MS;
    for (;;) {
      await this.#pressOnCanvas(keys);
      const options = { timeoutMs: SHORTCUT_ANSWER_MS };
      const shown = await this.#screen.find(role, name, options).catch(() => null);
      if (shown !== null) return shown;
      if (Date.now() > deadline) throw new Error(`no ${role} named “${String(name)}” was shown`);
    }
  }

  /** The row of `table` whose header matches `header`. */
  async #rowOf(table: WebElement, header: RegExp): Promise<WebElement> {
    for (const row of await this.#screen.queryAll('row', undefined, table)) {
      const headers = await this.#screen.queryAll('rowheader', header, row);
      if (headers.length > 0) return row;
    }
    throw new Error(`no row of the table has a header matching ${String(header)}`);
  }
}

/** The first file of `folder` ending with `extension`, once there is one. */
async function waitForFile(folder: string, extension: string): Promise<string> {
  const deadline = Date.now() + EXPORT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const names = await readdir(folder).catch(() => []);
    const found = names.find((name) => name.endsWith(extension));
    if (found !== undefined) return path.join(folder, found);
    await sleep(EXPORT_POLL_INTERVAL_MS);
  }
  throw new Error(`no ${extension} file appeared in ${folder}`);
}

/** Runs one step, naming it in the report and in its failure. */
async function step(t: TestContext, name: string, run: () => Promise<void>): Promise<void> {
  t.diagnostic(name);
  try {
    await run();
  } catch (error) {
    throw new Error(`${name}: ${error instanceof Error ? error.message : String(error)}`, {
      cause: error,
    });
  }
}

// M5's journey (desktop.md, T5): the debug app on temporary folders, driven through tauri-driver.
void test('draws, saves, finds and exports an animation in the desktop app', async (t) => {
  const build = debugBuild();
  const session = await DesktopSession.start(build.application);
  const journey = new Journey(session);
  try {
    await step(t, 'the app starts on a new animation', () => journey.startsOnANewAnimation());
    await step(t, 'creates the animation', () => journey.createsTheAnimation());
    await step(t, 'draws with the keyboard', () => journey.drawsWithTheKeyboard());
    await step(t, 'saves it into a new project', () => journey.savesIntoANewProject());
    await step(t, 'changes it and saves again', () => journey.changesAndSavesAgain());
    await step(t, 'finds it in the library', () => journey.findsItInTheLibrary());
    await step(t, 'exports a GIF', () => journey.exportsAGif());
    await step(t, 'the bundled CLI lists it', () => journey.isListedByTheBundledCli(build.cli));
  } catch (error) {
    await journey.keepEvidence().catch(() => undefined);
    throw error;
  } finally {
    await session.stop();
  }
});
