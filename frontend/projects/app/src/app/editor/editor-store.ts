import { computed, inject, Injectable, linkedSignal, signal } from '@angular/core';
import { EngineStore } from '../engine/engine-store';
import type { Area, FrameId, LayerId, Point } from '../engine/engine-types';
import { nearestSurvivor } from './nearest-survivor';

export type EditorTool = 'pencil' | 'eraser' | 'fill' | 'line' | 'rectangle' | 'select';

/** A range of frames by position, both ends included: what a tag covers. */
export interface FrameRange {
  readonly first: number;
  readonly last: number;
}

/** How many frames before and after the active one are drawn faintly beneath it. */
export interface OnionSkin {
  readonly enabled: boolean;
  readonly before: number;
  readonly after: number;
}

/** Palette entry 0 is transparent — the eraser's —, so drawing starts with the first colour. */
const FIRST_COLOR_INDEX = 1;
const DEFAULT_ZOOM = 8;
const DEFAULT_ONION_SKIN: OnionSkin = { enabled: false, before: 1, after: 1 };
const ORIGIN: Point = { x: 0, y: 0 };

function sameIds(a: readonly number[], b: readonly number[]): boolean {
  return a.length === b.length && a.every((id, index) => id === b[index]);
}

/** Keeps the index within the palette; with no document, keeps it as it is. */
function clampColorIndex(index: number, paletteSize: number): number {
  return paletteSize === 0 ? index : Math.max(0, Math.min(index, paletteSize - 1));
}

function clampRange(range: FrameRange | null, frameCount: number): FrameRange | null {
  if (range === null || range.first >= frameCount) return null;
  return { first: range.first, last: Math.min(range.last, frameCount - 1) };
}

/**
 * The editor's interface state, shared by the canvas, the timeline, the palette, the tools and
 * the export dialog (U2 to U5): each field is a writable signal they read and set. It follows the
 * document: an active layer or frame that disappears falls back to the nearest one, the colour
 * index and the frame selection stay within the palette and the frames, and the selection is
 * dropped when the canvas changes size.
 */
@Injectable({ providedIn: 'root' })
export class EditorStore {
  private readonly engine = inject(EngineStore);
  private readonly layerIds = computed(
    () => this.engine.document()?.layers.map((layer) => layer.id) ?? [],
    { equal: sameIds },
  );
  private readonly frameIds = computed(
    () => this.engine.document()?.frames.map((frame) => frame.id) ?? [],
    { equal: sameIds },
  );
  private readonly paletteSize = computed(() => this.engine.document()?.palette.length ?? 0);
  private readonly canvasSize = computed(() => {
    const document = this.engine.document();
    return document ? `${document.width}×${document.height}` : null;
  });

  readonly tool = signal<EditorTool>('pencil');
  readonly rectangleFilled = signal(false);
  readonly colorIndex = linkedSignal<number, number>({
    source: this.paletteSize,
    computation: (size, previous) => clampColorIndex(previous?.value ?? FIRST_COLOR_INDEX, size),
  });
  /** The layer drawn on; by default the topmost, since layers go from bottom to top. */
  readonly activeLayer = linkedSignal<readonly LayerId[], LayerId | null>({
    source: this.layerIds,
    computation: (ids, previous) =>
      nearestSurvivor({
        previousIds: previous?.source ?? [],
        ids,
        active: previous?.value ?? null,
      }) ??
      ids.at(-1) ??
      null,
  });
  /** The frame drawn on; by default the first. */
  readonly activeFrame = linkedSignal<readonly FrameId[], FrameId | null>({
    source: this.frameIds,
    computation: (ids, previous) =>
      nearestSurvivor({
        previousIds: previous?.source ?? [],
        ids,
        active: previous?.value ?? null,
      }) ??
      ids.at(0) ??
      null,
  });
  readonly frameSelection = linkedSignal<number, FrameRange | null>({
    source: () => this.frameIds().length,
    computation: (count, previous) => clampRange(previous?.value ?? null, count),
  });
  readonly zoom = signal(DEFAULT_ZOOM);
  readonly pan = signal<Point>(ORIGIN);
  readonly showGrid = signal(true);
  readonly onionSkin = signal<OnionSkin>(DEFAULT_ONION_SKIN);
  readonly selection = linkedSignal<string | null, Area | null>({
    source: this.canvasSize,
    computation: (size, previous) => (previous?.source === size ? previous.value : null),
  });
  readonly playing = signal(false);
}
