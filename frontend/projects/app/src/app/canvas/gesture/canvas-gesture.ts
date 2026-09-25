import type { EditorTool } from '../../editor/editor-store';
import type { Area, EditOperation, FrameId, LayerId, Point } from '../../engine/engine-types';

/** The active layer, frame and drawing options a gesture needs to build its operation. */
export interface GestureContext {
  readonly layer: LayerId;
  readonly frame: FrameId;
  readonly colorIndex: number;
  readonly rectangleFilled: boolean;
}

/** One in-progress gesture (editor.md, U2): nothing is applied until it ends. */
export type CanvasGesture =
  | {
      readonly kind: 'stroke';
      readonly tool: 'pencil' | 'eraser';
      readonly points: readonly Point[];
    }
  | { readonly kind: 'line'; readonly start: Point; readonly end: Point }
  | { readonly kind: 'rectangle'; readonly start: Point; readonly end: Point }
  | {
      readonly kind: 'moveSelection';
      readonly area: Area;
      readonly start: Point;
      readonly current: Point;
    }
  | { readonly kind: 'select'; readonly start: Point; readonly current: Point };

export interface GestureResult {
  readonly operation: EditOperation | null;
  /** Set when the gesture also changes the local selection (editor.md, U2's select tool). */
  readonly selection?: Area | null;
}

export function containsPoint(area: Area, point: Point): boolean {
  return (
    point.x >= area.x &&
    point.x < area.x + area.width &&
    point.y >= area.y &&
    point.y < area.y + area.height
  );
}

function normalizeArea(start: Point, current: Point): Area {
  return {
    x: Math.min(start.x, current.x),
    y: Math.min(start.y, current.y),
    width: Math.abs(current.x - start.x) + 1,
    height: Math.abs(current.y - start.y) + 1,
  };
}

/** Shift snaps a line to a multiple of 45°, the longer axis keeping its length. */
function constrainLine(start: Point, point: Point, shiftKey: boolean): Point {
  if (!shiftKey) return point;
  const dx = point.x - start.x;
  const dy = point.y - start.y;
  const distance = Math.max(Math.abs(dx), Math.abs(dy));
  const stepX = Math.sign(dx);
  const stepY = Math.sign(dy);
  if (Math.abs(dx) > Math.abs(dy) * 2) return { x: start.x + distance * stepX, y: start.y };
  if (Math.abs(dy) > Math.abs(dx) * 2) return { x: start.x, y: start.y + distance * stepY };
  return { x: start.x + distance * stepX, y: start.y + distance * stepY };
}

/** Shift forces the rectangle to a square, the longer side driving the size. */
function constrainRectangle(start: Point, point: Point, shiftKey: boolean): Point {
  if (!shiftKey) return point;
  const dx = point.x - start.x;
  const dy = point.y - start.y;
  const side = Math.max(Math.abs(dx), Math.abs(dy));
  return { x: start.x + side * (Math.sign(dx) || 1), y: start.y + side * (Math.sign(dy) || 1) };
}

/**
 * Starts a gesture for `tool` at `point`; `null` for the fill tool, applied immediately instead
 * (`buildFillOperation`). A `select` starting inside `selection` moves it; elsewhere it draws a
 * new one. Pass `selection: null` (the keyboard's two-press mode) to always draw a new one.
 */
export function startGesture(
  tool: EditorTool,
  point: Point,
  selection: Area | null,
): CanvasGesture | null {
  switch (tool) {
    case 'pencil':
    case 'eraser':
      return { kind: 'stroke', tool, points: [point] };
    case 'line':
      return { kind: 'line', start: point, end: point };
    case 'rectangle':
      return { kind: 'rectangle', start: point, end: point };
    case 'select':
      if (selection && containsPoint(selection, point)) {
        return { kind: 'moveSelection', area: selection, start: point, current: point };
      }
      return { kind: 'select', start: point, current: point };
    case 'fill':
      return null;
  }
}

