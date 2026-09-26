import { importProvidersFrom } from '@angular/core';
import { TestBed, type ComponentFixture } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { EngineStore } from '../engine/engine-store';
import { ExportFlow } from './export-flow';
import { ExportSnippet } from './export-snippet';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;

describe('the export snippet', () => {
  async function configure(): Promise<ComponentFixture<ExportSnippet>> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    await TestBed.inject(EngineStore).create({
      title: 'Mascot',
      width: 8,
      height: 8,
      layerName: 'Base',
    });
    await TestBed.inject(ExportFlow).open();
    const fixture = TestBed.createComponent(ExportSnippet);
    fixture.detectChanges();
    await fixture.whenStable();
    return fixture;
  }

  afterEach(() => document.body.replaceChildren());

  it('defaults the URLs from the WASM export, and calls engine.snippet on every change', async () => {
    const fixture = await configure();
    const root = fixture.nativeElement as HTMLElement;
    const src = root.querySelector<HTMLInputElement>('#export-src');
    expect(src?.value).toBe('/assets/mascot.wasm');
    const loader = root.querySelector<HTMLInputElement>('#export-loader');
    expect(loader?.value).toBe('/assets/life-pixel.js');

    const engine = TestBed.inject(EDITOR_ENGINE);
    const spy = vi.spyOn(engine, 'snippet');

    const framework = root.querySelector<HTMLSelectElement>('#export-framework');
    if (!framework) throw new Error('no framework field');
    framework.value = 'react';
    framework.dispatchEvent(new Event('change'));
    fixture.detectChanges();

    await vi.waitFor(() =>
      expect(spy).toHaveBeenCalledWith(
        expect.objectContaining({ framework: 'react', src: '/assets/mascot.wasm' }),
      ),
    );
    const code = root.querySelector('code');
    await vi.waitFor(() => expect(code?.textContent).toContain('react'));
  });

  /** Waits for the code, then clicks Copy. */
  async function clickCopy(fixture: ComponentFixture<ExportSnippet>): Promise<HTMLElement> {
    const root = fixture.nativeElement as HTMLElement;
    await vi.waitFor(() =>
      expect(root.querySelector('code')?.textContent?.trim().length).toBeGreaterThan(0),
    );
    fixture.detectChanges();
    root.querySelector<HTMLButtonElement>('.copy button')?.click();
    return root;
  }

  function stubClipboard(clipboard: Partial<Clipboard> | undefined): void {
    Object.defineProperty(navigator, 'clipboard', { value: clipboard, configurable: true });
  }

  it('copies the code to the clipboard, then says it is copied', async () => {
    const fixture = await configure();
    const writeText = vi.fn().mockResolvedValue(undefined);
    stubClipboard({ writeText });

    const root = await clickCopy(fixture);

    const code = root.querySelector('code')?.textContent ?? '';
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith(code));
    await vi.waitFor(() => {
      fixture.detectChanges();
      expect(root.querySelector('.copy button')?.textContent?.trim()).toBe(
        TEXTS['export.snippet.copied'],
      );
    });
    expect(root.querySelector('[aria-live="polite"]')?.textContent).toContain(
      TEXTS['export.snippet.copied'],
    );
    expect(root.querySelector('[role="alert"]')).toBeNull();
  });

  it('says the copy failed, and how to copy by hand, when the browser refuses', async () => {
    const fixture = await configure();
    stubClipboard({ writeText: vi.fn().mockRejectedValue(new Error('denied')) });

    const root = await clickCopy(fixture);

    const alert = await vi.waitFor(() => {
      fixture.detectChanges();
      const banner = root.querySelector('[role="alert"]');
      if (!banner) throw new Error('no failure yet');
      return banner;
    });
    expect(alert.textContent).toContain(TEXTS['export.snippet.copy_failed']);
    expect(root.querySelector('.copy button')?.textContent?.trim()).toBe(
      TEXTS['export.snippet.copy'],
    );
  });

  it('says the copy failed where the browser has no clipboard', async () => {
    const fixture = await configure();
    stubClipboard(undefined);

    const root = await clickCopy(fixture);

    await vi.waitFor(() => {
      fixture.detectChanges();
      expect(root.querySelector('[role="alert"]')?.textContent).toContain(
        TEXTS['export.snippet.copy_failed'],
      );
    });
  });
});
