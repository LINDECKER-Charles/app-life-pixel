import type { GestureContext } from './canvas-gesture';
import { CanvasInteraction } from './canvas-interaction';

const CONTEXT: GestureContext = { layer: 1, frame: 2, colorIndex: 3, rectangleFilled: false };
const BOUNDS = { width: 8, height: 8 };

describe('CanvasInteraction — pointer gestures', () => {
  it('previews a pencil drag as it moves, then applies one paintStroke on release', () => {
    const interaction = new CanvasInteraction();

    const onDown = interaction.pointerDown({
      pointerId: 1,
      tool: 'pencil',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: null,
    });
    expect(onDown).toBeNull();
    expect(interaction.preview(CONTEXT)).toMatchObject({ points: [{ x: 0, y: 0 }] });

    interaction.pointerMove(1, { x: 1, y: 0 }, false);
    expect(interaction.preview(CONTEXT)).toMatchObject({
      points: [
        { x: 0, y: 0 },
        { x: 1, y: 0 },
      ],
    });

    const result = interaction.pointerUp(1, CONTEXT);
    expect(result).toEqual({
      operation: {
        kind: 'paintStroke',
        layer: 1,
        frame: 2,
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 0 },
        ],
        index: 3,
      },
    });
    expect(interaction.gesture()).toBeNull();
  });

  it('sends the fill operation directly on pointer down, starting no gesture', () => {
    const interaction = new CanvasInteraction();

    const operation = interaction.pointerDown({
      pointerId: 1,
      tool: 'fill',
      point: { x: 2, y: 2 },
      context: CONTEXT,
      selection: null,
    });

    expect(operation).toEqual({ kind: 'fill', layer: 1, frame: 2, at: { x: 2, y: 2 }, index: 3 });
    expect(interaction.gesture()).toBeNull();
  });

  it('sends a line operation on release', () => {
    const interaction = new CanvasInteraction();
    interaction.pointerDown({
      pointerId: 1,
      tool: 'line',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: null,
    });
    interaction.pointerMove(1, { x: 3, y: 0 }, false);

    expect(interaction.pointerUp(1, CONTEXT)?.operation).toEqual({
      kind: 'line',
      layer: 1,
      frame: 2,
      from: { x: 0, y: 0 },
      to: { x: 3, y: 0 },
      index: 3,
    });
  });

  it('sends a rectangle operation with the filled option on release', () => {
    const interaction = new CanvasInteraction();
    interaction.pointerDown({
      pointerId: 1,
      tool: 'rectangle',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: null,
    });
    interaction.pointerMove(1, { x: 2, y: 2 }, false);

    expect(
      interaction.pointerUp(1, { ...CONTEXT, rectangleFilled: true })?.operation,
    ).toMatchObject({
      kind: 'rectangle',
      filled: true,
    });
  });

  it('sends a moveSelection operation when dragging inside the selection', () => {
    const interaction = new CanvasInteraction();
    const selection = { x: 0, y: 0, width: 2, height: 2 };
    interaction.pointerDown({
      pointerId: 1,
      tool: 'select',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: selection,
    });

    interaction.pointerMove(1, { x: 1, y: 0 }, false);
    const result = interaction.pointerUp(1, CONTEXT);

    expect(result).toEqual({
      operation: {
        kind: 'moveSelection',
        layer: 1,
        frame: 2,
        area: selection,
        offset: { x: 1, y: 0 },
      },
      selection: { x: 1, y: 0, width: 2, height: 2 },
    });
  });

  it('sends nothing on Escape', () => {
    const interaction = new CanvasInteraction();
    interaction.pointerDown({
      pointerId: 1,
      tool: 'pencil',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: null,
    });
    interaction.pointerMove(1, { x: 1, y: 0 }, false);

    interaction.cancel();

    expect(interaction.gesture()).toBeNull();
    expect(interaction.pointerUp(1, CONTEXT)).toBeNull();
  });

  it('ignores a move or release from a pointer that did not start the gesture', () => {
    const interaction = new CanvasInteraction();
    interaction.pointerDown({
      pointerId: 1,
      tool: 'pencil',
      point: { x: 0, y: 0 },
      context: CONTEXT,
      selection: null,
    });

    interaction.pointerMove(2, { x: 5, y: 5 }, false);
    expect(interaction.preview(CONTEXT)).toMatchObject({ points: [{ x: 0, y: 0 }] });
    expect(interaction.pointerUp(2, CONTEXT)).toBeNull();
  });
});

describe('CanvasInteraction — keyboard', () => {
  it('draws a single-point stroke immediately with the pencil', () => {
    const interaction = new CanvasInteraction();

    const result = interaction.keyboardActivate('pencil', CONTEXT);

    expect(result).toEqual({
      operation: {
        kind: 'paintStroke',
        layer: 1,
        frame: 2,
        points: [{ x: 0, y: 0 }],
        index: 3,
      },
    });
    expect(interaction.gesture()).toBeNull();
  });

  it('fills immediately at the cursor', () => {
    const interaction = new CanvasInteraction();
    interaction.moveCursor({ x: 1, y: 1 }, BOUNDS);

    expect(interaction.keyboardActivate('fill', CONTEXT)).toEqual({
      operation: { kind: 'fill', layer: 1, frame: 2, at: { x: 1, y: 1 }, index: 3 },
    });
  });

  it('takes a first press for the start of a line and a second for the end', () => {
    const interaction = new CanvasInteraction();

    expect(interaction.keyboardActivate('line', CONTEXT)).toBeNull();
    expect(interaction.gesture()).not.toBeNull();

    interaction.moveCursor({ x: 1, y: 0 }, BOUNDS);
    interaction.moveCursor({ x: 1, y: 0 }, BOUNDS);
    const result = interaction.keyboardActivate('line', CONTEXT);

    expect(result?.operation).toEqual({
      kind: 'line',
      layer: 1,
      frame: 2,
      from: { x: 0, y: 0 },
      to: { x: 2, y: 0 },
      index: 3,
    });
    expect(interaction.gesture()).toBeNull();
  });

  it('always draws a new selection from the keyboard, even inside the current one', () => {
    const interaction = new CanvasInteraction();

    interaction.keyboardActivate('select', CONTEXT);
    interaction.moveCursor({ x: 2, y: 1 }, BOUNDS);
    const result = interaction.keyboardActivate('select', CONTEXT);

    expect(result).toEqual({ operation: null, selection: { x: 0, y: 0, width: 3, height: 2 } });
  });

  it('clamps the cursor to the document bounds', () => {
    const interaction = new CanvasInteraction();

    interaction.moveCursor({ x: -5, y: -5 }, BOUNDS);
    expect(interaction.cursor()).toEqual({ x: 0, y: 0 });

    interaction.moveCursor({ x: 99, y: 99 }, BOUNDS);
    expect(interaction.cursor()).toEqual({ x: 7, y: 7 });
  });
});
