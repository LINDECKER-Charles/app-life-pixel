import { TestBed } from '@angular/core/testing';
import { EngineStore } from '../engine/engine-store';
import type { ExportedFile, ExportFormat } from '../engine/engine-types';
import { EXPORT_FORMATS } from './export-formats';
import { ExportFlow } from './export-flow';
import { EXPORT_OBSERVER, type ExportObserver } from './export-observer';
import { EXPORT_SAVER, type ExportSaver } from './export-saver';

describe('ExportFlow', () => {
  let saveSpy: ReturnType<typeof vi.fn<(files: readonly ExportedFile[]) => Promise<void>>>;
  let recordSpy: ReturnType<typeof vi.fn<(format: ExportFormat, size: number) => void>>;

  async function configure(): Promise<ExportFlow> {
    saveSpy = vi
      .fn<(files: readonly ExportedFile[]) => Promise<void>>()
      .mockResolvedValue(undefined);
    recordSpy = vi.fn<(format: ExportFormat, size: number) => void>();
    const saver: ExportSaver = { save: saveSpy };
    const observer: ExportObserver = { record: recordSpy };
    TestBed.configureTestingModule({
      providers: [
        { provide: EXPORT_SAVER, useValue: saver },
        { provide: EXPORT_OBSERVER, useValue: observer },
      ],
    });
    const engine = TestBed.inject(EngineStore);
    await engine.create({ title: 'Mascot', width: 8, height: 8, layerName: 'Base' });
    return TestBed.inject(ExportFlow);
  }

  it('exports every format, with a raw and a gzip size, when it opens', async () => {
    const flow = await configure();

    await flow.open();

    expect(flow.rows().map((row) => row.format)).toEqual(EXPORT_FORMATS);
    for (const row of flow.rows()) {
      expect(row.status).toBe('ready');
      expect(row.rawBytes).toBeGreaterThan(0);
      expect(row.gzipBytes).toBeGreaterThan(0);
    }
    expect(flow.lightestFormat()).not.toBeNull();
  });

  it('re-exports when the scale changes, within the limits', async () => {
    const flow = await configure();
    await flow.open();
    const before = flow.rows().find((row) => row.format === 'gif');

    await flow.setScale(16);

    const after = flow.rows().find((row) => row.format === 'gif');
    expect(after?.rawBytes).not.toBe(before?.rawBytes);
  });

  it('re-exports when the tag changes', async () => {
    const flow = await configure();
    await TestBed.inject(EngineStore).apply({
      kind: 'addTag',
      tag: { name: 'blink', first: 0, last: 0, loop: 'once' },
    });
    await flow.open();
    const before = flow.rows().find((row) => row.format === 'gif');

    await flow.setTag('blink');

    const after = flow.rows().find((row) => row.format === 'gif');
    expect(after?.rawBytes).not.toBe(before?.rawBytes);
  });

  it('downloads a ready row through ExportSaver, then tells the observer', async () => {
    const flow = await configure();
    await flow.open();

    await flow.download('gif');

    expect(saveSpy).toHaveBeenCalledTimes(1);
    const files = saveSpy.mock.calls[0]?.[0] ?? [];
    expect(files.length).toBeGreaterThan(0);
    expect(recordSpy).toHaveBeenCalledWith('gif', expect.any(Number));
  });
});
