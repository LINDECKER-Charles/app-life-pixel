import { runEngineContract } from './engine-contract';
import { MockEditorEngine } from './mock-editor-engine';

describe('MockEditorEngine', () => {
  runEngineContract(async () => new MockEditorEngine());

  it('records a paintStroke without changing the summary', async () => {
    const engine = new MockEditorEngine();
    await engine.create({ title: 'Paint', width: 4, height: 4, layerName: 'Base' });
    const before = await engine.serialize();

    await engine.apply({
      kind: 'paintStroke',
      layer: 1,
      frame: 2,
      points: [{ x: 0, y: 0 }],
      index: 1,
    });

    expect(await engine.serialize()).toEqual(before);
    const rendered = await engine.render({ frame: 2 });
    expect(rendered.pixels.slice(0, 4)).toEqual(Uint8ClampedArray.from([0, 0, 0, 255]));
  });

  it('rejects deleting the last layer or the last frame', async () => {
    const engine = new MockEditorEngine();
    await engine.create({ title: 'Guard', width: 2, height: 2, layerName: 'Base' });

    await expect(engine.apply({ kind: 'deleteLayer', layer: 1 })).rejects.toMatchObject({
      code: 'edit.last_layer',
    });
    await expect(engine.apply({ kind: 'deleteFrame', frame: 2 })).rejects.toMatchObject({
      code: 'edit.last_frame',
    });
  });
});
