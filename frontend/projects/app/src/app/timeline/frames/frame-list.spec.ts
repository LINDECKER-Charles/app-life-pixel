import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { FrameList } from './frame-list';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Timeline', width: 4, height: 4, layerName: 'Base' };

describe('the frame strip', () => {
  async function setup() {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    const editor = TestBed.inject(EditorStore);
    await engine.create(NEW_ANIMATION);
    const fixture = TestBed.createComponent(FrameList);
    fixture.detectChanges();
    return { fixture, engine, editor };
  }

  function toolbarButton(fixture: ComponentFixture<FrameList>, index: number): HTMLButtonElement {
    return fixture.debugElement.queryAll(By.css('.toolbar button'))[index]
      .nativeElement as HTMLButtonElement;
  }

  function thumbnailButtons(fixture: ComponentFixture<FrameList>): readonly HTMLButtonElement[] {
    return fixture.debugElement
      .queryAll(By.css('.thumbnail-button'))
      .map((debug) => debug.nativeElement as HTMLButtonElement);
  }

  afterEach(() => document.body.replaceChildren());

  it('adds a frame after the active frame', async () => {
    const { fixture, engine, editor } = await setup();
    const first = engine.document()?.frames[0].id ?? -1;
    editor.activeFrame.set(first);

    toolbarButton(fixture, 0).click();
    await fixture.whenStable();

    expect(engine.document()?.frames).toHaveLength(2);
    expect(engine.document()?.frames[1].durationMs).toBe(engine.limits()?.defaultFrameDurationMs);
  });

  it('duplicates and then deletes the active frame', async () => {
    const { fixture, engine, editor } = await setup();
    const first = engine.document()?.frames[0].id ?? -1;
    editor.activeFrame.set(first);

    toolbarButton(fixture, 1).click();
    await fixture.whenStable();
    expect(engine.document()?.frames).toHaveLength(2);

    toolbarButton(fixture, 2).click();
    await fixture.whenStable();
    expect(engine.document()?.frames).toHaveLength(1);
  });

  it('sends a bounded duration change', async () => {
    const { fixture, engine } = await setup();
    const frame = engine.document()?.frames[0].id ?? -1;
    const field = fixture.debugElement.query(By.css('input[type="number"]'))
      .nativeElement as HTMLInputElement;

    field.value = '250';
    field.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    expect(engine.document()?.frames.find((candidate) => candidate.id === frame)?.durationMs).toBe(
      250,
    );
  });

  it('selects a frame on click, and extends the selection on shift-click', async () => {
    const { fixture, engine, editor } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    fixture.detectChanges();
    const buttons = thumbnailButtons(fixture);

    buttons[0].click();
    await fixture.whenStable();
    expect(editor.frameSelection()).toEqual({ first: 0, last: 0 });

    thumbnailButtons(fixture)[2].dispatchEvent(
      new MouseEvent('click', { shiftKey: true, bubbles: true, cancelable: true }),
    );
    await fixture.whenStable();
    expect(editor.frameSelection()).toEqual({ first: 0, last: 2 });
  });

  it('reorders with Alt and the arrow keys', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 200 });
    fixture.detectChanges();
    const firstId = engine.document()?.frames[0].id;

    thumbnailButtons(fixture)[0].dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'ArrowRight',
        altKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    await fixture.whenStable();

    expect(engine.document()?.frames[1].id).toBe(firstId);
  });

  it('reorders by drag and drop', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 200 });
    fixture.detectChanges();
    const firstId = engine.document()?.frames[0].id;
    const items = fixture.debugElement.queryAll(By.css('.frame')).map((d) => d.nativeElement);

    items[0].dispatchEvent(new Event('dragstart', { bubbles: true }));
    items[1].dispatchEvent(new Event('drop', { bubbles: true }));
    await fixture.whenStable();

    expect(engine.document()?.frames[1].id).toBe(firstId);
  });
});
