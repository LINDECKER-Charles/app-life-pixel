import type { Point } from '../../engine/engine-types';

export interface Size {
  readonly width: number;
  readonly height: number;
}

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 64;
/** The grid shows from this zoom, one line per pixel (editor.md, U2). */
export const GRID_MIN_ZOOM = 8;
/** Alpha of the nearest onion-skin frame; halved for each step further (editor.md, U2). */
export const ONION_SKIN_BASE_ALPHA = 0.3;

export function clampZoom(zoom: number): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Math.round(zoom)));
}

/** The largest integer zoom that fits `content` inside `view`, at least `MIN_ZOOM`. */
export function fitZoom(view: Size, content: Size): number {
  if (content.width <= 0 || content.height <= 0 || view.width <= 0 || view.height <= 0) {
    return MIN_ZOOM;
  }
  const zoom = Math.min(
    Math.floor(view.width / content.width),
    Math.floor(view.height / content.height),
  );
  return clampZoom(zoom);
}

/** The view, the content it shows, and at what zoom and pan (editor.md, U2). */
export interface Viewport {
  readonly view: Size;
  readonly content: Size;
  readonly zoom: number;
  readonly pan: Point;
}

/** The content's top-left in client coordinates: the view centred, then panned. */
export function originFor(viewport: Viewport): Point {
  const { view, content, zoom, pan } = viewport;
  return {
    x: (view.width - content.width * zoom) / 2 + pan.x,
    y: (view.height - content.height * zoom) / 2 + pan.y,
  };
}

/** A pixel coordinate from a client point, the content's origin and the zoom (editor.md, U2). */
export function pixelFromClient(client: Point, origin: Point, zoom: number): Point {
  return {
    x: Math.floor((client.x - origin.x) / zoom),
    y: Math.floor((client.y - origin.y) / zoom),
  };
}

/** The pan that keeps the content point under `client` fixed while zooming to `nextZoom`. */
export function zoomAroundPoint(client: Point, viewport: Viewport, nextZoom: number): Point {
  const { view, content, zoom } = viewport;
  const prevOrigin = originFor(viewport);
  const contentX = (client.x - prevOrigin.x) / zoom;
  const contentY = (client.y - prevOrigin.y) / zoom;
  const nextOriginX = client.x - contentX * nextZoom;
  const nextOriginY = client.y - contentY * nextZoom;
  return {
    x: nextOriginX - (view.width - content.width * nextZoom) / 2,
    y: nextOriginY - (view.height - content.height * nextZoom) / 2,
  };
}

export function shouldShowGrid(zoom: number): boolean {
  return zoom >= GRID_MIN_ZOOM;
}

/** Alpha for a frame `stepsAway` (1-based) from the active one: halved for each step further. */
export function onionSkinAlpha(stepsAway: number): number {
  return ONION_SKIN_BASE_ALPHA / 2 ** (stepsAway - 1);
}
