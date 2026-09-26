import { expect, type Locator, type Page } from '@playwright/test';
import { label } from './labels';
import type { Language } from './visual-matrix';

/** The captured animation's side: at the default zoom of 8 it takes 256 CSS pixels. */
export const SIZE = 32;
/** EditorStore's zoom when an animation opens: one animation pixel is 8 CSS pixels. */
export const DEFAULT_ZOOM = 8;

/**
 * The action of the welcome that replaces the dialog opened on the first visit (design-system,
 * journeys.md §1): matched by its text in both languages, so the journey works before and after.
 */
const CREATE_ANIMATION = /^(create (an )?animation|créer une animation)$/i;

/** Palette indices of core's default palette: black, red and green. */
const BLACK = 1;
const RED = 6;
const GREEN = 9;

interface Pixel {
  readonly x: number;
  readonly y: number;
}

/** Frame 1: a black diagonal and a red bar; frame 2: one green dot. */
const DIAGONAL: readonly Pixel[] = [2, 4, 6, 8, 10, 12].map((step) => ({ x: step, y: step }));
const BAR: readonly Pixel[] = [
  { x: 4, y: 24 },
  { x: 16, y: 24 },
  { x: 27, y: 24 },
];
const DOT: Pixel = { x: 16, y: 16 };

/**
 * The editor as the captures walk through it, in any language: every control is found by its role
 * and by its text in that language's catalogue.
 */
export class VisualEditor {
  readonly stage: Locator;
  readonly canvas: Locator;
  readonly newDialog: Locator;
  readonly exportDialog: Locator;

  constructor(
    readonly page: Page,
    private readonly language: Language,
  ) {
    this.stage = page.getByRole('region', { name: this.text('editor.region.canvas'), exact: true });
    this.canvas = page.getByRole('application', { name: this.text('canvas.workspace.label') });
    this.newDialog = page.getByRole('dialog', { name: this.text('editor.new.title') });
    this.exportDialog = page.getByRole('dialog', { name: this.text('export.dialog.title') });
  }

  /**
   * Opens `/editor` and waits for its first screen: today the new-animation dialog opens on its
   * own; a welcome in the empty workspace may offer "Create animation" instead.
   */
  async open(): Promise<'dialog' | 'welcome'> {
    await this.page.goto('/editor');
    const welcome = this.page.getByRole('button', { name: CREATE_ANIMATION });
    await expect(this.newDialog.or(welcome).first()).toBeVisible();
    return (await this.newDialog.isVisible()) ? 'dialog' : 'welcome';
  }

  /** Opens the new-animation dialog from the welcome. */
  async startFromWelcome(): Promise<void> {
    await this.page.getByRole('button', { name: CREATE_ANIMATION }).first().click();
    await expect(this.newDialog).toBeVisible();
  }

  /** Creates a SIZE × SIZE animation through the open new-animation dialog. */
  async create(): Promise<void> {
    await this.spinbutton('editor.new.width').fill(String(SIZE));
    await this.spinbutton('editor.new.height').fill(String(SIZE));
    await this.button('editor.new.create').click();
    await expect(this.newDialog).toBeHidden();
    await expect(this.frame(1)).toBeVisible();
  }

  /** Draws frame 1 with the pencil, adds frame 2 with a dot, and comes back to frame 1. */
  async drawTwoFrames(): Promise<void> {
    await this.tool('tools.tool.pencil').click();
    await this.swatch(BLACK).click();
    await this.drag(DIAGONAL);
    await this.swatch(RED).click();
    await this.drag(BAR);
    await expect(this.button('tools.undo')).toBeEnabled();
    await this.button('timeline.frames.add').click();
    await this.frame(2).click();
    await this.swatch(GREEN).click();
    await this.drag([DOT]);
    await this.frame(1).click();
  }

  /** Opens the export dialog and waits, without failing, until each format shows its size. */
  async openExport(): Promise<void> {
    await this.button('export.action').click();
    await expect(this.exportDialog).toBeVisible();
    const table = this.page.getByRole('table', { name: this.text('export.table.caption') });
    await expect(table).toBeVisible();
    const sized = table.getByRole('cell').filter({ hasText: /^\d[\d.,]*\s?\p{L}+$/u });
    await expect(sized.first())
      .toBeVisible({ timeout: 20_000 })
      .catch(() => undefined);
  }

  private text(key: string, values?: Readonly<Record<string, number>>): string {
    return label(this.language, key, values);
  }

  private button(key: string): Locator {
    return this.page.getByRole('button', { name: this.text(key), exact: true });
  }

  private spinbutton(key: string): Locator {
    return this.page.getByRole('spinbutton', { name: this.text(key), exact: true });
  }

  private tool(key: string): Locator {
    return this.page
      .getByRole('region', { name: this.text('editor.region.tools'), exact: true })
      .getByRole('button', { name: this.text(key), exact: true });
  }

  private swatch(index: number): Locator {
    const name = this.text('palette.swatch', { index });
    return this.page.getByRole('button', { name, exact: true });
  }

  private frame(number: number): Locator {
    const name = this.text('timeline.frames.thumbnail_label', { number });
    return this.page.getByRole('button', { name, exact: true });
  }

  /** The centre of an animation pixel, on a canvas centred in its surface at the default zoom. */
  private async pointAt(pixel: Pixel): Promise<{ x: number; y: number }> {
    const box = await this.canvas.boundingBox();
    if (!box) throw new Error('The drawing canvas is not laid out.');
    const side = SIZE * DEFAULT_ZOOM;
    return {
      x: box.x + (box.width - side) / 2 + (pixel.x + 0.5) * DEFAULT_ZOOM,
      y: box.y + (box.height - side) / 2 + (pixel.y + 0.5) * DEFAULT_ZOOM,
    };
  }

  private async drag(pixels: readonly Pixel[]): Promise<void> {
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
}
