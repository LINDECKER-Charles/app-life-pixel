import type { EditorEngine } from '../../engine/editor-engine';
import type { RenderRequest, RenderedFrame } from '../../engine/engine-types';
import { loadActiveFrame, loadOnionSkin } from './canvas-frame-loader';

function fakeEngine(): { engine: EditorEngine; requests: RenderRequest[] } {
  const requests: RenderRequest[] = [];
  const engine = {
    render: async (request: RenderRequest): Promise<RenderedFrame> => {
      requests.push(request);
      return { width: 1, height: 1, pixels: new Uint8ClampedArray([request.frame, 0, 0, 0]) };
    },
  } as unknown as EditorEngine;
  return { engine, requests };
}

describe('loadActiveFrame', () => {
  it('renders null when there is no active frame', async () => {
    const { engine } = fakeEngine();
    expect(await loadActiveFrame(engine, null, null)).toBeNull();
  });

  it('renders the frame with a preview when a gesture is in progress', async () => {
    const { engine, requests } = fakeEngine();
    const preview = { kind: 'fill', layer: 1, frame: 2, at: { x: 0, y: 0 }, index: 1 } as const;

    await loadActiveFrame(engine, 2, preview);

    expect(requests).toEqual([{ frame: 2, preview }]);
  });

  it('renders the frame with no preview outside a gesture', async () => {
    const { engine, requests } = fakeEngine();

    await loadActiveFrame(engine, 2, null);

    expect(requests).toEqual([{ frame: 2 }]);
  });
});

describe('loadOnionSkin', () => {
  const frames = [{ id: 10 }, { id: 11 }, { id: 12 }, { id: 13 }];

  it('renders nothing when onion skin is disabled', async () => {
    const { engine } = fakeEngine();

    const result = await loadOnionSkin(engine, {
      frames,
      activeFrame: 11,
      onionSkin: { enabled: false, before: 1, after: 1 },
    });

    expect(result).toEqual({ before: [], after: [] });
  });

  it('renders the requested number of neighbours on each side, with their step', async () => {
    const { engine } = fakeEngine();

    const result = await loadOnionSkin(engine, {
      frames,
      activeFrame: 12,
      onionSkin: { enabled: true, before: 2, after: 1 },
    });

    expect(result.before.map((skin) => skin.stepsAway)).toEqual([1, 2]);
    expect(result.after.map((skin) => skin.stepsAway)).toEqual([1]);
  });

  it('stops at the edge of the frame list', async () => {
    const { engine } = fakeEngine();

    const result = await loadOnionSkin(engine, {
      frames,
      activeFrame: 10,
      onionSkin: { enabled: true, before: 3, after: 0 },
    });

    expect(result.before).toEqual([]);
  });
});
