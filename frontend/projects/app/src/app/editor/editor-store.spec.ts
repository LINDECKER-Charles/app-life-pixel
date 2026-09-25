import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../engine/engine-store';
import { EditorStore } from './editor-store';

const NEW_ANIMATION = { title: 'Store', width: 16, height: 16, layerName: 'Base' };

describe('EditorStore', () => {
  async function setup(): Promise<{ engine: EngineStore; store: EditorStore }> {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    const store = TestBed.inject(EditorStore);
    await engine.create(NEW_ANIMATION);
    return { engine, store };
  }

  function ids(list: readonly { readonly id: number }[] | undefined): number[] {
    return (list ?? []).map((item) => item.id);
  }

  it('starts on the topmost layer, the first frame and the first colour', async () => {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });

    const store = TestBed.inject(EditorStore);

    expect(store.activeLayer()).toBe(ids(engine.document()?.layers).at(-1));
    expect(store.activeFrame()).toBe(ids(engine.document()?.frames).at(0));
    expect(store.colorIndex()).toBe(1);
  });

  it('falls back to the next frame when the active one is deleted', async () => {
    const { engine, store } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    const [, middle, last] = ids(engine.document()?.frames);
    store.activeFrame.set(middle);

    await engine.apply({ kind: 'deleteFrame', frame: middle });

    expect(store.activeFrame()).toBe(last);
  });

  it('falls back to the previous layer when the active topmost one is deleted', async () => {
    const { engine, store } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    const [bottom, top] = ids(engine.document()?.layers);
    store.activeLayer.set(top);

    await engine.apply({ kind: 'deleteLayer', layer: top });

    expect(store.activeLayer()).toBe(bottom);
  });

  it('keeps the active layer and frame when others change', async () => {
    const { engine, store } = await setup();
    const active = store.activeFrame();

    await engine.apply({ kind: 'addFrame', position: 0, durationMs: 100 });

    expect(store.activeFrame()).toBe(active);
  });

  it('keeps the colour index within the palette', async () => {
    const { engine, store } = await setup();
    const lastIndex = (engine.document()?.palette.length ?? 0) - 1;
    store.colorIndex.set(lastIndex);

    await engine.apply({ kind: 'removePaletteEntry', index: lastIndex });

    expect(store.colorIndex()).toBe(lastIndex - 1);
  });

  it('keeps the frame selection within the frames', async () => {
    const { engine, store } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    store.frameSelection.set({ first: 1, last: 2 });

    await engine.apply({ kind: 'deleteFrame', frame: ids(engine.document()?.frames)[2] });

    expect(store.frameSelection()).toEqual({ first: 1, last: 1 });
  });

  it('follows a new document, and drops a selection that no longer fits it', async () => {
    const { engine, store } = await setup();
    store.selection.set({ x: 2, y: 2, width: 4, height: 4 });

    await engine.create({ ...NEW_ANIMATION, width: 4, height: 4 });

    expect(store.selection()).toBeNull();
    expect(store.activeLayer()).toBe(ids(engine.document()?.layers)[0]);
    expect(store.activeFrame()).toBe(ids(engine.document()?.frames)[0]);
  });
});
