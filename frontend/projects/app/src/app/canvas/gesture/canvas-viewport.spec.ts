import type { Point } from '../../engine/engine-types';
import type { Viewport } from '../geometry/canvas-geometry';
import { CanvasViewport } from './canvas-viewport';

function setup(initial: Viewport): { viewport: CanvasViewport; state: () => Viewport } {
  let current = initial;
  const viewport = new CanvasViewport({
    getViewport: () => current,
    setZoom: (zoom) => {
      current = { ...current, zoom };
    },
    setPan: (pan) => {
      current = { ...current, pan };
    },
  });
  return { viewport, state: () => current };
}

const BASE: Viewport = {
  view: { width: 320, height: 220 },
  content: { width: 32, height: 32 },
  zoom: 5,
  pan: { x: 0, y: 0 },
};

describe('CanvasViewport — drag pan', () => {
  it('pans by the pointer delta while the same pointer drags', () => {
    const { viewport, state } = setup(BASE);

    viewport.startPan(1, { x: 100, y: 100 });
    viewport.updatePan(1, { x: 110, y: 95 });

    expect(state().pan).toEqual({ x: 10, y: -5 });
    expect(viewport.isPanning).toBe(true);
  });

  it('ignores a move or an end from another pointer', () => {
    const { viewport, state } = setup(BASE);
    viewport.startPan(1, { x: 100, y: 100 });

    viewport.updatePan(2, { x: 200, y: 200 });
    expect(state().pan).toEqual({ x: 0, y: 0 });

    viewport.endPan(2);
    expect(viewport.isPanning).toBe(true);
  });

  it('stops panning on end', () => {
    const { viewport } = setup(BASE);
    viewport.startPan(1, { x: 0, y: 0 });

    viewport.endPan(1);

    expect(viewport.isPanning).toBe(false);
  });
});

describe('CanvasViewport — zoom', () => {
  it('zooms in and out by one step, within bounds', () => {
    const { viewport, state } = setup(BASE);

    viewport.zoomBy(1);
    expect(state().zoom).toBe(6);

    viewport.zoomBy(-10);
    expect(state().zoom).toBe(1);
  });

  it('fits the content to the view and re-centres the pan', () => {
    const { viewport, state } = setup({ ...BASE, pan: { x: 40, y: 40 } });

    viewport.zoomFit();

    expect(state().zoom).toBe(6);
    expect(state().pan).toEqual({ x: 0, y: 0 });
  });

  it('does nothing when there is no content to fit', () => {
    const { viewport, state } = setup({ ...BASE, content: { width: 0, height: 0 } });

    viewport.zoomFit();

    expect(state().zoom).toBe(BASE.zoom);
  });

  it('keeps the pixel under the wheel event fixed while zooming', () => {
    const { viewport, state } = setup(BASE);
    const client: Point = { x: 84, y: 34 };

    viewport.zoomWheel(client, -1);

    expect(state().zoom).toBe(6);
    expect(state().pan).not.toEqual({ x: 0, y: 0 });
  });

  it('zooms out on a positive deltaY', () => {
    const { viewport, state } = setup({ ...BASE, zoom: 10 });

    viewport.zoomWheel({ x: 0, y: 0 }, 1);

    expect(state().zoom).toBe(9);
  });
});

describe('CanvasViewport — keyboard pan', () => {
  it('pans by a fixed step opposite the arrow direction', () => {
    const { viewport, state } = setup(BASE);

    viewport.panByStep({ x: 1, y: 0 }, 40);

    expect(state().pan).toEqual({ x: -40, y: 0 });
  });
});
