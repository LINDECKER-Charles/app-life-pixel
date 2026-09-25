import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../../engine/engine-store';
import { ImportImage } from './import-image';

const NEW_ANIMATION = { title: 'Import', width: 8, height: 8, layerName: 'Base' };

function pngFile(bytes: number): File {
  return new File([new Uint8Array(bytes)], 'sprite.png', { type: 'image/png' });
}

describe('ImportImage', () => {
  async function setup(): Promise<{ engine: EngineStore; importImage: ImportImage }> {
    TestBed.configureTestingModule({});
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    return { engine, importImage: TestBed.inject(ImportImage) };
  }

  it('applies importImage at the top-left corner of the active layer and frame', async () => {
    const { engine, importImage } = await setup();
    const apply = vi.spyOn(engine, 'apply');
    const layer = engine.document()?.layers[0].id ?? 0;
    const frame = engine.document()?.frames[0].id ?? 0;

    await importImage.importFile(pngFile(16));

    expect(apply).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'importImage', layer, frame, at: { x: 0, y: 0 } }),
    );
    expect(importImage.error()).toBeNull();
  });

  it('refuses a file larger than the limits before reading it', async () => {
    const { engine, importImage } = await setup();
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    const apply = vi.spyOn(engine, 'apply');
    const file = pngFile(limits.importMaxBytes + 1);
    const arrayBuffer = vi.spyOn(file, 'arrayBuffer');

    await importImage.importFile(file);

    expect(apply).not.toHaveBeenCalled();
    expect(arrayBuffer).not.toHaveBeenCalled();
    expect(importImage.error()).toEqual({
      maxSide: limits.importMaxSide,
      maxBytes: limits.importMaxBytes,
    });
  });
});
