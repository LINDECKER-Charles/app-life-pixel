import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { TagBars } from './tag-bars';
import { TagDialogState } from './tag-dialog-state';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;
const NEW_ANIMATION = { title: 'Tags', width: 4, height: 4, layerName: 'Base' };

describe('the tag bars', () => {
  async function setup(): Promise<{ fixture: ComponentFixture<TagBars>; engine: EngineStore }> {
    TestBed.configureTestingModule({
      providers: [
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    await engine.apply({ kind: 'addTag', tag: { name: 'walk', first: 0, last: 1, loop: 'loop' } });
    await engine.apply({ kind: 'addTag', tag: { name: 'blink', first: 2, last: 2, loop: 'once' } });
    const fixture = TestBed.createComponent(TagBars);
    fixture.detectChanges();
    return { fixture, engine };
  }

  function bars(fixture: ComponentFixture<TagBars>): HTMLButtonElement[] {
    return Array.from(fixture.nativeElement.querySelectorAll('.bar'));
  }

  function addButton(fixture: ComponentFixture<TagBars>): HTMLButtonElement {
    return fixture.nativeElement.querySelector('.head button');
  }

  afterEach(() => document.body.replaceChildren());

  it('spans each tag over its frames, named "Edit tag “…”", with its range in words', async () => {
    const { fixture } = await setup();
    const [walk, blink] = bars(fixture);

    expect(walk.getAttribute('aria-label')).toBe('Edit tag “walk”');
    expect(walk.style.gridColumn).toBe('1 / 3');
    expect(blink.style.gridColumn).toBe('3 / 4');
    const summary = document.getElementById(walk.getAttribute('aria-describedby') ?? '');
    expect(summary?.textContent?.trim()).toBe('Frames 1 to 2, looping.');
  });

  it('says in words that a tag plays once', async () => {
    const { fixture } = await setup();
    const [walk, blink] = bars(fixture);

    expect(blink.textContent).toContain(TEXTS['timeline.tags.once']);
    expect(walk.textContent).not.toContain(TEXTS['timeline.tags.once']);
  });

  it('adds a tag over the frame selection only, and opens a bar for editing', async () => {
    const { fixture, engine } = await setup();
    const dialog = TestBed.inject(TagDialogState);
    expect(addButton(fixture).disabled).toBe(true);

    TestBed.inject(EditorStore).frameSelection.set({ first: 1, last: 2 });
    fixture.detectChanges();
    addButton(fixture).click();
    expect(dialog.current()).toEqual({ kind: 'add', first: 1, last: 2 });

    bars(fixture)[0].click();
    expect(dialog.current()).toEqual({ kind: 'edit', tag: engine.document()?.tags[0] });
  });
});
