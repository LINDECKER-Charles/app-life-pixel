import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../../engine/engine-store';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';

const NEW_ANIMATION = { title: 'Sheet', width: 8, height: 8, layerName: 'Base' };

function pngFile(bytes: number): File {
  return new File([new Uint8Array(bytes)], 'sheet.png', { type: 'image/png' });
}

describe('ImportSpriteSheetFlow', () => {
  async function setup(): Promise<{ engine: EngineStore; flow: ImportSpriteSheetFlow }> {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    return { engine, flow: TestBed.inject(ImportSpriteSheetFlow) };
  }

  it('opens the dialog once a valid file is picked', async () => {
    const { flow } = await setup();

    await flow.pickFile(pngFile(16));

    expect(flow.isOpen()).toBe(true);
    expect(flow.error()).toBeNull();
  });

  it('refuses an oversized file before reading it, without opening the dialog', async () => {
    const { engine, flow } = await setup();
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    const file = pngFile(limits.importMaxBytes + 1);
    const arrayBuffer = vi.spyOn(file, 'arrayBuffer');

    await flow.pickFile(file);

    expect(flow.isOpen()).toBe(false);
    expect(arrayBuffer).not.toHaveBeenCalled();
    expect(flow.error()).toEqual({
      maxSide: limits.importMaxSide,
      maxBytes: limits.importMaxBytes,
    });
  });

  it('applies importSpriteSheet after the active frame, then closes', async () => {
    const { engine, flow } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    const apply = vi.spyOn(engine, 'apply');
    const layer = engine.document()?.layers[0].id ?? 0;
    await flow.pickFile(pngFile(16));

    await flow.submit({ cellWidth: 4, cellHeight: 4, durationMs: 100 });

    expect(apply).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'importSpriteSheet',
        layer,
        position: 1,
        cellWidth: 4,
        cellHeight: 4,
        durationMs: 100,
      }),
    );
    expect(flow.isOpen()).toBe(false);
  });
});
