import { expect, test } from '@playwright/test';
import {
  BLINK_TAG,
  COLOR,
  DOT,
  DURATIONS_MS,
  FILL_AT,
  IDLE_TAG,
  LINE,
  PENCIL_STROKE,
} from './drawing';
import { EditorPage } from './editor-page';
import { expectPlayback, PlayerPage } from './player-page';

// M2's critical path (editor.md, U6): draw, animate, export, then play the export where a site
// would, with the pointer.
test('draws, animates, exports and plays an animation', async ({ page, browser }) => {
  const editor = new EditorPage(page);
  await editor.open();
  await editor.create();

  await test.step('draws frame 1 with the pencil, the line and the fill', async () => {
    await expect(editor.canvas).toBeVisible();
    await editor.tool('Pencil').click();
    await expect(editor.tool('Pencil')).toHaveAttribute('aria-pressed', 'true');
    await editor.swatch(COLOR.black.index).click();
    await editor.drag(PENCIL_STROKE);
    await editor.tool('Line').click();
    await editor.swatch(COLOR.red.index).click();
    await editor.drag([LINE.from, LINE.to]);
    await editor.tool('Fill').click();
    await editor.swatch(COLOR.blue.index).click();
    await editor.click(FILL_AT);
    await expect(editor.button('Undo')).toBeEnabled();
  });

  await test.step('adds frame 2 and draws on it', async () => {
    await editor.button('Add frame').click();
    await expect(editor.frameButton(2)).toBeVisible();
    await editor.frameButton(2).click();
    await expect(editor.frameButton(2)).toHaveAttribute('aria-pressed', 'true');
    await editor.tool('Pencil').click();
    await editor.swatch(COLOR.green.index).click();
    await editor.click(DOT);
  });

  await test.step('sets the durations', async () => {
    for (const [index, duration] of DURATIONS_MS.entries()) {
      await editor.duration(index + 1).fill(String(duration));
      await editor.duration(index + 1).press('Tab');
      await expect(editor.duration(index + 1)).toHaveValue(String(duration));
    }
  });

  await test.step('tags frame 1 idle, looping, and frame 2 blink, played once', async () => {
    await editor.frameButton(1).click();
    await editor.button('Add tag').click();
    await editor.textbox('Name').fill(IDLE_TAG);
    await editor.button('Save').click();
    await expect(editor.tagBar(IDLE_TAG)).toBeVisible();

    await editor.frameButton(2).click();
    await editor.button('Add tag').click();
    await editor.textbox('Name').fill(BLINK_TAG);
    await editor.radio('Play once').check();
    await editor.button('Save').click();
    await expect(editor.tagBar(BLINK_TAG)).toBeVisible();
  });

  await test.step('undoes and redoes the last change', async () => {
    await editor.button('Undo').click();
    await expect(editor.tagBar(BLINK_TAG)).toBeHidden();
    await expect(editor.button('Redo')).toBeEnabled();
    await editor.button('Redo').click();
    await expect(editor.tagBar(BLINK_TAG)).toBeVisible();
    await expect(editor.button('Redo')).toBeDisabled();
  });

  const files = await test.step('exports: five formats with their sizes, then WASM', async () => {
    await editor.button('Export').click();
    await expect(editor.exportDialog).toBeVisible();
    await editor.expectEveryFormatSized();
    const download = editor.exportRow('WASM').getByRole('button', { name: 'Download' });
    return editor.downloadWasm(() => download.click());
  });

  await test.step('plays the export with its loader, frame by frame', async () => {
    const player = new PlayerPage(await browser.newPage());
    await player.open(files);
    await expectPlayback(player);
    await player.page.close();
  });
});
