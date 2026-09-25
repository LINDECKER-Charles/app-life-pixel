import {
  clampZoom,
  fitZoom,
  GRID_MIN_ZOOM,
  onionSkinAlpha,
  originFor,
  pixelFromClient,
  shouldShowGrid,
  zoomAroundPoint,
} from './canvas-geometry';

describe('clampZoom', () => {
  it('keeps a zoom within 1 and 64', () => {
    expect(clampZoom(0)).toBe(1);
    expect(clampZoom(1)).toBe(1);
    expect(clampZoom(64)).toBe(64);
    expect(clampZoom(65)).toBe(64);
    expect(clampZoom(12.4)).toBe(12);
  });
});

describe('fitZoom', () => {
  it('picks the largest integer zoom that fits the content in the view', () => {
    expect(fitZoom({ width: 320, height: 200 }, { width: 32, height: 32 })).toBe(6);
    expect(fitZoom({ width: 100, height: 400 }, { width: 32, height: 32 })).toBe(3);
  });

  it('never returns less than 1, even for a content larger than the view', () => {
    expect(fitZoom({ width: 10, height: 10 }, { width: 32, height: 32 })).toBe(1);
  });

  it('returns 1 for a degenerate size', () => {
    expect(fitZoom({ width: 0, height: 200 }, { width: 32, height: 32 })).toBe(1);
    expect(fitZoom({ width: 320, height: 200 }, { width: 0, height: 32 })).toBe(1);
  });
});

const VIEW = { width: 320, height: 220 };
const CONTENT = { width: 32, height: 32 };

describe('originFor', () => {
  it('centres the content in the view, then applies the pan', () => {
    const origin = originFor({ view: VIEW, content: CONTENT, zoom: 5, pan: { x: 0, y: 0 } });
    expect(origin).toEqual({ x: 80, y: 30 });
  });

  it('shifts by the pan on top of the centring', () => {
    const origin = originFor({ view: VIEW, content: CONTENT, zoom: 5, pan: { x: 10, y: -5 } });
    expect(origin).toEqual({ x: 90, y: 25 });
  });
});

describe('pixelFromClient', () => {
  it('floors the client point minus the origin, divided by the zoom', () => {
    expect(pixelFromClient({ x: 84, y: 34 }, { x: 80, y: 30 }, 5)).toEqual({ x: 0, y: 0 });
    expect(pixelFromClient({ x: 95, y: 44 }, { x: 80, y: 30 }, 5)).toEqual({ x: 3, y: 2 });
  });

  it('handles points left of or above the origin', () => {
    expect(pixelFromClient({ x: 70, y: 20 }, { x: 80, y: 30 }, 5)).toEqual({ x: -2, y: -2 });
  });
});

describe('zoomAroundPoint', () => {
  it('keeps the content pixel under the client point fixed while zooming in', () => {
    const viewport = { view: VIEW, content: CONTENT, zoom: 5, pan: { x: 0, y: 0 } };
    const client = { x: 84, y: 34 };
    const pixelBefore = pixelFromClient(client, originFor(viewport), viewport.zoom);

    const nextPan = zoomAroundPoint(client, viewport, 10);
    const nextOrigin = originFor({ ...viewport, zoom: 10, pan: nextPan });
    const pixelAfter = pixelFromClient(client, nextOrigin, 10);

    expect(pixelAfter).toEqual(pixelBefore);
  });

  it('keeps the content pixel under the client point fixed while zooming out', () => {
    const viewport = { view: VIEW, content: CONTENT, zoom: 10, pan: { x: 12, y: -8 } };
    const client = { x: 150, y: 90 };
    const pixelBefore = pixelFromClient(client, originFor(viewport), viewport.zoom);

    const nextPan = zoomAroundPoint(client, viewport, 4);
    const nextOrigin = originFor({ ...viewport, zoom: 4, pan: nextPan });
    const pixelAfter = pixelFromClient(client, nextOrigin, 4);

    expect(pixelAfter).toEqual(pixelBefore);
  });
});

describe('shouldShowGrid', () => {
  it('shows the grid from zoom 8', () => {
    expect(shouldShowGrid(GRID_MIN_ZOOM - 1)).toBe(false);
    expect(shouldShowGrid(GRID_MIN_ZOOM)).toBe(true);
    expect(shouldShowGrid(GRID_MIN_ZOOM + 10)).toBe(true);
  });
});

describe('onionSkinAlpha', () => {
  it('is 0.3 for the nearest frame, halved for each step further', () => {
    expect(onionSkinAlpha(1)).toBeCloseTo(0.3);
    expect(onionSkinAlpha(2)).toBeCloseTo(0.15);
    expect(onionSkinAlpha(3)).toBeCloseTo(0.075);
  });
});
