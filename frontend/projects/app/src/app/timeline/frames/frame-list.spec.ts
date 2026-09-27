import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
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
      providers: [
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
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
    return fixture.debugElement.queryAll(By.css('.head button'))[index]
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

  it('duplicates and then deletes the active frame, the focus kept on the strip', async () => {
    const { fixture, engine, editor } = await setup();
    const first = engine.document()?.frames[0].id ?? -1;
    editor.activeFrame.set(first);

    toolbarButton(fixture, 1).click();
    await fixture.whenStable();
    fixture.detectChanges();
    expect(engine.document()?.frames).toHaveLength(2);

    const deleteButton = toolbarButton(fixture, 2);
    deleteButton.focus();
    deleteButton.click();
    await fixture.whenStable();
    expect(engine.document()?.frames).toHaveLength(1);
    expect(deleteButton.disabled).toBe(true);
    expect(document.activeElement).toBe(thumbnailButtons(fixture)[0]);
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

  it('shows a refused duration as the frame keeps it', async () => {
    const { fixture, engine } = await setup();
    const before = engine.document()?.frames[0].durationMs;
    const field = fixture.debugElement.query(By.css('input[type="number"]'))
      .nativeElement as HTMLInputElement;

    field.value = '12.5';
    field.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    expect(field.value).toBe(String(before));
  });

  it('marks the active frame apart from the rest of the selected range', async () => {
    const { fixture, engine, editor } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    await engine.apply({ kind: 'addFrame', position: 2, durationMs: 100 });
    fixture.detectChanges();

    thumbnailButtons(fixture)[0].click();
    thumbnailButtons(fixture)[2].dispatchEvent(
      new MouseEvent('click', { shiftKey: true, bubbles: true, cancelable: true }),
    );
    await fixture.whenStable();
    fixture.detectChanges();

    const [first, second, third] = thumbnailButtons(fixture);
    expect(editor.activeFrame()).toBe(engine.document()?.frames[0].id);
    expect([first, second, third].map((button) => button.getAttribute('aria-pressed'))).toEqual([
      'true',
      'true',
      'true',
    ]);
    expect(first.getAttribute('aria-current')).toBe('true');
    expect(first.classList).toContain('active');
    for (const other of [second, third]) {
      expect(other.hasAttribute('aria-current')).toBe(false);
      expect(other.classList).not.toContain('active');
    }
  });

  it('numbers the frames from 1, keeping "Frame N" as their names', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    fixture.detectChanges();

    const buttons = thumbnailButtons(fixture);
    expect(buttons.map((button) => button.textContent?.trim())).toEqual(['1', '2']);
    expect(buttons.map((button) => button.getAttribute('aria-label'))).toEqual([
      'Frame 1',
      'Frame 2',
    ]);
  });

  it('never deletes the only frame: Delete frame is disabled', async () => {
    const { fixture } = await setup();

    expect(toolbarButton(fixture, 2).disabled).toBe(true);
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

  // Ionic's router outlet keeps a page it leaves in the DOM: the strip can be there twice.
  it('keeps the duration fields named with a second strip in the page', async () => {
    const { fixture } = await setup();
    const second = TestBed.createComponent(FrameList);
    second.detectChanges();
    // Creating a component detaches the previous one from the page: both go back in.
    document.body.append(fixture.nativeElement, second.nativeElement);

    for (const strip of [fixture, second]) {
      const host = strip.nativeElement as HTMLElement;
      const label = host.querySelector('label');
      expect(label?.control).toBe(host.querySelector('input[type="number"]'));
    }
  });
});
