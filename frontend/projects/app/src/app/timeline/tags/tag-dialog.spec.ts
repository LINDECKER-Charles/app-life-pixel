import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { TagDialog } from './tag-dialog';
import { TagDialogState } from './tag-dialog-state';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Timeline', width: 4, height: 4, layerName: 'Base' };

describe('the tag dialog', () => {
  async function setup() {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    const state = TestBed.inject(TagDialogState);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    const fixture = TestBed.createComponent(TagDialog);
    fixture.detectChanges();
    return { fixture, engine, state };
  }

  // `ion-modal` teleports its content out of `lp-tag-dialog`'s subtree once it presents.
  async function form(): Promise<HTMLFormElement> {
    return vi.waitFor(() => {
      const found = document.querySelector<HTMLFormElement>('ion-modal form');
      if (!found) throw new Error('the dialog is not open');
      return found;
    });
  }

  afterEach(() => document.body.replaceChildren());

  it('adds a tag over the frame selection', async () => {
    const { fixture, engine, state } = await setup();
    state.open({ kind: 'add', first: 0, last: 1 });
    fixture.detectChanges();
    const dialog = await form();
    const name = dialog.querySelector<HTMLInputElement>('#tag-name');
    if (!name) throw new Error('no name field');

    name.value = 'walk';
    name.dispatchEvent(new Event('input'));
    fixture.detectChanges();
    dialog.querySelector<HTMLButtonElement>('button[type="submit"]')?.click();
    await fixture.whenStable();

    expect(engine.document()?.tags).toEqual([{ name: 'walk', first: 0, last: 1, loop: 'loop' }]);
    expect(state.current()).toBeNull();
  });

  it('renames and changes an existing tag', async () => {
    const { fixture, engine, state } = await setup();
    await engine.apply({ kind: 'addTag', tag: { name: 'walk', first: 0, last: 1, loop: 'loop' } });
    const tag = engine.document()?.tags[0];
    if (!tag) throw new Error('no tag');
    state.open({ kind: 'edit', tag });
    fixture.detectChanges();
    const dialog = await form();
    const name = dialog.querySelector<HTMLInputElement>('#tag-name');
    const once = dialog.querySelector<HTMLInputElement>('input[value="once"]');
    if (!name || !once) throw new Error('missing fields');

    name.value = 'run';
    name.dispatchEvent(new Event('input'));
    once.dispatchEvent(new Event('change'));
    dialog.querySelector<HTMLButtonElement>('button[type="submit"]')?.click();
    await fixture.whenStable();

    expect(engine.document()?.tags).toEqual([{ name: 'run', first: 0, last: 1, loop: 'once' }]);
  });

  it('deletes a tag', async () => {
    const { fixture, engine, state } = await setup();
    await engine.apply({ kind: 'addTag', tag: { name: 'walk', first: 0, last: 1, loop: 'loop' } });
    const tag = engine.document()?.tags[0];
    if (!tag) throw new Error('no tag');
    state.open({ kind: 'edit', tag });
    fixture.detectChanges();
    const dialog = await form();

    dialog.querySelector<HTMLButtonElement>('.danger')?.click();
    await fixture.whenStable();

    expect(engine.document()?.tags).toEqual([]);
    expect(state.current()).toBeNull();
  });

  it('refuses a duplicate name with the engine error, and keeps the dialog open', async () => {
    const { fixture, engine, state } = await setup();
    await engine.apply({ kind: 'addTag', tag: { name: 'walk', first: 0, last: 1, loop: 'loop' } });
    state.open({ kind: 'add', first: 2, last: 2 });
    fixture.detectChanges();
    const dialog = await form();
    const name = dialog.querySelector<HTMLInputElement>('#tag-name');
    if (!name) throw new Error('no name field');

    name.value = 'walk';
    name.dispatchEvent(new Event('input'));
    fixture.detectChanges();
    dialog.querySelector<HTMLButtonElement>('button[type="submit"]')?.click();
    await fixture.whenStable();

    expect(engine.document()?.tags).toHaveLength(1);
    expect(state.current()).not.toBeNull();
    expect(engine.notifications()[0]?.key).toBe('errors.document.tag');
  });
});
