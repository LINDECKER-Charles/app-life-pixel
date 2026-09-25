import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
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

describe('the export dialog', () => {
  let saveSpy: ReturnType<typeof vi.fn<(files: readonly ExportedFile[]) => Promise<void>>>;
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

  afterEach(() => document.body.replaceChildren());

  it('lists every format with its raw and gzip sizes, the lightest marked', async () => {
    const root = await openDialog();

    await vi.waitFor(() => expect(rows(root)).toHaveLength(EXPORT_FORMATS.length));
    for (const row of rows(root)) {
      const cells = row.querySelectorAll('td');
      expect(cells[0]?.textContent?.trim()).not.toBe('');
      expect(cells[1]?.textContent?.trim()).not.toBe('');
    }
    expect(root.querySelectorAll('.badge')).toHaveLength(1);
  });

  it('re-exports a raster format when the scale changes', async () => {
    const root = await openDialog();
    await vi.waitFor(() => expect(rows(root)).toHaveLength(EXPORT_FORMATS.length));
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

  it('downloads a row through ExportSaver, then tells the observer', async () => {
    const root = await openDialog();
    await vi.waitFor(() => expect(rows(root)).toHaveLength(EXPORT_FORMATS.length));

    const button = rows(root)[0]?.querySelector<HTMLButtonElement>('button');
    button?.click();

    await vi.waitFor(() => expect(saveSpy).toHaveBeenCalledTimes(1));
    expect(recordSpy).toHaveBeenCalledWith(EXPORT_FORMATS[0], expect.any(Number));
  });
});