/** Extends a gesture towards `point`; a duplicate point is dropped from a stroke. */
export function updateGesture(
  gesture: CanvasGesture,
  point: Point,
  shiftKey: boolean,
): CanvasGesture {
  switch (gesture.kind) {
    case 'stroke': {
      const last = gesture.points.at(-1);
      if (last && last.x === point.x && last.y === point.y) return gesture;
      return { ...gesture, points: [...gesture.points, point] };
    }
    case 'line':
      return { ...gesture, end: constrainLine(gesture.start, point, shiftKey) };
    case 'rectangle':
      return { ...gesture, end: constrainRectangle(gesture.start, point, shiftKey) };
    case 'moveSelection':
      return { ...gesture, current: point };
    case 'select':
      return { ...gesture, current: point };
  }
}

function buildStrokeOperation(
  gesture: Extract<CanvasGesture, { kind: 'stroke' }>,
  context: GestureContext,
): EditOperation | null {
  if (gesture.points.length === 0) return null;
  return {
    kind: 'paintStroke',
    layer: context.layer,
    frame: context.frame,
    points: gesture.points,
    index: gesture.tool === 'eraser' ? 0 : context.colorIndex,
  };
}

function buildLineOperation(
  gesture: Extract<CanvasGesture, { kind: 'line' }>,
  context: GestureContext,
): EditOperation {
  return {
    kind: 'line',
    layer: context.layer,
    frame: context.frame,
    from: gesture.start,
    to: gesture.end,
    index: context.colorIndex,
  };
}

function buildRectangleOperation(
  gesture: Extract<CanvasGesture, { kind: 'rectangle' }>,
  context: GestureContext,
): EditOperation {
  return {
    kind: 'rectangle',
    layer: context.layer,
    frame: context.frame,
    from: gesture.start,
    to: gesture.end,
    index: context.colorIndex,
    filled: context.rectangleFilled,
  };
}

function selectionOffset(gesture: Extract<CanvasGesture, { kind: 'moveSelection' }>): Point {
  return { x: gesture.current.x - gesture.start.x, y: gesture.current.y - gesture.start.y };
}

function buildMoveSelectionOperationFor(
  gesture: Extract<CanvasGesture, { kind: 'moveSelection' }>,
  context: GestureContext,
): EditOperation {
  return buildMoveSelectionOperation(context, gesture.area, selectionOffset(gesture));
}

/** The operation a gesture would apply if it ended now, sent to `render` as a live preview. */
export function previewOperation(
  gesture: CanvasGesture,
  context: GestureContext,
): EditOperation | null {
  switch (gesture.kind) {
    case 'stroke':
      return buildStrokeOperation(gesture, context);
    case 'line':
      return buildLineOperation(gesture, context);
    case 'rectangle':
      return buildRectangleOperation(gesture, context);
    case 'moveSelection':
      return buildMoveSelectionOperationFor(gesture, context);
    case 'select':
      return null;
  }
}

function finishMoveSelection(
  gesture: Extract<CanvasGesture, { kind: 'moveSelection' }>,
  context: GestureContext,
): GestureResult {
  const offset = selectionOffset(gesture);
  if (offset.x === 0 && offset.y === 0) return { operation: null };
  const { area } = gesture;
  return {
    operation: buildMoveSelectionOperationFor(gesture, context),
    selection: {
      x: area.x + offset.x,
      y: area.y + offset.y,
      width: area.width,
      height: area.height,
    },
  };
}

/** The operation and, for the select tool, the selection change a finished gesture produces. */
export function finishGesture(gesture: CanvasGesture, context: GestureContext): GestureResult {
  switch (gesture.kind) {
    case 'stroke':
      return { operation: buildStrokeOperation(gesture, context) };
    case 'line':
      return { operation: buildLineOperation(gesture, context) };
    case 'rectangle':
      return { operation: buildRectangleOperation(gesture, context) };
    case 'moveSelection':
      return finishMoveSelection(gesture, context);
    case 'select':
      return { operation: null, selection: normalizeArea(gesture.start, gesture.current) };
  }
}

export function buildFillOperation(context: GestureContext, at: Point): EditOperation {
  return {
    kind: 'fill',
    layer: context.layer,
    frame: context.frame,
    at,
    index: context.colorIndex,
  };
}

export function buildMoveSelectionOperation(
  context: GestureContext,
  area: Area,
  offset: Point,
): EditOperation {
  return { kind: 'moveSelection', layer: context.layer, frame: context.frame, area, offset };
}
