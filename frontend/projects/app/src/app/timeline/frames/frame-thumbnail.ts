import {
  ChangeDetectionStrategy,
  Component,
  effect,
  ElementRef,
  inject,
  input,
  viewChild,
} from '@angular/core';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import type { FrameId } from '../../engine/engine-types';

/** Runs `draw` at most once per animation frame; jsdom lacks `requestAnimationFrame` before v27. */
const schedule: (draw: () => void) => void =
  typeof requestAnimationFrame === 'function'
    ? requestAnimationFrame
    : (draw) => setTimeout(draw, 0);

/**
 * A frame's thumbnail: the engine's render of `frame`, redrawn at most once per animation frame
 * whenever `frame` or `changedAt` changes. Drawing pixels needs a real 2D canvas context, absent
 * in the test environment: a missing context is not an error, just nothing drawn (editor.md, U3).
 */
@Component({
  selector: 'lp-frame-thumbnail',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<canvas #canvas class="thumbnail" width="48" height="48"></canvas>`,
  styles: `
    .thumbnail {
      display: block;
      width: 48px;
      height: 48px;
      image-rendering: pixelated;
      background-color: var(--lp-color-surface);
    }
  `,
})
export class FrameThumbnail {
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly canvas = viewChild.required<ElementRef<HTMLCanvasElement>>('canvas');
  private scheduled = false;

  readonly frame = input.required<FrameId>();
  /** Any value that changes when the thumbnail must be redrawn, e.g. the engine's state. */
  readonly changedAt = input.required<unknown>();

  constructor() {
    effect(() => {
      this.frame();
      this.changedAt();
      this.scheduleDraw();
    });
  }

  private scheduleDraw(): void {
    if (this.scheduled) return;
    this.scheduled = true;
    schedule(() => {
      this.scheduled = false;
      void this.draw();
    });
  }

  /** A frame gone by the time it renders — deleted mid-flight — is left blank, not reported. */
  private async draw(): Promise<void> {
    const frame = this.frame();
    const rendered = await this.engine.render({ frame }).catch(() => null);
    if (!rendered || this.frame() !== frame) return;
    const element = this.canvas().nativeElement;
    element.width = rendered.width;
    element.height = rendered.height;
    try {
      const context = element.getContext('2d');
      if (!context) return;
      context.imageSmoothingEnabled = false;
      const pixels = new Uint8ClampedArray(rendered.pixels);
      context.putImageData(new ImageData(pixels, rendered.width, rendered.height), 0, 0);
    } catch {
      // No 2D context in this environment (the test environment ships none): nothing to draw.
    }
  }
}
