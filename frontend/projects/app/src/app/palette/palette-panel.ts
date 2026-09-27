import {
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  ElementRef,
  inject,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import type { Color } from '../engine/engine-types';
import { Icon } from '../ui/icon/icon';
import { PaletteEntryDialog } from './palette-entry-dialog';
import { PaletteEntryFlow } from './palette-entry-flow';
import { dropTarget, reorderTarget } from './palette-reorder';

/** The selected palette entry, as the panel describes it under the swatches. */
interface SelectedColor {
  readonly index: number;
  readonly color: Color;
}

/**
 * The editor page's palette panel (editor.md, U4): a grid of swatches, entry 0 the checkerboard
 * eraser colour, selectable but not editable; the selected one ringed twice and checked, then
 * named with its index and `#rrggbbaa` value under the grid. Selecting sets `colorIndex`; `[` and
 * `]` move through it. One Edit and Remove pair acts on the selected colour; Add and Edit open
 * `PaletteEntryDialog`; Remove and reorder (drag and drop, or Alt with the arrows) apply directly.
 */
@Component({
  selector: 'lp-palette-panel',
  imports: [Icon, PaletteEntryDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './palette-panel.html',
  styleUrl: './palette-panel.scss',
})
export class PalettePanel {
  private readonly editor = inject(EditorStore);
  private readonly engine = inject(EngineStore);
  private readonly entryFlow = inject(PaletteEntryFlow);
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private dragFrom: number | null = null;

  protected readonly id = idScope('palette-panel');

  protected readonly colorIndex = this.editor.colorIndex;
  protected readonly palette = computed(() => this.engine.document()?.palette ?? []);
  protected readonly isFull = computed(
    () => this.palette().length >= (this.engine.limits()?.maxPaletteEntries ?? Infinity),
  );
  protected readonly selected = computed((): SelectedColor | null => {
    const index = this.colorIndex();
    const color = this.palette()[index];
    return color === undefined ? null : { index, color };
  });
  /** Entry 0, the transparency, is neither edited nor removed. */
  protected readonly isEditable = computed(() => (this.selected()?.index ?? 0) > 0);

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

  protected edit(): void {
    const selected = this.selected();
    if (selected && selected.index > 0) this.entryFlow.openEdit(selected.index, selected.color);
  }

  protected async remove(): Promise<void> {
    const selected = this.selected();
    if (!selected || selected.index === 0) return;
    await this.engine.apply({ kind: 'removePaletteEntry', index: selected.index });
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

  /**
   * Moves an entry; the selection and the keyboard focus follow the colour moved, so that Alt with
   * an arrow pressed again keeps moving the same one.
   */
  private async move(from: number, to: number | null): Promise<void> {
    if (to === null) return;
    const before = this.palette();
    await this.engine.apply({ kind: 'movePaletteEntry', from, to });
    if (this.palette() === before) return;
    if (this.colorIndex() === from) this.editor.colorIndex.set(to);
    const host = this.host.nativeElement;
    const swatches = host.querySelectorAll<HTMLButtonElement>('.swatches .swatch');
    if (swatches[from] === document.activeElement) swatches[to]?.focus();
  }

  private step(direction: -1 | 1): void {
    const size = this.palette().length;
    if (size === 0) return;
    const next = this.colorIndex() + direction;
    this.editor.colorIndex.set(Math.max(0, Math.min(next, size - 1)));
  }
}
