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
const NEW_ANIMATION = { title: 'Mascot', width: 8, height: 8, layerName: 'Base' };

describe('the export button', () => {
  async function configure(withAnimation = true): Promise<HTMLElement> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    if (withAnimation) await TestBed.inject(EngineStore).create(NEW_ANIMATION);
    const fixture = TestBed.createComponent(ExportButton);
    fixture.detectChanges();
    await fixture.whenStable();
    return fixture.nativeElement as HTMLElement;
  }

  afterEach(() => document.body.replaceChildren());

  it('is the primary action, and opens the dialog on click', async () => {
    const root = await configure();
    const button = root.querySelector<HTMLElement>('button');
    expect(button?.classList).toContain('lp-button--primary');
    expect(button?.textContent?.trim()).toBe('Export');

    button?.click();

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

  it('is disabled while there is no animation, and Ctrl/⌘ E opens nothing', async () => {
    const root = await configure(false);
    const button = root.querySelector<HTMLButtonElement>('button');
    const event = new KeyboardEvent('keydown', { key: 'e', ctrlKey: true, cancelable: true });

    TestBed.inject(Shortcuts).handle(event);

    expect(button?.disabled).toBe(true);
    expect(TestBed.inject(ExportFlow).isOpen()).toBe(false);
  });

  it('becomes enabled once an animation is created', async () => {
    const root = await configure(false);
    const button = root.querySelector<HTMLButtonElement>('button');

    await TestBed.inject(EngineStore).create(NEW_ANIMATION);
    TestBed.tick();

    expect(button?.disabled).toBe(false);
  });
});
