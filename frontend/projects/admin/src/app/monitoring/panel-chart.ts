import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  effect,
  ElementRef,
  inject,
  input,
  viewChild,
} from '@angular/core';
import { ChartData } from './chart-data';
import { ChartHandle, ChartRenderer } from './chart-renderer';

const HEIGHT = 280;
const FALLBACK_WIDTH = 720;
const COLOR_SCHEME = '(prefers-color-scheme: dark)';

/**
 * A panel's chart, drawn by the chart renderer and redrawn when its data, its width or the
 * colour scheme changes. The canvas is hidden from assistive technologies: the panel's table
 * says the same in text.
 */
@Component({
  selector: 'lp-panel-chart',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `<div #plot class="plot" aria-hidden="true"></div>`,
  styles: `
    .plot {
      min-height: 280px;
    }
  `,
})
export class PanelChart {
  private readonly renderer = inject(ChartRenderer);
  private readonly plot = viewChild.required<ElementRef<HTMLElement>>('plot');
  private handle: ChartHandle | null = null;
  private drawing = 0;

  readonly data = input.required<ChartData>();
  readonly format = input.required<(value: number) => string>();

  constructor() {
    effect(() => void this.draw(this.data(), this.format()));
    const destroyRef = inject(DestroyRef);
    const observer =
      typeof ResizeObserver === 'function'
        ? new ResizeObserver(() => this.handle?.resize(this.width()))
        : null;
    effect(() => observer?.observe(this.plot().nativeElement));
    const scheme = typeof matchMedia === 'function' ? matchMedia(COLOR_SCHEME) : null;
    const redraw = (): void => void this.draw(this.data(), this.format());
    scheme?.addEventListener('change', redraw);
    destroyRef.onDestroy(() => {
      observer?.disconnect();
      scheme?.removeEventListener('change', redraw);
      this.handle?.destroy();
    });
  }

  private width(): number {
    return this.plot().nativeElement.clientWidth || FALLBACK_WIDTH;
  }

  private async draw(data: ChartData, format: (value: number) => string): Promise<void> {
    const drawing = ++this.drawing;
    const target = this.plot().nativeElement;
    const handle = await this.renderer.render(target, {
      data,
      format,
      width: this.width(),
      height: HEIGHT,
    });
    if (drawing !== this.drawing) {
      handle.destroy();
      return;
    }
    this.handle?.destroy();
    this.handle = handle;
  }
}
