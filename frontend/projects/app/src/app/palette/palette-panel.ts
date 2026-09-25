import { ChangeDetectionStrategy, Component, computed, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { PaletteEntryDialog } from './palette-entry-dialog';
import { PaletteEntryFlow } from './palette-entry-flow';
import { dropTarget, reorderTarget } from './palette-reorder';

/**
 * The editor page's palette panel (editor.md, U4): a grid of swatches, entry 0 the checkerboard
 * eraser colour, selectable but not editable. Selecting sets `colorIndex`; `[` and `]` move
 * through it. Add and edit open `PaletteEntryDialog`; remove and reorder (drag and drop, or Alt
 * with the arrows) apply directly.
 */
@Component({
  selector: 'lp-palette-panel',
  imports: [PaletteEntryDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './palette-panel.html',
  styleUrl: './palette-panel.scss',
})
export class PalettePanel {
  private readonly editor = inject(EditorStore);
  private readonly engine = inject(EngineStore);
  private readonly entryFlow = inject(PaletteEntryFlow);
  private dragFrom: number | null = null;

  protected readonly colorIndex = this.editor.colorIndex;
  protected readonly palette = computed(() => this.engine.document()?.palette ?? []);
  protected readonly isFull = computed(
    () => this.palette().length >= (this.engine.limits()?.maxPaletteEntries ?? Infinity),
  );

  constructor() {
    const shortcuts = inject(Shortcuts);
    inject(DestroyRef).onDestroy(
      shortcuts.register([
        { key: '[', label: 'palette.shortcut.previous_color', action: () => this.step(-1) },
        { key: ']', label: 'palette.shortcut.next_color', action: () => this.step(1) },
      ]),
    );
  }

  protected select(index: number): void {
    this.editor.colorIndex.set(index);
  }

  protected add(): void {
    this.entryFlow.openAdd();
  }

  protected edit(index: number): void {
    this.entryFlow.openEdit(index, this.palette()[index]);
  }

  protected async remove(index: number): Promise<void> {
    await this.engine.apply({ kind: 'removePaletteEntry', index });
  }

  protected onSwatchKeydown(event: KeyboardEvent, index: number): void {
    if (!event.altKey || (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight')) return;
    event.preventDefault();
    void this.move(
      index,
      reorderTarget(index, event.key === 'ArrowLeft' ? -1 : 1, this.palette().length),
    );
  }

  protected onDragStart(event: DragEvent, index: number): void {
    this.dragFrom = index;
    event.dataTransfer?.setData('text/plain', String(index));
  }

  protected onDragOver(event: DragEvent): void {
    event.preventDefault();
  }

  protected onDrop(event: DragEvent, index: number): void {
    event.preventDefault();
    const from = this.dragFrom;
    this.dragFrom = null;
    if (from !== null) void this.move(from, dropTarget(from, index, this.palette().length));
  }

  private async move(from: number, to: number | null): Promise<void> {
    if (to !== null) await this.engine.apply({ kind: 'movePaletteEntry', from, to });
  }

  private step(direction: -1 | 1): void {
    const size = this.palette().length;
    if (size === 0) return;
    const next = this.colorIndex() + direction;
    this.editor.colorIndex.set(Math.max(0, Math.min(next, size - 1)));
  }
}
