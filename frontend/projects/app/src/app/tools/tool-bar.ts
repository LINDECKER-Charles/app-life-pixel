import { ChangeDetectionStrategy, Component, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore, type EditorTool } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { ImportImageButton } from './import/import-image-button';
import { ImportSpriteSheetButton } from './import/import-sprite-sheet-button';
import { ShortcutsHelpDialog } from './shortcuts-help-dialog';
import { ShortcutsHelpState } from './shortcuts-help-state';
import { TOOL_DEFINITIONS } from './tool-definitions';
import { buildToolShortcuts } from './tool-shortcuts';

/**
 * The editor page's tool bar (editor.md, U4): one button per tool, `aria-pressed`, a tooltip with
 * its shortcut; the rectangle's filled/outlined switch; undo and redo bound to `canUndo` and
 * `canRedo`; both imports; the `?` button opening the shortcuts help dialog.
 */
@Component({
  selector: 'lp-tool-bar',
  imports: [ImportImageButton, ImportSpriteSheetButton, ShortcutsHelpDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tool-bar.html',
  styleUrl: './tool-bar.scss',
})
export class ToolBar {
  private readonly editor = inject(EditorStore);

  protected readonly engine = inject(EngineStore);
  protected readonly help = inject(ShortcutsHelpState);
  protected readonly tools = TOOL_DEFINITIONS;
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
