import { readFile } from 'node:fs/promises';
import { expect, type Download, type Locator, type Page } from '@playwright/test';
import { SIZE, TITLE, type Pixel } from './drawing';

/** EditorStore's zoom when an animation opens: one animation pixel is 8 CSS pixels. */
const DEFAULT_ZOOM = 8;
/** How many Tab presses reach any control of the editor, dialogs and palette included. */
const MAX_TAB_PRESSES = 150;

/** The formats of the export dialog, by the name each row shows (editor.md, U5). */
export const EXPORT_FORMATS = ['WASM', 'GIF', 'APNG', 'Sprite sheet', 'PNG frames'] as const;

/** A file the editor downloaded: its suggested name and its bytes. */
export interface DownloadedFile {
  readonly name: string;
  readonly bytes: Buffer;
}

/** The WASM export's two files: the animation and the loader that plays it. */
export interface WasmExport {
  readonly animation: DownloadedFile;
  readonly loader: DownloadedFile;
}

/**
 * The editor as a person finds it: every control by its role and accessible name, the canvas
 * addressed in animation pixels.
 */
export class EditorPage {
  readonly canvas: Locator;
  readonly newDialog: Locator;
  readonly exportDialog: Locator;
  readonly shortcutsDialog: Locator;
  readonly tagDialog: Locator;

  constructor(readonly page: Page) {
    this.canvas = page.getByRole('application', { name: 'Drawing canvas' });
    this.newDialog = page.getByRole('dialog', { name: 'New animation' });
    this.exportDialog = page.getByRole('dialog', { name: 'Export the animation' });
    this.shortcutsDialog = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
    this.tagDialog = page.getByRole('dialog', { name: 'Add tag' });
  }

  /** Opens `/editor`, whose first visit asks for a new animation. */
  async open(): Promise<void> {
    await this.page.goto('/editor');
    await expect(this.newDialog).toBeVisible();
  }

  /** Creates the journeys' animation through the new-animation dialog, with the pointer. */
  async create(): Promise<void> {
    await this.textbox('Title').fill(TITLE);
    await this.spinbutton('Width').fill(String(SIZE));
    await this.spinbutton('Height').fill(String(SIZE));
    await this.button('Create').click();
    await this.expectReady();
  }

  /** The animation is open: the dialog closed, and its first frame listed in the timeline. */
  async expectReady(): Promise<void> {
    await expect(this.newDialog).toBeHidden();
    await expect(this.frameButton(1)).toBeVisible();
  }

  // --- Controls. Ionic sets `role="dialog"` on a wrapper inside `ion-modal`'s shadow root, which
  // its slotted content does not descend from: a dialog's controls are found on the page, where
  // they are the only ones, since an open modal hides everything behind it with `aria-hidden`. ---

  button(name: string): Locator {
    return this.page.getByRole('button', { name, exact: true });
  }

  textbox(name: string): Locator {
    return this.page.getByRole('textbox', { name, exact: true });
  }

  spinbutton(name: string): Locator {
    return this.page.getByRole('spinbutton', { name, exact: true });
  }

  radio(name: string): Locator {
    return this.page.getByRole('radio', { name, exact: true });
  }

  tool(name: string): Locator {
    return this.page
      .getByRole('region', { name: 'Tools' })
      .getByRole('button', { name, exact: true });
  }

  swatch(index: number): Locator {
    return this.page
      .getByRole('region', { name: 'Palette' })
      .getByRole('button', { name: `Colour ${index}`, exact: true });
  }

  frameButton(number: number): Locator {
    return this.page.getByRole('button', { name: `Frame ${number}`, exact: true });
  }

  duration(number: number): Locator {
    return this.spinbutton(`Duration of frame ${number}, in milliseconds`);
  }

  tagBar(name: string): Locator {
    return this.page.getByRole('button', { name: `Edit tag “${name}”` });
  }

