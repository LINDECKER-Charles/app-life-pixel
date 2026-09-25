import type { EditorStore } from '../editor/editor-store';
import type { Shortcut } from '../editor/shortcuts';
import type { EngineStore } from '../engine/engine-store';
import { ShortcutsHelpState } from './shortcuts-help-state';
import { TOOL_DEFINITIONS } from './tool-definitions';

export interface ToolShortcutContext {
  readonly editor: EditorStore;
  readonly engine: EngineStore;
  readonly help: ShortcutsHelpState;
}

function toolShortcuts(editor: EditorStore): Shortcut[] {
  return TOOL_DEFINITIONS.map((definition) => ({
    key: definition.shortcutKey,
    label: definition.labelKey,
    action: () => editor.tool.set(definition.tool),
  }));
}

function undoRedoShortcuts(engine: EngineStore): Shortcut[] {
  return [
    { key: 'z', primary: true, label: 'tools.undo', action: () => void engine.undo() },
    { key: 'z', primary: true, shift: true, label: 'tools.redo', action: () => void engine.redo() },
    { key: 'y', primary: true, label: 'tools.redo', action: () => void engine.redo() },
  ];
}

/**
 * Shift `R` selects the rectangle tool filled; Shift `G` toggles the grid — a plain flag on the
 * shared `EditorStore`, needing no canvas geometry (editor.md, U4).
 */
function miscShortcuts(editor: EditorStore, help: ShortcutsHelpState): Shortcut[] {
  return [
    {
      key: 'r',
      shift: true,
      label: 'tools.shortcut.rectangle_filled',
      action: () => {
        editor.tool.set('rectangle');
        editor.rectangleFilled.set(true);
      },
    },
    {
      key: 'g',
      shift: true,
      label: 'tools.shortcut.grid',
      action: () => editor.showGrid.update((visible) => !visible),
    },
    { key: '?', label: 'tools.shortcuts_button', action: () => help.open() },
  ];
}

/** Every shortcut of editor.md's U4 table achievable from the tool bar alone. */
export function buildToolShortcuts(context: ToolShortcutContext): Shortcut[] {
  return [
    ...toolShortcuts(context.editor),
    ...undoRedoShortcuts(context.engine),
    ...miscShortcuts(context.editor, context.help),
  ];
}
