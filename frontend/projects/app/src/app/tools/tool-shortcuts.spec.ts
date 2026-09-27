import { TestBed } from '@angular/core/testing';
import { EditorStore } from '../editor/editor-store';
import type { Shortcut } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { ShortcutsHelpState } from './shortcuts-help-state';
import { buildToolShortcuts } from './tool-shortcuts';

const NEW_ANIMATION = { title: 'Shortcuts', width: 8, height: 8, layerName: 'Base' };

interface Match {
  readonly primary?: boolean;
  readonly shift?: boolean;
}

function find(
  shortcuts: readonly Shortcut[],
  key: string,
  match: Match = {},
): Shortcut | undefined {
  return shortcuts.find(
    (candidate) =>
      candidate.key === key &&
      !!candidate.primary === !!match.primary &&
      !!candidate.shift === !!match.shift,
  );
}

describe('buildToolShortcuts', () => {
  async function setup() {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    const editor = TestBed.inject(EditorStore);
    const help = TestBed.inject(ShortcutsHelpState);
    return { engine, editor, help, shortcuts: buildToolShortcuts({ editor, engine, help }) };
  }

  it('selects a tool for each letter of the table', async () => {
    const { editor, shortcuts } = await setup();

    find(shortcuts, 'b')?.action();
    expect(editor.tool()).toBe('pencil');
    find(shortcuts, 'm')?.action();
    expect(editor.tool()).toBe('select');
  });

  it('selects the rectangle tool filled on Shift R', async () => {
    const { editor, shortcuts } = await setup();

    find(shortcuts, 'r', { shift: true })?.action();

    expect(editor.tool()).toBe('rectangle');
    expect(editor.rectangleFilled()).toBe(true);
  });

  it('toggles the grid on Shift G', async () => {
    const { editor, shortcuts } = await setup();
    const before = editor.showGrid();

    find(shortcuts, 'g', { shift: true })?.action();

    expect(editor.showGrid()).toBe(!before);
  });

  it('undoes and redoes on Ctrl/⌘ Z and Ctrl/⌘ Shift Z', async () => {
    const { engine, shortcuts } = await setup();
    await engine.apply({ kind: 'setTitle', title: 'Changed' });

    find(shortcuts, 'z', { primary: true })?.action();
    await vi.waitFor(() => expect(engine.document()?.title).toBe('Shortcuts'));

    find(shortcuts, 'z', { primary: true, shift: true })?.action();
    await vi.waitFor(() => expect(engine.document()?.title).toBe('Changed'));
  });

  it('also redoes on Ctrl Y', async () => {
    const { engine, shortcuts } = await setup();
    await engine.apply({ kind: 'setTitle', title: 'Changed' });
    find(shortcuts, 'z', { primary: true })?.action();
    await vi.waitFor(() => expect(engine.document()?.title).toBe('Shortcuts'));

    find(shortcuts, 'y', { primary: true })?.action();

    await vi.waitFor(() => expect(engine.document()?.title).toBe('Changed'));
  });

  it('opens the shortcuts help on ?', async () => {
    const { help, shortcuts } = await setup();

    find(shortcuts, '?')?.action();

    expect(help.isOpen()).toBe(true);
  });
});