  exportRow(format: string): Locator {
    return this.page
      .getByRole('table', { name: 'Every export format, with its size' })
      .getByRole('row')
      .filter({ has: this.page.getByRole('rowheader', { name: new RegExp(`^${format}\\b`) }) });
  }

  // --- The canvas, with the pointer. ---

  /** The centre of an animation pixel, in the page's coordinates, at the default zoom. */
  async pointAt(pixel: Pixel): Promise<{ x: number; y: number }> {
    const box = await this.canvas.boundingBox();
    if (!box) throw new Error('The drawing canvas is not laid out.');
    const side = SIZE * DEFAULT_ZOOM;
    return {
      x: box.x + (box.width - side) / 2 + (pixel.x + 0.5) * DEFAULT_ZOOM,
      y: box.y + (box.height - side) / 2 + (pixel.y + 0.5) * DEFAULT_ZOOM,
    };
  }

  /** Presses on the first pixel, moves through the others, and releases on the last. */
  async drag(pixels: readonly Pixel[]): Promise<void> {
    const [first, ...rest] = pixels;
    const start = await this.pointAt(first);
    await this.page.mouse.move(start.x, start.y);
    await this.page.mouse.down();
    for (const pixel of rest) {
      const point = await this.pointAt(pixel);
      await this.page.mouse.move(point.x, point.y, { steps: 4 });
    }
    await this.page.mouse.up();
  }

  async click(pixel: Pixel): Promise<void> {
    const point = await this.pointAt(pixel);
    await this.page.mouse.click(point.x, point.y);
  }

  // --- The export dialog. ---

  /** Waits until every format shows its raw and gzip sizes, then checks there are five. */
  async expectEveryFormatSized(): Promise<void> {
    const rows = this.page
      .getByRole('table', { name: 'Every export format, with its size' })
      .getByRole('row')
      .filter({ has: this.page.getByRole('cell') });
    await expect(rows).toHaveCount(EXPORT_FORMATS.length);
    for (const format of EXPORT_FORMATS) {
      const cells = this.exportRow(format).getByRole('cell');
      await expect(cells.nth(0)).toHaveText(/^\d[\d.,]*\s?[a-zA-Z]+$/);
      await expect(cells.nth(1)).toHaveText(/^\d[\d.,]*\s?[a-zA-Z]+$/);
    }
  }

  /** Runs `trigger`, then collects the WASM export's two downloads. */
  async downloadWasm(trigger: () => Promise<void>): Promise<WasmExport> {
    const downloads: Download[] = [];
    const collect = (download: Download): void => void downloads.push(download);
    this.page.on('download', collect);
    await trigger();
    await expect.poll(() => downloads.length).toBe(2);
    this.page.off('download', collect);
    const files = await Promise.all(downloads.map(readDownload));
    const animation = files.find((file) => file.name.endsWith('.wasm'));
    const loader = files.find((file) => file.name === 'life-pixel.js');
    if (!animation || !loader) throw new Error(`Unexpected downloads: ${files.map((f) => f.name)}`);
    return { animation, loader };
  }
}

async function readDownload(download: Download): Promise<DownloadedFile> {
  return { name: download.suggestedFilename(), bytes: await readFile(await download.path()) };
}

/**
 * Presses Tab — or Shift+Tab — until `target` holds the focus, as a keyboard user reaches a
 * control; fails when it cannot be reached.
 */
export async function tabTo(page: Page, target: Locator, backwards = false): Promise<void> {
  const key = backwards ? 'Shift+Tab' : 'Tab';
  for (let presses = 0; presses < MAX_TAB_PRESSES; presses++) {
    if (await isFocused(target)) return;
    await page.keyboard.press(key);
  }
  await expect(target).toBeFocused();
}

function isFocused(target: Locator): Promise<boolean> {
  return target.evaluate((element) => {
    // Ionic's buttons keep the focused native button in their shadow root.
    let active = document.activeElement;
    while (active?.shadowRoot?.activeElement) active = active.shadowRoot.activeElement;
    return active !== null && (element === active || element.contains(active));
  });
}
