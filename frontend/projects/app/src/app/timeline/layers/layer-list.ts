import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { LayerId } from '../../engine/engine-types';
import { dropPosition, stepPosition } from '../reorder';

/** Layers are stored bottom first; moving `ArrowUp` in the top-first list steps toward the end. */
function directionOf(key: string): 1 | -1 | null {
  if (key === 'ArrowUp') return 1;
  if (key === 'ArrowDown') return -1;
  return null;
}

/**
 * The layer list, top layer first: add, delete, rename in place, show or hide, and reorder by
 * drag and drop or Alt with the arrows (editor.md, U3).
 */
@Component({
  selector: 'lp-layer-list',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './layer-list.html',
  styleUrl: './layer-list.scss',
})
export class LayerList {
  private readonly engine = inject(EngineStore);
  private readonly transloco = inject(TranslocoService);
  protected readonly editor = inject(EditorStore);
  private dragIndex: number | null = null;

  protected readonly document = this.engine.document;
  protected readonly limits = this.engine.limits;
  /** Top layer first, the reverse of the engine's bottom-first order. */
  protected readonly layers = computed(() => [...(this.document()?.layers ?? [])].reverse());

  private underlyingIndex(displayedIndex: number): number {
    return this.layers().length - 1 - displayedIndex;
  }

  protected async addLayer(): Promise<void> {
    const name = this.transloco.translate('timeline.layers.new_name');
    const position = this.layers().length;
    await this.engine.apply({ kind: 'addLayer', position, name });
  }

  protected async deleteLayer(layer: LayerId): Promise<void> {
    await this.engine.apply({ kind: 'deleteLayer', layer });
  }

  protected nameOf(layer: LayerId): string {
    return this.document()?.layers.find((candidate) => candidate.id === layer)?.name ?? '';
  }

  protected async rename(layer: LayerId, field: HTMLInputElement): Promise<void> {
    const name = field.value.trim();
    if (name.length > 0 && name !== this.nameOf(layer)) {
      await this.engine.apply({ kind: 'renameLayer', layer, name });
    }
    field.value = this.nameOf(layer);
  }

  protected revert(layer: LayerId, field: HTMLInputElement): void {
    field.value = this.nameOf(layer);
    field.blur();
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
  }
}
