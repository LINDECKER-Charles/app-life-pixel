import { ChangeDetectionStrategy, Component, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore, type EditorTool } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { Icon } from '../ui/icon/icon';
import { Tooltip } from '../ui/tooltip/tooltip';
import { ImportImageButton } from './import/import-image-button';
import { ImportSpriteSheetButton } from './import/import-sprite-sheet-button';
import { describeShortcut } from './shortcut-label';
import { ShortcutsHelpDialog } from './shortcuts-help-dialog';
import { ShortcutsHelpState } from './shortcuts-help-state';
import { TOOL_DEFINITIONS } from './tool-definitions';
import { buildToolShortcuts } from './tool-shortcuts';

/**
 * The editor's tool rail (editor.md, U4; design-system/docs/components.md, "Editor toolbar and
 * canvas"): one icon button per tool, named by its label, `aria-pressed`, with a tooltip holding
 * its shortcut; the filled switch beside the rectangle; undo and redo side by side, bound to
 * `canUndo` and `canRedo`; both imports; the `?` button opening the shortcuts help dialog.
 */
@Component({
  selector: 'lp-tool-bar',
  imports: [
    Icon,
    ImportImageButton,
    ImportSpriteSheetButton,
    ShortcutsHelpDialog,
    Tooltip,
    TranslocoPipe,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tool-bar.html',
  styleUrl: './tool-bar.scss',
})
export class ToolBar {
  private readonly editor = inject(EditorStore);

  protected readonly engine = inject(EngineStore);
  protected readonly help = inject(ShortcutsHelpState);
  protected readonly tools = TOOL_DEFINITIONS.map((definition) => ({
    ...definition,
    shortcut: describeShortcut({ key: definition.shortcutKey }),
  }));
  protected readonly shortcut = {
    filled: describeShortcut({ key: 'r', shift: true }),
    undo: describeShortcut({ key: 'z', primary: true }),
    redo: describeShortcut({ key: 'z', primary: true, shift: true }),
    help: describeShortcut({ key: '?' }),
  };
  protected readonly tool = this.editor.tool;
  protected readonly rectangleFilled = this.editor.rectangleFilled;

  constructor() {
    const shortcuts = inject(Shortcuts);
    inject(DestroyRef).onDestroy(
      shortcuts.register(
        buildToolShortcuts({ editor: this.editor, engine: this.engine, help: this.help }),
      ),
    );
  }

  protected selectTool(tool: EditorTool): void {
    this.editor.tool.set(tool);
  }

  protected toggleFilled(): void {
    this.editor.rectangleFilled.update((filled) => !filled);
  }
}
