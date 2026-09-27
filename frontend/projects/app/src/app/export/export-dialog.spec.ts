import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EngineStore } from '../engine/engine-store';
import type { ExportedFile, ExportFormat } from '../engine/engine-types';
import { ExportDialog } from './export-dialog';
import { EXPORT_FORMATS } from './export-formats';
import { ExportFlow } from './export-flow';
import { EXPORT_OBSERVER, type ExportObserver } from './export-observer';
import { EXPORT_SAVER, type ExportSaver } from './export-saver';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;

type SaveSpy = ReturnType<typeof vi.fn<(files: readonly ExportedFile[]) => Promise<void>>>;

describe('the export dialog', () => {
  let saveSpy: SaveSpy;
  let recordSpy: ReturnType<typeof vi.fn<(format: ExportFormat, size: number) => void>>;

  async function openDialog(): Promise<HTMLElement> {
    saveSpy = vi
      .fn<(files: readonly ExportedFile[]) => Promise<void>>()
      .mockResolvedValue(undefined);
    recordSpy = vi.fn<(format: ExportFormat, size: number) => void>();
    const saver: ExportSaver = { save: saveSpy };
    const observer: ExportObserver = { record: recordSpy };
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
        { provide: EXPORT_SAVER, useValue: saver },
        { provide: EXPORT_OBSERVER, useValue: observer },
      ],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    await TestBed.inject(EngineStore).create({
      title: 'Mascot',
      width: 8,
      height: 8,
      layerName: 'Base',
    });
    const fixture = TestBed.createComponent(ExportDialog);
    await TestBed.inject(ExportFlow).open();
    fixture.detectChanges();
    await fixture.whenStable();
    // ion-modal moves its presented content to document.body for stacking, once open.
    return document.body;
  }

  function rows(root: HTMLElement): HTMLTableRowElement[] {
    return Array.from(root.querySelectorAll('tbody tr'));
  }

  /** Waits until every format has answered, sized or failed. */
  async function everyRowSettled(root: HTMLElement): Promise<void> {
    await vi.waitFor(() => {
      expect(rows(root)).toHaveLength(EXPORT_FORMATS.length);
      expect(root.querySelector('table')?.getAttribute('aria-busy')).toBe('false');
    });
  }

  function downloadButton(root: HTMLElement, index: number): HTMLButtonElement {
    const button = rows(root)[index]?.querySelector<HTMLButtonElement>('button');
    if (!button) throw new Error(`no download button in row ${index}`);
    return button;
  }

  afterEach(() => document.body.replaceChildren());

  it('lists every format with its raw and gzip sizes', async () => {
    const root = await openDialog();

    await everyRowSettled(root);
    for (const row of rows(root)) {
      const cells = row.querySelectorAll('td');
      expect(cells[0]?.textContent).toMatch(/^\d[\d.,]*\s?[a-zA-Z]+$/);
      expect(cells[1]?.textContent).toMatch(/^\d[\d.,]*\s?[a-zA-Z]+$/);
    }
  });

  it('names the smallest format in words, on the row of the lightest gzip size', async () => {
    const root = await openDialog();
    await everyRowSettled(root);
    const flow = TestBed.inject(ExportFlow);
    const lightest = flow.lightestFormat();

    const marked = rows(root).filter((row) =>
      row.querySelector('th')?.textContent?.includes(TEXTS['export.table.lightest']),
    );

    expect(marked).toHaveLength(1);
    const index = EXPORT_FORMATS.indexOf(lightest as ExportFormat);
    expect(marked[0]).toBe(rows(root)[index]);
  });

  it('says that exporting does not save the animation', async () => {
    const root = await openDialog();

    expect(root.querySelector('ion-modal')?.textContent).toContain(TEXTS['export.download_note']);
  });

  it('re-exports a raster format when the scale changes', async () => {
    const root = await openDialog();
    await everyRowSettled(root);
    const gifCellBefore = rows(root)[1]?.querySelectorAll('td')[0]?.textContent;

    const scaleField = root.querySelector<HTMLInputElement>('#export-scale');
    if (!scaleField) throw new Error('no scale field');
    scaleField.value = '16';
    scaleField.dispatchEvent(new Event('change'));

    await vi.waitFor(() => {
      const gifCellAfter = rows(root)[1]?.querySelectorAll('td')[0]?.textContent;
      expect(gifCellAfter).not.toBe(gifCellBefore);
    });
  });

  it('bounds the scale by the engine limits', async () => {
    const root = await openDialog();
    const limits = TestBed.inject(EngineStore).limits();
    const scaleField = root.querySelector<HTMLInputElement>('#export-scale');
    if (!scaleField) throw new Error('no scale field');

    scaleField.value = String((limits?.exportMaxScale ?? 0) + 10);
    scaleField.dispatchEvent(new Event('change'));

    await vi.waitFor(() => expect(scaleField.value).toBe(String(limits?.exportMaxScale)));
  });

  it('downloads a row through ExportSaver, tells the observer, then says it is done', async () => {
    const root = await openDialog();
    await everyRowSettled(root);

    downloadButton(root, 0).click();

    await vi.waitFor(() => expect(saveSpy).toHaveBeenCalledTimes(1));
    expect(recordSpy).toHaveBeenCalledWith(EXPORT_FORMATS[0], expect.any(Number));
    const status = await vi.waitFor(() => {
      const done = root.querySelector('lp-export-download-done');
      if (!done) throw new Error('no success yet');
      return done;
    });
    expect(status.textContent).toContain('WASM export done');
    expect(status.querySelector('img')?.getAttribute('alt')).toBe('');
    expect(status.closest('[role="status"]')).not.toBeNull();
  });

  it('keeps the dialog open with an error, and no Pip, when a download fails', async () => {
    const root = await openDialog();
    await everyRowSettled(root);
    saveSpy.mockRejectedValueOnce(new Error('refused'));

    downloadButton(root, 1).click();

    const alert = await vi.waitFor(() => {
      const banner = root.querySelector('.lp-dialog__failure [role="alert"]');
      if (!banner) throw new Error('no failure yet');
      return banner;
    });
    expect(alert.textContent).toContain('Could not export the GIF files');
    expect(root.querySelector('lp-export-download-done')).toBeNull();
    expect(TestBed.inject(ExportFlow).isOpen()).toBe(true);
    expect(recordSpy).not.toHaveBeenCalled();
  });
});
