import { importProvidersFrom } from '@angular/core';
import { TestBed, type ComponentFixture } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { EngineStore } from '../engine/engine-store';
import { ExportFlow } from './export-flow';
import { ExportSnippet } from './export-snippet';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

describe('the export snippet', () => {
  async function configure(): Promise<ComponentFixture<ExportSnippet>> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
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

  it('copies the code to the clipboard, and announces it', async () => {
    const fixture = await configure();
    const root = fixture.nativeElement as HTMLElement;
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });

    await vi.waitFor(() =>
      expect(root.querySelector('code')?.textContent?.trim().length).toBeGreaterThan(0),
    );
    const button = root.querySelector<HTMLButtonElement>('button');
    button?.click();
    fixture.detectChanges();

    const code = root.querySelector('code')?.textContent ?? '';
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith(code));
    await vi.waitFor(() => expect(root.textContent).toContain(en['export.snippet.copied']));
  });
});
