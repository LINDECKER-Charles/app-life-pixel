import {
  buildFillOperation,
  buildMoveSelectionOperation,
  containsPoint,
  finishGesture,
  previewOperation,
  startGesture,
  updateGesture,
  type CanvasGesture,
  type GestureContext,
} from './canvas-gesture';

const CONTEXT: GestureContext = { layer: 1, frame: 2, colorIndex: 3, rectangleFilled: false };

/** `startGesture` is null only for the fill tool, which every call here avoids. */
function requireGesture(gesture: CanvasGesture | null): CanvasGesture {
  if (!gesture) throw new Error('expected a gesture');
  return gesture;
}

describe('containsPoint', () => {
  it('is true on the area, false past its far edge', () => {
    const area = { x: 2, y: 2, width: 3, height: 3 };
    expect(containsPoint(area, { x: 2, y: 2 })).toBe(true);
    expect(containsPoint(area, { x: 4, y: 4 })).toBe(true);
    expect(containsPoint(area, { x: 5, y: 4 })).toBe(false);
    expect(containsPoint(area, { x: 1, y: 4 })).toBe(false);
  });
});

describe('pencil and eraser', () => {
  it('previews the dragged points, then applies one paintStroke on finish', () => {
    let gesture = requireGesture(startGesture('pencil', { x: 0, y: 0 }, null));
    expect(previewOperation(gesture, CONTEXT)).toEqual({
      kind: 'paintStroke',
      layer: 1,
      frame: 2,
      points: [{ x: 0, y: 0 }],
      index: 3,
    });

    gesture = updateGesture(gesture, { x: 1, y: 0 }, false);
    gesture = updateGesture(gesture, { x: 2, y: 0 }, false);
    expect(previewOperation(gesture, CONTEXT)).toMatchObject({
      points: [
        { x: 0, y: 0 },
        { x: 1, y: 0 },
        { x: 2, y: 0 },
      ],
    });

    const result = finishGesture(gesture, CONTEXT);
    expect(result).toEqual({
      operation: {
        kind: 'paintStroke',
        layer: 1,
        frame: 2,
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 0 },
          { x: 2, y: 0 },
        ],
        index: 3,
      },
    });
  });

  it('drops a repeated point', () => {
    let gesture = requireGesture(startGesture('pencil', { x: 0, y: 0 }, null));
    gesture = updateGesture(gesture, { x: 0, y: 0 }, false);
    expect(gesture).toMatchObject({ points: [{ x: 0, y: 0 }] });
  });

  it('paints with colour index 0 for the eraser', () => {
    const gesture = requireGesture(startGesture('eraser', { x: 0, y: 0 }, null));
    const result = finishGesture(gesture, CONTEXT);
    expect(result.operation).toMatchObject({ kind: 'paintStroke', index: 0 });
  });
});

describe('line and rectangle', () => {
  it('sends a line operation from the start to the finish point', () => {
    let gesture = requireGesture(startGesture('line', { x: 0, y: 0 }, null));
    gesture = updateGesture(gesture, { x: 4, y: 1 }, false);
    expect(finishGesture(gesture, CONTEXT).operation).toEqual({
      kind: 'line',
      layer: 1,
      frame: 2,
      from: { x: 0, y: 0 },
      to: { x: 4, y: 1 },
      index: 3,
    });
  });

  it('constrains a line to 45° with Shift', () => {
    let gesture = requireGesture(startGesture('line', { x: 0, y: 0 }, null));
    gesture = updateGesture(gesture, { x: 6, y: 1 }, true);
    expect(finishGesture(gesture, CONTEXT).operation).toMatchObject({ to: { x: 6, y: 0 } });
  });

  it('sends a rectangle operation with the filled option', () => {
    let gesture = requireGesture(startGesture('rectangle', { x: 1, y: 1 }, null));
    gesture = updateGesture(gesture, { x: 3, y: 5 }, false);
    expect(finishGesture(gesture, { ...CONTEXT, rectangleFilled: true }).operation).toEqual({
      kind: 'rectangle',
      layer: 1,
      frame: 2,
      from: { x: 1, y: 1 },
      to: { x: 3, y: 5 },
      index: 3,
      filled: true,
    });
  });

  it('constrains a rectangle to a square with Shift', () => {
    let gesture = requireGesture(startGesture('rectangle', { x: 0, y: 0 }, null));
    gesture = updateGesture(gesture, { x: 6, y: 2 }, true);
    expect(finishGesture(gesture, CONTEXT).operation).toMatchObject({ to: { x: 6, y: 6 } });
  });
});

describe('fill', () => {
  it('builds a fill operation directly, with no gesture', () => {
    expect(startGesture('fill', { x: 2, y: 2 }, null)).toBeNull();
    expect(buildFillOperation(CONTEXT, { x: 2, y: 2 })).toEqual({
      kind: 'fill',
      layer: 1,
      frame: 2,
      at: { x: 2, y: 2 },
      index: 3,
    });
  });
});

describe('select', () => {
  it('draws a new selection when starting outside the current one', () => {
    let gesture = requireGesture(startGesture('select', { x: 0, y: 0 }, null));
    expect(gesture.kind).toBe('select');
    gesture = updateGesture(gesture, { x: 2, y: 3 }, false);
    const result = finishGesture(gesture, CONTEXT);
    expect(result).toEqual({
      operation: null,
      selection: { x: 0, y: 0, width: 3, height: 4 },
    });
  });

  it('normalises a selection dragged up and to the left', () => {
    let gesture = requireGesture(startGesture('select', { x: 4, y: 4 }, null));
    gesture = updateGesture(gesture, { x: 1, y: 2 }, false);
    expect(finishGesture(gesture, CONTEXT).selection).toEqual({ x: 1, y: 2, width: 4, height: 3 });
  });

  it('moves the selection when the drag starts inside it, previewing then applying', () => {
    const selection = { x: 1, y: 1, width: 2, height: 2 };
    let gesture = requireGesture(startGesture('select', { x: 1, y: 1 }, selection));
    expect(gesture.kind).toBe('moveSelection');
    expect(previewOperation(gesture, CONTEXT)).toEqual({
      kind: 'moveSelection',
      layer: 1,
      frame: 2,
      area: selection,
      offset: { x: 0, y: 0 },
    });

    gesture = updateGesture(gesture, { x: 3, y: 2 }, false);
    expect(previewOperation(gesture, CONTEXT)).toMatchObject({ offset: { x: 2, y: 1 } });

    const result = finishGesture(gesture, CONTEXT);
    expect(result).toEqual({
      operation: {
        kind: 'moveSelection',
        layer: 1,
        frame: 2,
        area: selection,
        offset: { x: 2, y: 1 },
      },
      selection: { x: 3, y: 2, width: 2, height: 2 },
    });
  });

  it('applies nothing when the selection is picked up and dropped in place', () => {
    const selection = { x: 1, y: 1, width: 2, height: 2 };
    const gesture = requireGesture(startGesture('select', { x: 1, y: 1 }, selection));
    expect(finishGesture(gesture, CONTEXT)).toEqual({ operation: null });
  });
});

describe('buildMoveSelectionOperation', () => {
  it('builds the operation from the active layer, frame, area and offset', () => {
    const area = { x: 0, y: 0, width: 2, height: 2 };
    expect(buildMoveSelectionOperation(CONTEXT, area, { x: -1, y: 0 })).toEqual({
      kind: 'moveSelection',
      layer: 1,
      frame: 2,
      area,
      offset: { x: -1, y: 0 },
    });
  });
});
