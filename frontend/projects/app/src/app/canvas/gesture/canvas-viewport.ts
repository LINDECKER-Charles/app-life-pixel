import {
  clampZoom,
  fitZoom,
  originFor,
  zoomAroundPoint,
  type Viewport,
} from '../geometry/canvas-geometry';
import type { Point } from '../../engine/engine-types';

export interface CanvasViewportDeps {
  readonly getViewport: () => Viewport;
  readonly setZoom: (zoom: number) => void;
  readonly setPan: (pan: Point) => void;
}

/** Pan-by-drag and zoom, pointer- and keyboard-driven alike (editor.md, U2's "Zoom and pan"). */
export class CanvasViewport {
  private panPointerId: number | null = null;
  private panPointerStart: Point | null = null;
  private panOriginStart: Point | null = null;

  constructor(private readonly deps: CanvasViewportDeps) {}

  get isPanning(): boolean {
    return this.panPointerId !== null;
  }

  startPan(pointerId: number, client: Point): void {
    this.panPointerId = pointerId;
    this.panPointerStart = client;
    this.panOriginStart = this.deps.getViewport().pan;
  }

  updatePan(pointerId: number, client: Point): void {
    if (pointerId !== this.panPointerId || !this.panPointerStart || !this.panOriginStart) return;
    this.deps.setPan({
      x: this.panOriginStart.x + (client.x - this.panPointerStart.x),
      y: this.panOriginStart.y + (client.y - this.panPointerStart.y),
    });
  }

  endPan(pointerId: number): void {
    if (pointerId !== this.panPointerId) return;
    this.panPointerId = null;
    this.panPointerStart = null;
    this.panOriginStart = null;
  }

  /** Ctrl or ⌘ with the wheel, and pinch (browsers report it as a ctrl-wheel too). */
  zoomWheel(client: Point, deltaY: number): void {
    const viewport = this.deps.getViewport();
    const nextZoom = clampZoom(viewport.zoom + (deltaY < 0 ? 1 : -1));
    if (nextZoom === viewport.zoom || viewport.content.width === 0) return;
    this.deps.setZoom(nextZoom);
    this.deps.setPan(zoomAroundPoint(client, viewport, nextZoom));
  }

  zoomBy(delta: number): void {
    const viewport = this.deps.getViewport();
    this.deps.setZoom(clampZoom(viewport.zoom + delta));
  }

  /** `0`: the largest integer zoom that fits the content, re-centred. */
  zoomFit(): void {
    const viewport = this.deps.getViewport();
    if (viewport.content.width === 0) return;
    this.deps.setZoom(fitZoom(viewport.view, viewport.content));
    this.deps.setPan({ x: 0, y: 0 });
  }

  /** Shift with the arrows: a fixed step, independent of zoom. */
  panByStep(delta: Point, stepPx: number): void {
    const pan = this.deps.getViewport().pan;
    this.deps.setPan({ x: pan.x - delta.x * stepPx, y: pan.y - delta.y * stepPx });
  }

  /** The content's top-left in client coordinates, for the current viewport. */
  origin(): Point {
    return originFor(this.deps.getViewport());
  }
}
