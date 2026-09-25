import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../engine/engine-store';
import { PaletteEntryFlow } from './palette-entry-flow';

const NEW_ANIMATION = { title: 'Flow', width: 8, height: 8, layerName: 'Base' };

describe('PaletteEntryFlow', () => {
  async function setup(): Promise<{ engine: EngineStore; flow: PaletteEntryFlow }> {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    return { engine, flow: TestBed.inject(PaletteEntryFlow) };
  }

  it('adds a colour and closes once the palette grows', async () => {
    const { engine, flow } = await setup();
    const before = engine.document()?.palette.length ?? 0;
    flow.openAdd();

    await flow.submit('#11223344');

    expect(engine.document()?.palette.length).toBe(before + 1);
    expect(engine.document()?.palette.at(-1)).toBe('#11223344');
    expect(flow.current()).toBeNull();
  });

  it('edits a colour and closes once it changes', async () => {
    const { engine, flow } = await setup();
    flow.openEdit(1, engine.document()?.palette[1] ?? '#000000ff');

    await flow.submit('#55667788');

    expect(engine.document()?.palette[1]).toBe('#55667788');
    expect(flow.current()).toBeNull();
  });

  it('stays open when the palette is full', async () => {
    const { engine, flow } = await setup();
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    const start = engine.document()?.palette.length ?? 0;
    for (let index = start; index < limits.maxPaletteEntries; index++) {
      await engine.apply({ kind: 'addPaletteEntry', color: '#00000000' });
    }
    flow.openAdd();

    await flow.submit('#11223344');

    expect(flow.current()).not.toBeNull();
  });

  it('does nothing with no mode open', async () => {
    const { engine, flow } = await setup();
    const before = engine.document()?.palette.length;

    await flow.submit('#11223344');

    expect(engine.document()?.palette.length).toBe(before);
  });
});
