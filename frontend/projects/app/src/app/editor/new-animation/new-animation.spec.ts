import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { DiscardConfirmation } from './discard-confirmation';
import { NewAnimationDialog } from './new-animation-dialog';
import { NewAnimationFlow } from './new-animation-flow';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

describe('the new-animation dialog', () => {
  let confirm: ReturnType<typeof vi.fn<() => Promise<boolean>>>;

  async function configure(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        { provide: DiscardConfirmation, useValue: { confirm } },
      ],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  }

  async function openDialog(): Promise<HTMLFormElement> {
    await configure();
    const fixture = TestBed.createComponent(NewAnimationDialog);
    await TestBed.inject(NewAnimationFlow).start();
    await fixture.whenStable();
    return vi.waitFor(() => {
      const form = document.querySelector<HTMLFormElement>('lp-new-animation-form form');
      if (!form) throw new Error('the dialog is not open');
      return form;
    });
  }

  function field(form: HTMLFormElement, id: string): HTMLInputElement {
    const input = form.querySelector<HTMLInputElement>(`#${id}`);
    if (!input) throw new Error(`no field ${id}`);
    return input;
  }

  beforeEach(() => {
    confirm = vi.fn<() => Promise<boolean>>();
  });

  afterEach(() => document.body.replaceChildren());

  it('creates a 32 × 32 animation with the translated default names', async () => {
    const form = await openDialog();
    expect(field(form, 'new-animation-title').value).toBe(en['editor.new.default_title']);

    form.querySelector<HTMLButtonElement>('button[type="submit"]')?.click();

    const engine = TestBed.inject(EngineStore);
    await vi.waitFor(() => expect(engine.document()).not.toBeNull());
    expect(engine.document()).toMatchObject({
      title: en['editor.new.default_title'],
      width: 32,
      height: 32,
      layers: [{ name: en['editor.new.layer_name'] }],
    });
    await vi.waitFor(() => expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(false));
  });

  it('bounds the size by the engine limits', async () => {
    const form = await openDialog();
    const limits = TestBed.inject(EngineStore).limits();
    const width = field(form, 'new-animation-width');
    expect(width.max).toBe(String(limits?.canvasMaxSide));

    width.value = String((limits?.canvasMaxSide ?? 0) + 1);
    width.dispatchEvent(new Event('input'));
    TestBed.tick();

    expect(width.getAttribute('aria-invalid')).toBe('true');
    expect(form.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(true);
  });

  it('asks before replacing unsaved work, and opens only once allowed', async () => {
    await configure();
    const engine = TestBed.inject(EngineStore);
    const flow = TestBed.inject(NewAnimationFlow);
    await engine.create({ title: 'Work', width: 8, height: 8, layerName: 'Base' });
    await engine.apply({ kind: 'setTitle', title: 'Work in progress' });

    confirm.mockResolvedValueOnce(false);
    await flow.start();
    expect(flow.isOpen()).toBe(false);

    confirm.mockResolvedValueOnce(true);
    await flow.start();
    expect(flow.isOpen()).toBe(true);
    expect(confirm).toHaveBeenCalledTimes(2);
  });

  it('opens without asking when there is nothing to lose', async () => {
    await configure();
    const flow = TestBed.inject(NewAnimationFlow);

    await flow.start();

    expect(flow.isOpen()).toBe(true);
    expect(confirm).not.toHaveBeenCalled();
  });
});
