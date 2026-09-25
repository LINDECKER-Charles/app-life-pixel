/*
 * W0's contract suite and W1's own checks against editor-wasm in its worker, in a real Chromium:
 * `npm run test:engine` (editor.md, W1).
 */
import type { RecoveryReport } from '../engine-recovery';
import type { EngineState, NewAnimationOptions } from '../engine-types';
import { runEngineContract } from '../testing/engine-contract';
import { SNAPSHOT_INTERVAL_MS } from './snapshot';
import {
  createEngineWorker,
  WasmEditorEngine,
  type WasmEditorEngineOptions,
} from './wasm-editor-engine';

const NEW_ANIMATION: NewAnimationOptions = {
  title: 'Engine Test',
  width: 4,
  height: 4,
  layerName: 'Base',
};
const started: WasmEditorEngine[] = [];

/**
 * The worker, loaded from the root of the test build. Vitest serves each spec from its source
 * path, where `new URL('worker-….js', import.meta.url)` finds nothing, and Vite adds a query of
 * its own; the app serves its chunks from the root of the build, beside the worker.
 */
class RootedWorker extends Worker {
  constructor(url: string | URL, options?: WorkerOptions) {
    const target = new URL(url, location.href);
    if (target.protocol !== 'blob:') {
      target.pathname = target.pathname.slice(target.pathname.lastIndexOf('/'));
      target.search = '';
    }
    super(target, options);
  }
}

function start(options?: WasmEditorEngineOptions): WasmEditorEngine {
  const engine = new WasmEditorEngine(options);
  started.push(engine);
  return engine;
}

function follow(engine: WasmEditorEngine): { latest(): EngineState; statuses: string[] } {
  const statuses: string[] = [];
  let latest: EngineState | undefined;
  engine.subscribe((state) => {
    statuses.push(state.status);
    latest = state;
  });
  return { latest: () => latest ?? expect.unreachable('no state'), statuses };
}

/** An engine whose current worker `fail` breaks with a request it cannot read. */
function startBreakable(): { engine: WasmEditorEngine; fail(): void } {
  let worker: Worker | undefined;
  const engine = start({ createWorker: () => (worker = createEngineWorker()) });
  return { engine, fail: () => worker?.postMessage({ id: 0, method: 'fail', args: [] }) };
}

function nextRecovery(engine: WasmEditorEngine): Promise<RecoveryReport> {
  return new Promise((resolve) => {
    const stop = engine.onRecovered((report) => {
      stop();
      resolve(report);
    });
  });
}

/** The RGBA bytes of a `#rrggbbaa` colour. */
function bytesOf(color: string): number[] {
  return [1, 3, 5, 7].map((start) => Number.parseInt(color.slice(start, start + 2), 16));
}

beforeEach(() => {
  vi.stubGlobal('Worker', RootedWorker);
});

