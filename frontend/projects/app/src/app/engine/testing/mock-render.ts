import type { DocumentSummary, EditOperation, Point, RenderedFrame } from '../engine-types';
import { frameNotFound } from './mock-errors';
import { findFrameIndex, strokeKey, type MockDocument } from './mock-state';

type Rgba = readonly [number, number, number, number];

function parseColor(color: string): Rgba {
  const hex = color.slice(1);
  const byte = (start: number): number => parseInt(hex.slice(start, start + 2), 16);
  return [byte(0), byte(2), byte(4), byte(6)];
}

interface Canvas {
  readonly pixels: Uint8ClampedArray;
  readonly width: number;
  readonly height: number;
}

function fillBackground(canvas: Canvas, color: Rgba): void {
  for (let offset = 0; offset < canvas.pixels.length; offset += 4) {
    canvas.pixels.set(color, offset);
  }
}

function paintPoints(canvas: Canvas, points: readonly Point[], color: Rgba): void {
  for (const point of points) {
    if (point.x < 0 || point.x >= canvas.width || point.y < 0 || point.y >= canvas.height) continue;
    canvas.pixels.set(color, (point.y * canvas.width + point.x) * 4);
  }
}

/** Only `paintStroke` carries points the mock can preview (editor.md, W0). */
function previewPoints(
  preview: EditOperation | undefined,
  layer: number,
  frame: number,
): readonly Point[] {
  if (!preview || preview.kind !== 'paintStroke') return [];
  return preview.layer === layer && preview.frame === frame ? preview.points : [];
}

/**
 * A minimal composite: transparent background, then each visible layer's last stroke on this
 * frame, bottom to top — enough pixel behaviour for the interface tasks (U2) to build against,
 * until `editor-wasm` renders the real thing (W1).
 */
export function renderFrame(
  document: MockDocument,
  frame: number,
  preview?: EditOperation,
): RenderedFrame {
  const summary: DocumentSummary = document.summary;
  if (findFrameIndex(summary, frame) < 0) throw frameNotFound(frame);
  const { width, height, palette } = summary;
  const canvas: Canvas = { pixels: new Uint8ClampedArray(width * height * 4), width, height };
  fillBackground(canvas, parseColor(palette[0]));
  for (const layer of summary.layers) {
    if (!layer.visible) continue;
    const stroke = document.strokes.get(strokeKey(layer.id, frame));
    if (stroke) paintPoints(canvas, stroke.points, parseColor(palette[stroke.index] ?? palette[0]));
    const previewed = previewPoints(preview, layer.id, frame);
    const previewIndex = preview && preview.kind === 'paintStroke' ? preview.index : 0;
    if (previewed.length > 0) paintPoints(canvas, previewed, parseColor(palette[previewIndex]));
  }
  return { width, height, pixels: canvas.pixels };
}
