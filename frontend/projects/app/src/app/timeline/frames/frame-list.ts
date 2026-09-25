import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore, type FrameRange } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { FrameId } from '../../engine/engine-types';
import { dropPosition, stepPosition } from '../reorder';
import { FrameThumbnail } from './frame-thumbnail';

/** The direction an arrow key moves a frame, `null` for every other key. */
function directionOf(key: string): 1 | -1 | null {
  if (key === 'ArrowRight') return 1;
  if (key === 'ArrowLeft') return -1;
  return null;
}

/**
 * The frame strip: a thumbnail and a duration field per frame, the active one highlighted, added
 * after it, duplicated, deleted, reordered by drag and drop or Alt with the arrows, and extended
 * with Shift-click (editor.md, U3).
 */
@Component({
  selector: 'lp-frame-list',
  imports: [FrameThumbnail, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './frame-list.html',
  styleUrl: './frame-list.scss',
})
export class FrameList {
  private readonly engine = inject(EngineStore);
  protected readonly editor = inject(EditorStore);
  private dragIndex: number | null = null;

  protected readonly document = this.engine.document;
  protected readonly limits = this.engine.limits;
  protected readonly frames = computed(() => this.document()?.frames ?? []);
  protected readonly changedAt = this.engine.state;

  protected indexOf(frame: FrameId): number {
    return this.frames().findIndex((candidate) => candidate.id === frame);
  }

  protected isSelected(index: number): boolean {
    const range = this.editor.frameSelection();
    return range !== null && index >= range.first && index <= range.last;
  }

  protected select(index: number, event: MouseEvent): void {
    const frame = this.frames()[index];
    if (!frame) return;
    if (event.shiftKey) {
      this.extendSelection(index);
      return;
    }
    this.editor.activeFrame.set(frame.id);
    this.editor.frameSelection.set({ first: index, last: index });
  }

  private extendSelection(index: number): void {
    const anchor = this.indexOf(this.editor.activeFrame() ?? -1);
    const start = anchor < 0 ? index : anchor;
    const range: FrameRange = { first: Math.min(start, index), last: Math.max(start, index) };
    this.editor.frameSelection.set(range);
  }

  protected async addFrame(): Promise<void> {
    const position = this.indexOf(this.editor.activeFrame() ?? -1) + 1;
    const durationMs = this.limits()?.defaultFrameDurationMs ?? 0;
    await this.engine.apply({ kind: 'addFrame', position, durationMs });
  }

  protected async duplicateFrame(): Promise<void> {
    const frame = this.editor.activeFrame();
    if (frame === null) return;
    await this.engine.apply({ kind: 'duplicateFrame', frame });
  }

  protected async deleteFrame(): Promise<void> {
    const frame = this.editor.activeFrame();
    if (frame === null) return;
    await this.engine.apply({ kind: 'deleteFrame', frame });
  }

  protected async setDuration(frame: FrameId, durationMs: number): Promise<void> {
    if (!Number.isInteger(durationMs)) return;
    await this.engine.apply({ kind: 'setFrameDuration', frame, durationMs });
  }

  protected onDragStart(index: number): void {
    this.dragIndex = index;
  }

  protected async onDrop(targetIndex: number): Promise<void> {
    const from = this.dragIndex;
    this.dragIndex = null;
    const frame = this.frames()[from ?? -1];
    if (from === null || !frame || from === targetIndex) return;
    const position = dropPosition(targetIndex);
    await this.engine.apply({ kind: 'moveFrame', frame: frame.id, position });
  }

  protected async onKeydown(index: number, event: KeyboardEvent): Promise<void> {
    const direction = event.altKey ? directionOf(event.key) : null;
    if (direction === null) return;
    event.preventDefault();
    const position = stepPosition(index, this.frames().length, direction);
    const frame = this.frames()[index];
    if (position === null || !frame) return;
    await this.engine.apply({ kind: 'moveFrame', frame: frame.id, position });
  }
}
