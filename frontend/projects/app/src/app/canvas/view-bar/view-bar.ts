import { ChangeDetectionStrategy, Component, computed, inject, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EditorStore, MAX_ONION_SKIN_FRAMES } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import type { Point } from '../../engine/engine-types';
import { describeShortcut } from '../../tools/shortcut-label';
import { TOOL_DEFINITIONS } from '../../tools/tool-definitions';
import { Icon } from '../../ui/icon/icon';
import { Tooltip } from '../../ui/tooltip/tooltip';
import type { CanvasViewport } from '../gesture/canvas-viewport';

/**
 * The bar under the canvas (plan C7, design-system/docs/accessibility.md): zoom out, the zoom in
 * percent, zoom in and fit, which call the canvas's own `CanvasViewport` as `-`, `+` and `0` do;
 * the grid and onion-skin switches, which call `EditorStore` as Shift `G` and `O` do, with the
 * onion skin's frame counts; then the pixel under the pointer or the keyboard cursor, the active
 * frame and layer, and the tool.
 */
@Component({
  selector: 'lp-view-bar',
  imports: [Icon, Tooltip, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './view-bar.html',
  styleUrl: './view-bar.scss',
})
export class ViewBar {
  private readonly engine = inject(EngineStore);

  protected readonly store = inject(EditorStore);

  /** The canvas's viewport, whose actions the zoom shortcuts call too. */
  readonly viewport = input.required<CanvasViewport>();
  /** The pixel to show, in the animation's coordinates, or none. */
  readonly position = input<Point | null>(null);

  protected readonly counts = Array.from({ length: MAX_ONION_SKIN_FRAMES + 1 }, (_, i) => i);
  protected readonly shortcut = {
    grid: describeShortcut({ key: 'g', shift: true }),
    onionSkin: describeShortcut({ key: 'o' }),
  };
  protected readonly zoomPercent = computed(() => this.store.zoom() * 100);
  protected readonly frame = computed(() => {
    const frames = this.engine.document()?.frames ?? [];
    const index = frames.findIndex((frame) => frame.id === this.store.activeFrame());
    return index < 0 ? null : { index: index + 1, count: frames.length };
  });
  protected readonly layerName = computed(() => {
    const layers = this.engine.document()?.layers ?? [];
    return layers.find((layer) => layer.id === this.store.activeLayer())?.name ?? null;
  });
  protected readonly toolLabelKey = computed(
    () => TOOL_DEFINITIONS.find((definition) => definition.tool === this.store.tool())?.labelKey,
  );

  protected setRange(before: number, after: number): void {
    this.store.setOnionSkinRange({ before, after });
  }
}
