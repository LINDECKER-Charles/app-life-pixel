import { expect, test, type Locator, type Page } from '@playwright/test';
import {
  BLINK_TAG,
  COLOR,
  DOT,
  DURATIONS_MS,
  FILL_AT,
  IDLE_TAG,
  LINE,
  PENCIL_STROKE,
  SIZE,
  TITLE,
  type Pixel,
} from './drawing';
import { EditorPage, tabTo } from './editor-page';
import { expectPlayback, PlayerPage } from './player-page';

/** Palette entry 1, the first colour, is selected when an animation opens. */
const FIRST_COLOR_INDEX = 1;

/**
 * The canvas's pixel cursor, driven by the arrow keys: it starts at the top-left pixel, and Enter
 * applies the tool where it stands (editor.md, U2).
 */
class KeyboardCursor {
  private at: Pixel = { x: 0, y: 0 };

  constructor(private readonly page: Page) {}

  async moveTo(target: Pixel): Promise<void> {
    const dx = target.x - this.at.x;
    const dy = target.y - this.at.y;
    await this.press(dx < 0 ? 'ArrowLeft' : 'ArrowRight', Math.abs(dx));
    await this.press(dy < 0 ? 'ArrowUp' : 'ArrowDown', Math.abs(dy));
    this.at = target;
  }

  async applyAt(target: Pixel): Promise<void> {
    await this.moveTo(target);
    await this.page.keyboard.press('Enter');
  }

  private async press(key: string, times: number): Promise<void> {
    for (let count = 0; count < times; count++) await this.page.keyboard.press(key);
  }
}

/** Moves through the palette with `[` and `]`, as the shortcuts dialog lists them. */
async function stepColor(editor: EditorPage, from: number, to: number): Promise<void> {
  const key = to < from ? '[' : ']';
  for (let count = 0; count < Math.abs(to - from); count++) await editor.page.keyboard.press(key);
  await expect(editor.swatch(to)).toHaveAttribute('aria-pressed', 'true');
}

/** Replaces a field's value by typing, as a person does after selecting it all. */
async function typeInto(page: Page, field: Locator, value: string): Promise<void> {
  await tabTo(page, field);
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.type(value);
}

async function addTag(editor: EditorPage, frame: number, name: string): Promise<void> {
  const page = editor.page;
  await tabTo(page, editor.frameButton(frame), true);
  await page.keyboard.press('Enter');
  await expect(editor.frameButton(frame)).toHaveAttribute('aria-pressed', 'true');
  await tabTo(page, editor.button('Add tag'), true);
  await page.keyboard.press('Enter');
  await expect(editor.tagDialog).toBeVisible();
  await typeInto(page, editor.textbox('Name'), name);
  // Save turns enabled once the name is valid: Enter submits the form only then.
  await expect(editor.button('Save')).toBeEnabled();
}

// The same drawing as the pointer journey, with the keyboard alone (editor.md, U6): every control
// reached with Tab, every tool and colour by its shortcut, every pixel with the arrows and Enter.
test('draws the same animation with the keyboard alone', async ({ page, browser }) => {
  const editor = new EditorPage(page);
  const cursor = new KeyboardCursor(page);
  await editor.open();

  await test.step('creates the animation', async () => {
    await typeInto(page, editor.textbox('Title'), TITLE);
    await typeInto(page, editor.spinbutton('Width'), String(SIZE));
    await typeInto(page, editor.spinbutton('Height'), String(SIZE));
    await page.keyboard.press('Enter');
    await editor.expectReady();
  });

  await test.step('draws frame 1 with the pencil, the line and the fill', async () => {
    // A keyboard user draws what they see: the canvas must be shown, not merely focusable.
    await expect(editor.canvas).toBeVisible();
    await tabTo(page, editor.canvas);
    await page.keyboard.press('b');
    await expect(editor.tool('Pencil')).toHaveAttribute('aria-pressed', 'true');
    await stepColor(editor, FIRST_COLOR_INDEX, COLOR.black.index);
    for (const pixel of PENCIL_STROKE) await cursor.applyAt(pixel);
    await page.keyboard.press('l');
    await expect(editor.tool('Line')).toHaveAttribute('aria-pressed', 'true');
    await stepColor(editor, COLOR.black.index, COLOR.red.index);
    await cursor.applyAt(LINE.from);
    await cursor.applyAt(LINE.to);
    await page.keyboard.press('g');
    await expect(editor.tool('Fill')).toHaveAttribute('aria-pressed', 'true');
    await stepColor(editor, COLOR.red.index, COLOR.blue.index);
    await cursor.applyAt(FILL_AT);
    await expect(editor.button('Undo')).toBeEnabled();
  });

  await test.step('adds frame 2 and draws on it', async () => {
    await tabTo(page, editor.button('Add frame'));
    await page.keyboard.press('Enter');
    await expect(editor.frameButton(2)).toBeVisible();
    await page.keyboard.press('.');
    await expect(editor.frameButton(2)).toHaveAttribute('aria-pressed', 'true');
    await tabTo(page, editor.canvas, true);
    await page.keyboard.press('b');
    await stepColor(editor, COLOR.blue.index, COLOR.green.index);
    await cursor.applyAt(DOT);
  });

  await test.step('sets the durations', async () => {
    for (const [index, duration] of DURATIONS_MS.entries()) {
      await typeInto(page, editor.duration(index + 1), String(duration));
      await page.keyboard.press('Tab');
      await expect(editor.duration(index + 1)).toHaveValue(String(duration));
    }
  });

  await test.step('tags frame 1 idle, looping, and frame 2 blink, played once', async () => {
    await addTag(editor, 1, IDLE_TAG);
    await page.keyboard.press('Enter');
    await expect(editor.tagBar(IDLE_TAG)).toBeVisible();

    await addTag(editor, 2, BLINK_TAG);
    await tabTo(page, editor.radio('Loop'));
    await page.keyboard.press('ArrowDown');
    await expect(editor.radio('Play once')).toBeChecked();
    await tabTo(page, editor.button('Save'));
    await page.keyboard.press('Enter');
    await expect(editor.tagBar(BLINK_TAG)).toBeVisible();
  });

  await test.step('undoes and redoes the last change', async () => {
    await tabTo(page, editor.canvas, true);
    await page.keyboard.press('ControlOrMeta+z');
    await expect(editor.tagBar(BLINK_TAG)).toBeHidden();
    await page.keyboard.press('ControlOrMeta+Shift+z');
    await expect(editor.tagBar(BLINK_TAG)).toBeVisible();
  });

  const files = await test.step('exports: five formats with their sizes, then WASM', async () => {
    await page.keyboard.press('ControlOrMeta+e');
    await expect(editor.exportDialog).toBeVisible();
    await editor.expectEveryFormatSized();
    return editor.downloadWasm(async () => {
      await tabTo(page, editor.exportRow('WASM').getByRole('button', { name: 'Download' }));
      await page.keyboard.press('Enter');
    });
  });

  await test.step('plays the export with its loader, frame by frame', async () => {
    const player = new PlayerPage(await browser.newPage());
    await player.open(files);
    await expectPlayback(player);
    await player.page.close();
  });
});
