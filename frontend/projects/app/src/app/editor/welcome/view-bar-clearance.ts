import {
  afterNextRender,
  DestroyRef,
  ElementRef,
  inject,
  type Signal,
  signal,
} from '@angular/core';

/**
 * The height, in pixels, of the canvas's view bar beside the calling component in the stage,
 * following it as it wraps: what the welcome leaves free at the bottom of the stage, so that it
 * never covers the bar (0 until measured, or where sizes cannot be observed).
 */
export function viewBarClearance(): Signal<number> {
  const host = inject<ElementRef<HTMLElement>>(ElementRef).nativeElement;
  const destroyRef = inject(DestroyRef);
  const height = signal(0);
  afterNextRender(() => {
    const bar = host.parentElement?.querySelector<HTMLElement>('lp-view-bar');
    if (!bar || typeof ResizeObserver === 'undefined') return;
    const measure = (): void => height.set(bar.offsetHeight);
    const observer = new ResizeObserver(measure);
    observer.observe(bar);
    measure();
    destroyRef.onDestroy(() => observer.disconnect());
  });
  return height.asReadonly();
}
