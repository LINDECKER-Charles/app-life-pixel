import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { ExportButton } from './export-button';
import { ExportFlow } from './export-flow';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

describe('the export button', () => {
  async function configure(): Promise<HTMLElement> {
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
    const fixture = TestBed.createComponent(ExportButton);
    fixture.detectChanges();
    await fixture.whenStable();
    return fixture.nativeElement as HTMLElement;
  }

  afterEach(() => document.body.replaceChildren());

  it('opens the dialog on click', async () => {
    await configure();

    document.body.querySelector<HTMLElement>('ion-button')?.click();

    await vi.waitFor(() => expect(TestBed.inject(ExportFlow).isOpen()).toBe(true));
  });

  it('opens the dialog on Ctrl/⌘ E', async () => {
    await configure();
    const event = new KeyboardEvent('keydown', {
      key: 'e',
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });

    TestBed.inject(Shortcuts).handle(event);

    await vi.waitFor(() => expect(TestBed.inject(ExportFlow).isOpen()).toBe(true));
    expect(event.defaultPrevented).toBe(true);
  });
});
