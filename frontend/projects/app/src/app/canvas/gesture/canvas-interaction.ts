import { signal, type Signal } from '@angular/core';
import type { EditorTool } from '../../editor/editor-store';
import type { Area, EditOperation, Point } from '../../engine/engine-types';
import {
  buildFillOperation,
  finishGesture,
  previewOperation,
  startGesture,
  updateGesture,
  type CanvasGesture,
  type GestureContext,
  type GestureResult,
} from './canvas-gesture';

/**
 * Turns pointer and keyboard input into gestures and operations (editor.md, U2): the component
 * converts raw events to pixel coordinates and applies the operations this returns; this class
 * only holds the gesture and the keyboard cursor.
 */
export class CanvasInteraction {
  private readonly gestureSignal = signal<CanvasGesture | null>(null);
  private readonly cursorSignal = signal<Point>({ x: 0, y: 0 });
  private activePointerId: number | null = null;

  readonly gesture: Signal<CanvasGesture | null> = this.gestureSignal.asReadonly();
  readonly cursor: Signal<Point> = this.cursorSignal.asReadonly();

  /** The operation the current gesture would apply if it ended now, for `render`'s preview. */
  preview(context: GestureContext | null): EditOperation | null {
    const gesture = this.gestureSignal();
    return gesture && context ? previewOperation(gesture, context) : null;
  }

  /** A pointer press: starts a gesture, or builds the fill operation directly. */
  pointerDown(input: {
    readonly pointerId: number;
    readonly tool: EditorTool;
    readonly point: Point;
    readonly context: GestureContext;
    readonly selection: Area | null;
  }): EditOperation | null {
    const { pointerId, tool, point, context, selection } = input;
    if (tool === 'fill') return buildFillOperation(context, point);
    this.activePointerId = pointerId;
    this.gestureSignal.set(startGesture(tool, point, selection));
    return null;
  }

  pointerMove(pointerId: number, point: Point, shiftKey: boolean): void {
    if (pointerId !== this.activePointerId) return;
    const gesture = this.gestureSignal();
    if (gesture) this.gestureSignal.set(updateGesture(gesture, point, shiftKey));
  }

  /** Ends the gesture started by `pointerId`; `null` when it does not own the active gesture. */
  pointerUp(pointerId: number, context: GestureContext): GestureResult | null {
    if (pointerId !== this.activePointerId) return null;
    return this.finish(context);
  }

  /** Escape, or a lost pointer: drops the gesture without applying anything. */
  cancel(): void {
    this.activePointerId = null;
    this.gestureSignal.set(null);
  }

  /** An arrow key: moves the cursor within `bounds`, extending a pending two-press gesture. */
  moveCursor(delta: Point, bounds: { readonly width: number; readonly height: number }): Point {
    const cursor = this.cursorSignal();
    const next = {
      x: Math.min(Math.max(cursor.x + delta.x, 0), bounds.width - 1),
      y: Math.min(Math.max(cursor.y + delta.y, 0), bounds.height - 1),
    };
    this.cursorSignal.set(next);
    const gesture = this.gestureSignal();
    if (gesture) this.gestureSignal.set(updateGesture(gesture, next, false));
    return next;
  }

  /**
   * Enter or Space at the keyboard cursor: pencil, eraser and fill apply immediately; line,
   * rectangle and select take a first press for the start and this same call again for the end
   * (editor.md, U2). The select tool always draws a new selection from the keyboard.
   */
  keyboardActivate(tool: EditorTool, context: GestureContext): GestureResult | null {
    if (this.gestureSignal()) return this.finish(context);
    if (tool === 'fill') return { operation: buildFillOperation(context, this.cursorSignal()) };
    const gesture = startGesture(tool, this.cursorSignal(), null);
    if (!gesture) return null;
    if (gesture.kind !== 'stroke') {
      this.gestureSignal.set(gesture);
      return null;
    }
    return finishGesture(gesture, context);
  }

  private finish(context: GestureContext): GestureResult {
    const gesture = this.gestureSignal();
    this.activePointerId = null;
    this.gestureSignal.set(null);
    if (!gesture) return { operation: null };
    return finishGesture(gesture, context);
  }
}
