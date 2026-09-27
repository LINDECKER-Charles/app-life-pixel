import {
  ApplicationRef,
  type ComponentRef,
  createComponent,
  EnvironmentInjector,
  importProvidersFrom,
} from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation } from '../../library/current-animation';
import { DiscardConfirmation } from './discard-confirmation';
import { NewAnimationDialog } from './new-animation-dialog';
import { NewAnimationFlow } from './new-animation-flow';
import { NewAnimationForm } from './new-animation-form';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

describe('the new-animation dialog', () => {
  let confirm: ReturnType<typeof vi.fn<() => Promise<boolean>>>;

  async function configure(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
        { provide: DiscardConfirmation, useValue: { confirm } },
        provideRouter([{ path: '**', children: [] }]),
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

  /** The field its label names, reached through that label as assistive technology does. */
  function field(form: HTMLFormElement, label: string): HTMLInputElement {
    const labels = Array.from(form.querySelectorAll('label'));
    const control = labels.find((candidate) => candidate.textContent?.trim() === label)?.control;
    if (!(control instanceof HTMLInputElement)) throw new Error(`no field labelled ${label}`);
    return control;
  }

  /** The texts that describe `element` through its `aria-describedby`, in order. */
  function descriptions(element: Element | null): string[] {
    const ids = element?.getAttribute('aria-describedby')?.split(' ') ?? [];
    return ids.map((id) => document.getElementById(id)?.textContent?.trim() ?? `no #${id}`);
  }

  beforeEach(() => {
    confirm = vi.fn<() => Promise<boolean>>();
  });

  afterEach(() => document.body.replaceChildren());

  it('creates a 32 × 32 animation with the translated default names', async () => {
    const form = await openDialog();
    expect(field(form, 'Title').value).toBe(en['editor.new.default_title']);

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
    const width = field(form, 'Width');
    expect(width.max).toBe(String(limits?.canvasMaxSide));

    width.value = String((limits?.canvasMaxSide ?? 0) + 1);
    width.dispatchEvent(new Event('input'));
    TestBed.tick();

    expect(width.getAttribute('aria-invalid')).toBe('true');
    expect(form.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(true);
  });

  it('focuses the title, its default selected, once the dialog is shown', async () => {
    const form = await openDialog();
    const title = field(form, 'Title');

    await vi.waitFor(() => expect(document.activeElement).toBe(title));
    expect(title.selectionStart).toBe(0);
    expect(title.selectionEnd).toBe(title.value.length);
  });

  it('states the sides in pixels, within the engine limits', async () => {
    const form = await openDialog();
    const limits = TestBed.inject(EngineStore).limits();
    const hints = descriptions(form.querySelector('fieldset'));

    expect(hints).toEqual([
      `From ${limits?.canvasMinSide} to ${limits?.canvasMaxSide} pixels on each side.`,
    ]);
    const units = Array.from(form.querySelectorAll('.lp-field__unit'), (unit) => unit.textContent);
    expect(units.map((unit) => unit?.trim())).toEqual(['px', 'px']);
  });

  it('explains a side out of the limits next to it, keeping what was typed', async () => {
    const form = await openDialog();
    const height = field(form, 'Height');

    height.value = '0';
    height.dispatchEvent(new Event('input'));
    TestBed.tick();

    expect(descriptions(height)).toEqual([expect.stringContaining('Enter a whole number from')]);
    expect(height.closest('.lp-field')?.textContent).toContain('Enter a whole number from');
    expect(height.value).toBe('0');
  });

  it('keeps its labels and hints on its own fields beside another copy of the form', async () => {
    const form = await openDialog();
    const copy = await mountFormCopyFirst();
    try {
      expect(form.isConnected).toBe(true);
      for (const label of Array.from(form.querySelectorAll('label'))) {
        expect(form.contains(label.control), label.textContent?.trim()).toBe(true);
      }
      for (const described of Array.from(form.querySelectorAll('[aria-describedby]'))) {
        for (const id of described.getAttribute('aria-describedby')?.split(' ') ?? []) {
          expect(form.contains(document.getElementById(id)), id).toBe(true);
        }
      }
    } finally {
      copy.destroy();
    }
  });

  /**
   * Mounts another form first in the document, where it would take any id the two forms shared.
   * Not through `TestBed.createComponent`: that removes the dialog fixture's root, and Ionic
   * dismisses — unmounting its form — a modal whose original parent leaves the document.
   */
  async function mountFormCopyFirst(): Promise<ComponentRef<NewAnimationForm>> {
    const host = document.createElement('div');
    document.body.prepend(host);
    const application = TestBed.inject(ApplicationRef);
    const copy = createComponent(NewAnimationForm, {
      environmentInjector: TestBed.inject(EnvironmentInjector),
      hostElement: host,
    });
    copy.setInput('limits', TestBed.inject(EngineStore).limits());
    application.attachView(copy.hostView);
    await application.whenStable();
    return copy;
  }

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

  /** Puts a saved animation at `/editor/a1`, as H8's `CurrentAnimation` would after a save. */
  async function openSavedRoute(): Promise<{ router: Router; current: CurrentAnimation }> {
    await configure();
    const router = TestBed.inject(Router);
    await router.navigateByUrl('/editor/a1');
    const current = TestBed.inject(CurrentAnimation);
    current.setSaved({
      id: 'a1',
      projectId: 'p1',
      title: 'Saved',
      width: 8,
      height: 8,
      frameCount: 1,
      documentBytes: 1,
      version: 1,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z',
    });
    return { router, current };
  }

  it('makes the new animation unsaved work, leaving a saved animation’s route', async () => {
    const { router, current } = await openSavedRoute();
    const fixture = TestBed.createComponent(NewAnimationDialog);
    const flow = TestBed.inject(NewAnimationFlow);
    await flow.start();
    await fixture.whenStable();

    await flow.create({ title: 'Fresh', width: 8, height: 8 });

    expect(current.state()).toEqual({ kind: 'unsaved' });
    expect(router.url).toBe('/editor');
  });

  it('does not leave the saved route until the dialog’s own close is acknowledged (wave-15, Group D)', async () => {
    const { router, current } = await openSavedRoute();
    const flow = TestBed.inject(NewAnimationFlow);
    const engine = TestBed.inject(EngineStore);

    const created = flow.create({ title: 'One too many', width: 8, height: 8 });
    // The engine only finishes creating once `create()` is past the point where it sets the
    // dialog closing and starts waiting for its close to be acknowledged.
    await vi.waitFor(() => expect(engine.document()?.title).toBe('One too many'));
    // No `NewAnimationDialog` is mounted here, so nothing has told the flow that the dialog
    // (which `isOpen` alone does not close) is actually gone: leaving already would risk tearing
    // it down mid-dismissal, the very way it was found stuck open.
    expect(router.url).toBe('/editor/a1');
    expect(current.state()).not.toEqual({ kind: 'unsaved' });

    // Left to itself, `create()` stays there: it does not move on without being told the dialog
    // is actually gone.
    const settledOnItsOwn = await Promise.race([
      created.then(() => true),
      new Promise<boolean>((resolve) => setTimeout(() => resolve(false), 100)),
    ]);
    expect(settledOnItsOwn).toBe(false);

    flow.close(); // what the dialog's own `didDismiss` calls, once it is really closed
    await created;

    expect(router.url).toBe('/editor');
    expect(current.state()).toEqual({ kind: 'unsaved' });
  });
});
