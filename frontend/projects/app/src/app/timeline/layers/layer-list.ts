import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  Injector,
  signal,
} from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { LayerId } from '../../engine/engine-types';
import { Icon } from '../../ui/icon/icon';
import { dropPosition, stepPosition } from '../reorder';

/** Layers are stored bottom first; moving `ArrowUp` in the top-first list steps toward the end. */
function directionOf(key: string): 1 | -1 | null {
  if (key === 'ArrowUp') return 1;
  if (key === 'ArrowDown') return -1;
  return null;
}

/**
 * The layer list, top layer first (editor.md, U3). A click on a row's name makes it the active
 * layer, the one drawn on (`EditorStore.activeLayer`, plan C8), marked by its edge, its surface,
 * the word "Active" and `aria-current`. Each row shows or hides its layer, renames it on demand
 * (the edit button or a double click; Enter or leaving the field confirms, Escape reverts) and
 * deletes it; the list adds a layer on top and reorders by drag and drop or Alt with the arrows,
 * the focus following the layer moved.
 */
@Component({
  selector: 'lp-layer-list',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './layer-list.html',
  styleUrl: './layer-list.scss',
})
export class LayerList {
  private readonly engine = inject(EngineStore);
  private readonly transloco = inject(TranslocoService);
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly injector = inject(Injector);
  protected readonly editor = inject(EditorStore);
  private dragIndex: number | null = null;

  protected readonly document = this.engine.document;
  protected readonly limits = this.engine.limits;
  /** Top layer first, the reverse of the engine's bottom-first order. */
  protected readonly layers = computed(() => [...(this.document()?.layers ?? [])].reverse());
  /** The layer whose name is being edited, if any. */
  protected readonly renaming = signal<LayerId | null>(null);

  private underlyingIndex(displayedIndex: number): number {
    return this.layers().length - 1 - displayedIndex;
  }

  protected activate(layer: LayerId): void {
    this.editor.activeLayer.set(layer);
  }

  protected async addLayer(): Promise<void> {
    const name = this.transloco.translate('timeline.layers.new_name');
    const position = this.layers().length;
    await this.engine.apply({ kind: 'addLayer', position, name });
  }

  /** The row goes with its layer: the focus goes to the layer then active. */
  protected async deleteLayer(layer: LayerId): Promise<void> {
    await this.engine.apply({ kind: 'deleteLayer', layer });
    const active = this.editor.activeLayer();
    if (active !== null) this.focusRow(active);
  }

  protected nameOf(layer: LayerId): string {
    return this.document()?.layers.find((candidate) => candidate.id === layer)?.name ?? '';
  }

  protected startRename(layer: LayerId): void {
    this.renaming.set(layer);
    this.afterRender(() => {
      const field = this.row(layer)?.querySelector<HTMLInputElement>('.name-field');
      field?.focus();
      field?.select();
    });
  }

  /** Enter or leaving the field: applies a changed, non-empty name, then closes the field. */
  protected async finishRename(
    layer: LayerId,
    field: HTMLInputElement,
    keepFocus: boolean,
  ): Promise<void> {
    if (this.renaming() !== layer) return;
    this.renaming.set(null);
    if (keepFocus) this.focusRow(layer);
    const name = field.value.trim();
    if (name.length > 0 && name !== this.nameOf(layer)) {
      await this.engine.apply({ kind: 'renameLayer', layer, name });
    }
  }

  /** Escape: closes the field without a change, the focus back on the layer. */
  protected cancelRename(layer: LayerId): void {
    this.renaming.set(null);
    this.focusRow(layer);
  }

  protected async toggleVisibility(layer: LayerId, visible: boolean): Promise<void> {
    await this.engine.apply({ kind: 'setLayerVisibility', layer, visible: !visible });
  }

  protected onDragStart(displayedIndex: number): void {
    this.dragIndex = this.underlyingIndex(displayedIndex);
  }

  protected async onDrop(targetDisplayedIndex: number): Promise<void> {
    const from = this.dragIndex;
    this.dragIndex = null;
    if (from === null) return;
    const layer = this.document()?.layers[from];
    const target = this.underlyingIndex(targetDisplayedIndex);
    if (!layer || from === target) return;
    const position = dropPosition(target);
    await this.engine.apply({ kind: 'moveLayer', layer: layer.id, position });
  }

  protected async onKeydown(displayedIndex: number, event: KeyboardEvent): Promise<void> {
    const direction = event.altKey ? directionOf(event.key) : null;
    if (direction === null) return;
    event.preventDefault();
    const from = this.underlyingIndex(displayedIndex);
    const layer = this.document()?.layers[from];
    const position = stepPosition(from, this.layers().length, direction);
    if (position === null || !layer) return;
    await this.engine.apply({ kind: 'moveLayer', layer: layer.id, position });
    this.focusRow(layer.id);
  }

  private row(layer: LayerId): HTMLElement | null {
    return this.host.nativeElement.querySelector<HTMLElement>(`[data-layer="${layer}"]`);
  }

  /**
   * Focuses the layer's name once rendered, since moving or re-rendering a row drops its focus;
   * never while the focus has gone elsewhere in the meantime.
   */
  private focusRow(layer: LayerId): void {
    this.afterRender(() => {
      const focused = document.activeElement;
      const host = this.host.nativeElement;
      if (focused && focused !== document.body && !host.contains(focused)) return;
      this.row(layer)?.querySelector<HTMLElement>('.select')?.focus();
    });
  }

  private afterRender(action: () => void): void {
    afterNextRender(action, { injector: this.injector });
  }
}
