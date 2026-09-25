import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { TagSpec } from '../../engine/engine-types';
import { TagDialogState } from './tag-dialog-state';

/** The width a frame thumbnail takes in the strip, so a tag's bar lines up above its frames. */
const COLUMN_WIDTH = '48px';

/**
 * Tags as bars over their frames, one column per frame so they line up with the strip; "Add tag"
 * covers `frameSelection`, and each bar opens the dialog to rename, change or delete it
 * (editor.md, U3).
 */
@Component({
  selector: 'lp-tag-bars',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tag-bars.html',
  styleUrl: './tag-bars.scss',
})
export class TagBars {
  private readonly engine = inject(EngineStore);
  private readonly dialog = inject(TagDialogState);
  protected readonly editor = inject(EditorStore);

  protected readonly document = this.engine.document;
  private readonly frameCount = computed(() => this.document()?.frames.length ?? 0);
  protected readonly gridColumns = computed(
    () => `repeat(${Math.max(this.frameCount(), 1)}, ${COLUMN_WIDTH})`,
  );

  protected columnsOf(tag: TagSpec): string {
    return `${tag.first + 1} / ${tag.last + 2}`;
  }

  protected addTag(): void {
    const selection = this.editor.frameSelection();
    if (!selection) return;
    this.dialog.open({ kind: 'add', first: selection.first, last: selection.last });
  }

  protected edit(tag: TagSpec): void {
    this.dialog.open({ kind: 'edit', tag });
  }
}
