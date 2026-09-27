import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  ElementRef,
  inject,
  Injector,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { EditorStore, type FrameRange } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { FrameId } from '../../engine/engine-types';
import { Icon } from '../../ui/icon/icon';
import { dropPosition, stepPosition } from '../reorder';
import { FrameThumbnail } from './frame-thumbnail';
import { revealInline } from './reveal-inline';

/** The direction an arrow key moves a frame, `null` for every other key. */
function directionOf(key: string): 1 | -1 | null {
  if (key === 'ArrowRight') return 1;
  if (key === 'ArrowLeft') return -1;
  return null;
}

/**
 * The frame strip (editor.md, U3): the commands, then per frame a numbered thumbnail and its
 * duration in milliseconds. The active frame, the one drawn on (`activeFrame`), carries a strong
 * edge, a pencil and `aria-current`; the frames of `frameSelection`, extended with Shift-click,
 * a soft surface and `aria-pressed`. Frames are added after the active one, duplicated, deleted,
 * and reordered by drag and drop or Alt with the arrows, the focus following the frame moved; the
 * active frame scrolls into view when it changes.
 */
@Component({
  selector: 'lp-frame-list',
  imports: [FrameThumbnail, Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './frame-list.html',
  styleUrl: './frame-list.scss',
})
export class FrameList {
  private readonly engine = inject(EngineStore);
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly injector = inject(Injector);
  protected readonly editor = inject(EditorStore);
  private dragIndex: number | null = null;

  protected readonly id = idScope('frame-list');

  protected readonly document = this.engine.document;
  protected readonly limits = this.engine.limits;
  protected readonly frames = computed(() => this.document()?.frames ?? []);
  protected readonly changedAt = this.engine.state;

  constructor() {
    let previous = this.editor.activeFrame();
    effect(() => {
      const frame = this.editor.activeFrame();
      if (frame === previous) return;
      previous = frame;
      if (frame !== null) this.afterRender(() => this.reveal(frame));
    });
  }

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
    this.keepFocusOnActiveFrame();
  }

  /** Applies a whole duration; the field then shows the frame's duration, refused or not. */
  protected async setDuration(frame: FrameId, field: HTMLInputElement): Promise<void> {
    const durationMs = field.valueAsNumber;
    if (Number.isInteger(durationMs)) {
      await this.engine.apply({ kind: 'setFrameDuration', frame, durationMs });
    }
    const current = this.frames().find((candidate) => candidate.id === frame);
    if (current) field.value = String(current.durationMs);
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
    this.afterRender(() => this.cell(frame.id)?.querySelector<HTMLElement>('button')?.focus());
  }

  /** Brings a frame into the strip's view, clear of the sticky commands. */
  private reveal(frame: FrameId): void {
    const cell = this.cell(frame);
    const head = this.host.nativeElement.querySelector<HTMLElement>('.head');
    if (cell) revealInline(cell, head?.getBoundingClientRect().width ?? 0);
  }

  /** Deleting down to one frame disables the focused button: the focus goes to the active frame. */
  private keepFocusOnActiveFrame(): void {
    this.afterRender(() => {
      const focused = document.activeElement;
      const lost = !focused || focused === document.body || focused.matches(':disabled');
      const frame = this.editor.activeFrame();
      if (lost && frame !== null) this.cell(frame)?.querySelector<HTMLElement>('button')?.focus();
    });
  }

  private cell(frame: FrameId): HTMLElement | null {
    return this.host.nativeElement.querySelector<HTMLElement>(`[data-frame="${frame}"]`);
  }

  private afterRender(action: () => void): void {
    afterNextRender(action, { injector: this.injector });
  }
}