afterEach(() => {
  for (const engine of started.splice(0)) engine.dispose();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('WasmEditorEngine', () => {
  runEngineContract(async () => start());

  it('renders a frame as RGBA, and previews an operation without applying it', async () => {
    const engine = start();
    const state = follow(engine);
    await engine.create(NEW_ANIMATION);
    const { layers, frames, palette } = state.latest().document ?? expect.unreachable();
    const [layer, frame] = [layers[0].id, frames[0].id];
    await engine.apply({ kind: 'paintStroke', layer, frame, points: [{ x: 1, y: 2 }], index: 1 });

    const rendered = await engine.render({ frame });
    expect(rendered.pixels).toBeInstanceOf(Uint8ClampedArray);
    expect(rendered.pixels).toHaveLength(4 * 4 * 4);
    const at = (x: number, y: number) => [
      ...rendered.pixels.slice((y * 4 + x) * 4, (y * 4 + x + 1) * 4),
    ];
    expect(at(1, 2)).toEqual(bytesOf(palette[1]));

    const line = {
      kind: 'line',
      layer,
      frame,
      from: { x: 0, y: 0 },
      to: { x: 3, y: 0 },
      index: 2,
    } as const;
    const preview = await engine.render({ frame, preview: line });
    expect([...preview.pixels.slice(0, 4)]).toEqual(bytesOf(palette[2]));
    expect(await engine.render({ frame })).toEqual(rendered);
  });

  it('names the WebAssembly export after the title, with the loader beside it', async () => {
    const engine = start();
    await engine.create(NEW_ANIMATION);

    const { files, totalBytes } = await engine.export({ format: 'wasm' });
    expect(files.map(({ name, mediaType }) => [name, mediaType])).toEqual([
      ['engine-test.wasm', 'application/wasm'],
      ['life-pixel.js', 'text/javascript'],
    ]);
    expect(files[0].bytes).toBeInstanceOf(Uint8Array);
    expect(totalBytes).toBe(files[0].bytes.byteLength + files[1].bytes.byteLength);
    await expect(engine.export({ format: 'gif', scale: 0 })).rejects.toMatchObject({
      code: 'export.scale',
    });
  });

  it('gives the integration snippet of the export', async () => {
    const engine = start();
    await engine.create(NEW_ANIMATION);

    const snippet = await engine.snippet({
      framework: 'html',
      src: '/assets/engine-test.wasm',
      loader: '/assets/life-pixel.js',
      alt: 'A test',
    });
    expect(snippet).toContain('/assets/engine-test.wasm');
  });

  it('transfers the document it opens, and refuses one it cannot read', async () => {
    const engine = start();
    await engine.create(NEW_ANIMATION);
    const bytes = await engine.serialize();

    await engine.open(bytes);
    expect(bytes.byteLength).toBe(0);
    await expect(engine.open(new TextEncoder().encode('not json'))).rejects.toMatchObject({
      code: 'document.malformed',
    });
  });

  it('refuses every request but the history before an animation is open', async () => {
    const engine = start();

    await expect(engine.serialize()).rejects.toMatchObject({ code: 'engine.no_document' });
    await expect(engine.undo()).resolves.toBeUndefined();
  });
});

describe('WasmEditorEngine recovery', () => {
  it('reopens the last save after a worker failure, and says what was lost', async () => {
    const { engine, fail } = startBreakable();
    const state = follow(engine);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'setTitle', title: 'Saved' });
    await engine.markSaved();
    await engine.apply({ kind: 'addLayer', position: 0, name: 'Lost' });
    await engine.apply({ kind: 'setTitle', title: 'Lost too' });

    const recovered = nextRecovery(engine);
    fail();
    expect(await recovered).toEqual({ lostChanges: 2 });

    expect(state.statuses.slice(-2)).toEqual(['failed', 'ready']);
    const document = state.latest().document;
    expect(document?.title).toBe('Saved');
    expect(document?.layers).toHaveLength(1);
    expect(state.latest().hasUnsavedWork).toBe(false);
    await engine.apply({ kind: 'addLayer', position: 0, name: 'After' });
    expect(state.latest().document?.layers).toHaveLength(2);
  });

  it('restores the snapshot of 30 seconds of changes as unsaved work', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const { engine, fail } = startBreakable();
    const state = follow(engine);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'setTitle', title: 'Kept' });
    vi.advanceTimersByTime(SNAPSHOT_INTERVAL_MS);
    await engine.serialize(); // answered after the snapshot's own serialize
    await engine.apply({ kind: 'setTitle', title: 'Lost' });

    const recovered = nextRecovery(engine);
    fail();
    expect(await recovered).toEqual({ lostChanges: 1 });

    expect(state.latest().document?.title).toBe('Kept');
    expect(state.latest().hasUnsavedWork).toBe(true);
    expect(state.latest().canUndo).toBe(false);
  });

  it('recovers an engine without an animation to an empty one', async () => {
    const { engine, fail } = startBreakable();
    const state = follow(engine);
    await expect(engine.undo()).resolves.toBeUndefined();

    const recovered = nextRecovery(engine);
    fail();
    expect(await recovered).toEqual({ lostChanges: 0 });
    expect(state.statuses.slice(-2)).toEqual(['failed', 'empty']);
  });

  it('stays failed when its worker fails before answering, and refuses every request', async () => {
    let workers = 0;
    const broken = new Blob(['throw new Error("no engine");'], { type: 'text/javascript' });
    const engine = start({
      createWorker: () => {
        workers += 1;
        return new Worker(URL.createObjectURL(broken), { type: 'module' });
      },
    });
    const failed = new Promise<void>((resolve) =>
      engine.subscribe((state) => {
        if (state.status === 'failed') resolve();
      }),
    );

    await failed;
    await expect(engine.create(NEW_ANIMATION)).rejects.toEqual({
      code: 'internal.error',
      params: {},
    });
    expect(workers).toBe(1);
  });
});
