import type { RenderedFrame } from '../../engine/engine-types';
import { drawScene, type Scene } from './canvas-renderer';

/** jsdom has no real 2D context: this records the calls `drawScene` makes instead of making them. */
class FakeContext2D {
  readonly calls: string[] = [];
  fillStyle = '';
  strokeStyle = '';
  lineWidth = 1;
  globalAlpha = 1;
  imageSmoothingEnabled = true;

  save(): void {
    this.calls.push('save');
  }

  restore(): void {
    this.calls.push('restore');
  }

  translate(): void {
    this.calls.push('translate');
  }

  fillRect(): void {
    this.calls.push('fillRect');
  }

  clearRect(): void {
    this.calls.push('clearRect');
  }

  drawImage(): void {
    this.calls.push('drawImage');
  }

  putImageData(): void {
    this.calls.push('putImageData');
  }

  beginPath(): void {
    this.calls.push('beginPath');
  }

  moveTo(): void {
    /* recorded through beginPath/stroke for this test's purposes */
  }

  lineTo(): void {
    /* recorded through beginPath/stroke for this test's purposes */
  }

  stroke(): void {
    this.calls.push('stroke');
  }
}

function frame(width: number, height: number): RenderedFrame {
  return { width, height, pixels: new Uint8ClampedArray(width * height * 4) };
}

// jsdom has no ImageData either: a minimal stand-in, since this suite only checks composition.
class FakeImageData {
  constructor(
    readonly data: Uint8ClampedArray,
    readonly width: number,
    readonly height: number,
  ) {}
}

beforeAll(() => vi.stubGlobal('ImageData', FakeImageData));
afterAll(() => vi.unstubAllGlobals());

const BASE_SCENE: Scene = {
  view: { width: 320, height: 220 },
  layout: { origin: { x: 80, y: 30 }, zoom: 5, content: { width: 32, height: 32 } },
  frame: frame(32, 32),
  before: [],
  after: [],
  showGrid: false,
};

describe('drawScene', () => {
  it('clears the view and draws the checkerboard and the active frame', () => {
    const ctx = new FakeContext2D();
    const getContext = vi
      .spyOn(HTMLCanvasElement.prototype, 'getContext')
      .mockReturnValue(new FakeContext2D() as unknown as CanvasRenderingContext2D);

    drawScene(ctx as unknown as CanvasRenderingContext2D, BASE_SCENE);

    expect(ctx.calls[0]).toBe('clearRect');
    expect(ctx.calls).toContain('fillRect');
    expect(ctx.calls).toContain('drawImage');
    expect(ctx.calls).not.toContain('stroke');
    getContext.mockRestore();
  });

  it('draws the grid only when asked to', () => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      new FakeContext2D() as unknown as CanvasRenderingContext2D,
    );
    const ctx = new FakeContext2D();

    drawScene(ctx as unknown as CanvasRenderingContext2D, { ...BASE_SCENE, showGrid: true });

    expect(ctx.calls).toContain('stroke');
    vi.restoreAllMocks();
  });

  it('draws one onion-skin frame per neighbour, before and after', () => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      new FakeContext2D() as unknown as CanvasRenderingContext2D,
    );
    const ctx = new FakeContext2D();
    const scene: Scene = {
      ...BASE_SCENE,
      frame: null,
      before: [{ image: frame(32, 32), stepsAway: 1 }],
      after: [
        { image: frame(32, 32), stepsAway: 1 },
        { image: frame(32, 32), stepsAway: 2 },
      ],
    };

    drawScene(ctx as unknown as CanvasRenderingContext2D, scene);

    expect(ctx.calls.filter((call) => call === 'drawImage')).toHaveLength(3);
    vi.restoreAllMocks();
  });

  it('draws nothing beyond the background when the frame has not loaded yet', () => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null);
    const ctx = new FakeContext2D();

    drawScene(ctx as unknown as CanvasRenderingContext2D, { ...BASE_SCENE, frame: null });

    expect(ctx.calls).not.toContain('drawImage');
    vi.restoreAllMocks();
  });
});
