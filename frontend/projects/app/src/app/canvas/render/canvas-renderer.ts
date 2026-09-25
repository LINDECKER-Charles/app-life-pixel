import type { Point, RenderedFrame } from '../../engine/engine-types';
import type { Size } from '../geometry/canvas-geometry';
import { onionSkinAlpha } from '../geometry/canvas-geometry';

const CHECKER_CELL = 8;
const CHECKER_LIGHT = '#ffffff';
const CHECKER_DARK = '#cccccc';
const GRID_COLOR = 'rgba(0, 0, 0, 0.25)';

export interface OnionSkinFrame {
  readonly image: RenderedFrame;
  readonly stepsAway: number;
}

/** Where and how large the content is drawn: the view's origin at this zoom. */
interface Layout {
  readonly origin: Point;
  readonly zoom: number;
  readonly content: Size;
}

/** Everything a single draw needs: the active frame, its onion-skin neighbours, and the view. */
export interface Scene {
  readonly view: Size;
  readonly layout: Layout;
  readonly frame: RenderedFrame | null;
  readonly before: readonly OnionSkinFrame[];
  readonly after: readonly OnionSkinFrame[];
  readonly showGrid: boolean;
}

/** A `RenderedFrame`'s pixels, put onto a canvas of their own so they can be scaled and drawn. */
export function imageToCanvas(image: RenderedFrame): HTMLCanvasElement | null {
  const canvas = document.createElement('canvas');
  canvas.width = image.width;
  canvas.height = image.height;
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  const pixels = new Uint8ClampedArray(image.pixels);
  ctx.putImageData(new ImageData(pixels, image.width, image.height), 0, 0);
  return canvas;
}

function drawCheckerboard(ctx: CanvasRenderingContext2D, layout: Layout): void {
  const width = layout.content.width * layout.zoom;
  const height = layout.content.height * layout.zoom;
  ctx.save();
  ctx.translate(layout.origin.x, layout.origin.y);
  ctx.fillStyle = CHECKER_LIGHT;
  ctx.fillRect(0, 0, width, height);
  ctx.fillStyle = CHECKER_DARK;
  for (let y = 0; y < height; y += CHECKER_CELL) {
    for (let x = 0; x < width; x += CHECKER_CELL) {
      if ((Math.floor(x / CHECKER_CELL) + Math.floor(y / CHECKER_CELL)) % 2 === 1) {
        ctx.fillRect(x, y, CHECKER_CELL, CHECKER_CELL);
      }
    }
  }
  ctx.restore();
}

/** Draws a pixel-content canvas scaled to the layout's zoom, with no smoothing (editor.md, U2). */
function drawScaled(
  ctx: CanvasRenderingContext2D,
  source: HTMLCanvasElement,
  options: { readonly layout: Layout; readonly alpha: number },
): void {
  const { layout, alpha } = options;
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    source,
    0,
    0,
    layout.content.width,
    layout.content.height,
    layout.origin.x,
    layout.origin.y,
    layout.content.width * layout.zoom,
    layout.content.height * layout.zoom,
  );
  ctx.restore();
}

function drawGrid(ctx: CanvasRenderingContext2D, layout: Layout): void {
  const { origin, zoom, content } = layout;
  ctx.save();
  ctx.strokeStyle = GRID_COLOR;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = 0; x <= content.width; x++) {
    const px = Math.round(origin.x + x * zoom) + 0.5;
    ctx.moveTo(px, origin.y);
    ctx.lineTo(px, origin.y + content.height * zoom);
  }
  for (let y = 0; y <= content.height; y++) {
    const py = Math.round(origin.y + y * zoom) + 0.5;
    ctx.moveTo(origin.x, py);
    ctx.lineTo(origin.x + content.width * zoom, py);
  }
  ctx.stroke();
  ctx.restore();
}

function drawOnionSkin(
  ctx: CanvasRenderingContext2D,
  frames: readonly OnionSkinFrame[],
  layout: Layout,
): void {
  for (const skin of frames) {
    const canvas = imageToCanvas(skin.image);
    if (canvas) drawScaled(ctx, canvas, { layout, alpha: onionSkinAlpha(skin.stepsAway) });
  }
}

/** The whole visible canvas for one animation frame: checkerboard, onion skin, frame, grid. */
export function drawScene(ctx: CanvasRenderingContext2D, scene: Scene): void {
  ctx.clearRect(0, 0, scene.view.width, scene.view.height);
  drawCheckerboard(ctx, scene.layout);
  drawOnionSkin(ctx, scene.before, scene.layout);
  drawOnionSkin(ctx, scene.after, scene.layout);
  const active = scene.frame ? imageToCanvas(scene.frame) : null;
  if (active) drawScaled(ctx, active, { layout: scene.layout, alpha: 1 });
  if (scene.showGrid) drawGrid(ctx, scene.layout);
}
