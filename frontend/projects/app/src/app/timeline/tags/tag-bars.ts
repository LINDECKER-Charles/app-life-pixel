import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { TagSpec } from '../../engine/engine-types';
import { Icon } from '../../ui/icon/icon';
import { TagDialogState } from './tag-dialog-state';

/**
 * Tags as bars over their frames, one column per frame so they line up with the strip, each with
 * its name and, played once, the word for it; "Add tag" covers `frameSelection`, and each bar
 * opens the dialog to rename, change or delete it (editor.md, U3).
 */
@Component({
  selector: 'lp-tag-bars',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './tag-bars.html',
  styleUrl: './tag-bars.scss',
})
export class TagBars {
  private readonly engine = inject(EngineStore);
  private readonly dialog = inject(TagDialogState);
  protected readonly editor = inject(EditorStore);

  protected readonly id = idScope('tag-bars');

  protected readonly document = this.engine.document;
  protected readonly frameCount = computed(() => Math.max(this.document()?.frames.length ?? 0, 1));

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
