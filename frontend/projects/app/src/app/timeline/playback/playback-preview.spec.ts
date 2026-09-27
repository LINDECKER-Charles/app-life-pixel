import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore } from '../../editor/editor-store';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import { PlaybackPreview } from './playback-preview';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;
const NEW_ANIMATION = { title: 'Preview', width: 4, height: 4, layerName: 'Base' };

/** jsdom ships neither: the element's `connectedCallback` and frame loop need a stand-in. */
class FakeIntersectionObserver {
  observe(): void {
    return undefined;
  }

  unobserve(): void {
    return undefined;
  }

  disconnect(): void {
    return undefined;
  }
}

describe('the playback preview', () => {
  let rafSpy: ReturnType<typeof vi.spyOn>;
  let createObjectUrlSpy: ReturnType<typeof vi.spyOn>;
  let revokeObjectUrlSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.stubGlobal('IntersectionObserver', FakeIntersectionObserver);
    // jsdom cannot fetch a blob: URL, and the preview now shows a player error: the export stays
    // loading until a test says how the player took it, with its `load` or `error` event.
    vi.stubGlobal('fetch', () => new Promise<never>(() => undefined));
    rafSpy = vi.spyOn(globalThis, 'requestAnimationFrame').mockReturnValue(0);
    vi.spyOn(globalThis, 'cancelAnimationFrame').mockImplementation(() => undefined);
    // jsdom's Blob/URL.createObjectURL pairing is broken under Vitest; the mission only asks to
    // test the preview through attributes and events, not through real blob internals.
    createObjectUrlSpy = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:mock-url');
    revokeObjectUrlSpy = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => undefined);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    rafSpy.mockRestore();
    createObjectUrlSpy.mockRestore();
    revokeObjectUrlSpy.mockRestore();
    document.body.replaceChildren();
  });

  async function setup() {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    const editor = TestBed.inject(EditorStore);
    await engine.create(NEW_ANIMATION);
    const fixture = TestBed.createComponent(PlaybackPreview);
    fixture.detectChanges();
    return { fixture, engine, editor };
  }

  function button(fixture: ComponentFixture<PlaybackPreview>): HTMLButtonElement {
    return fixture.debugElement.nativeElement.querySelector('button') as HTMLButtonElement;
  }

  function stage(fixture: ComponentFixture<PlaybackPreview>): HTMLElement {
    return fixture.nativeElement.querySelector('.stage') as HTMLElement;
  }

  /** The error the preview shows, once it has failed. */
  async function failure(fixture: ComponentFixture<PlaybackPreview>): Promise<HTMLElement> {
    return vi.waitFor(() => {
      fixture.detectChanges();
      const root = fixture.nativeElement as HTMLElement;
      const alert = root.querySelector<HTMLElement>('[role="alert"]');
      if (!alert) throw new Error('no error shown');
      return alert;
    });
  }

  async function waitForElement(): Promise<HTMLElementTagNameMap['life-pixel']> {
    return vi.waitFor(() => {
      const element = document.querySelector('life-pixel');
      if (!element) throw new Error('the preview has not started');
      return element;
    });
  }

  it('never plays on its own: no element until the button is pressed', async () => {
    const { fixture } = await setup();

    expect(document.querySelector('life-pixel')).toBeNull();
    expect(button(fixture).getAttribute('aria-pressed')).toBe('false');
  });

  it('plays a fresh export from a blob: URL, on the tag holding the active frame', async () => {
    const { fixture, engine, editor } = await setup();
    await engine.apply({ kind: 'addFrame', position: 1, durationMs: 100 });
    const second = engine.document()?.frames[1].id ?? -1;
    await engine.apply({ kind: 'addTag', tag: { name: 'idle', first: 1, last: 1, loop: 'loop' } });
    editor.activeFrame.set(second);

    button(fixture).click();
    const element = await waitForElement();

    expect(element.src.startsWith('blob:')).toBe(true);
    expect(element.tag).toBe('idle');
    fixture.detectChanges();
    expect(button(fixture).getAttribute('aria-pressed')).toBe('true');
  });

  it('is stopped by any later engine change', async () => {
    const { fixture, engine } = await setup();

    button(fixture).click();
    await waitForElement();

    await engine.apply({ kind: 'setTitle', title: 'Changed' });
    fixture.detectChanges();

    expect(document.querySelector('life-pixel')).toBeNull();
    expect(button(fixture).getAttribute('aria-pressed')).toBe('false');
  });

  it('says so, visibly, when the export fails, and plays again once asked', async () => {
    const { fixture } = await setup();
    const engine = TestBed.inject(EDITOR_ENGINE);
    const exportSpy = vi.spyOn(engine, 'export').mockRejectedValueOnce(new Error('refused'));

    button(fixture).click();
    const alert = await failure(fixture);

    expect(alert.textContent).toContain(TEXTS['timeline.playback.failed']);
    expect(document.querySelector('life-pixel')).toBeNull();
    expect(button(fixture).getAttribute('aria-pressed')).toBe('false');

    button(fixture).click();
    await waitForElement();
    fixture.detectChanges();
    expect(exportSpy).toHaveBeenCalledTimes(2);
    expect(fixture.nativeElement.querySelector('[role="alert"]')).toBeNull();
  });

  it('says so, visibly, when the player cannot play the export', async () => {
    const { fixture } = await setup();

    button(fixture).click();
    const element = await waitForElement();
    element.dispatchEvent(new Event('error'));
    const alert = await failure(fixture);

    expect(alert.textContent).toContain(TEXTS['timeline.playback.failed']);
    expect(document.querySelector('life-pixel')).toBeNull();
  });

  it('shows that it is preparing until the player has drawn the first frame', async () => {
    const { fixture } = await setup();
    expect(stage(fixture).textContent).toContain(TEXTS['timeline.playback.idle']);

    button(fixture).click();
    const element = await waitForElement();
    fixture.detectChanges();
    expect(stage(fixture).getAttribute('aria-busy')).toBe('true');
    expect(stage(fixture).textContent).toContain(TEXTS['timeline.playback.preparing']);

    element.dispatchEvent(new Event('load'));
    fixture.detectChanges();
    expect(stage(fixture).getAttribute('aria-busy')).toBe('false');
    expect(stage(fixture).textContent?.trim()).toBe('');
  });

  it('plays even under reduced motion, since the artist asked for it', async () => {
    const { fixture } = await setup();

    button(fixture).click();
    const element = await waitForElement();

    expect(element.getAttribute('motion')).toBe('always');
  });

  it('the button stops the preview while it plays', async () => {
    const { fixture } = await setup();

    button(fixture).click();
    await waitForElement();

    button(fixture).click();
    fixture.detectChanges();

    expect(document.querySelector('life-pixel')).toBeNull();
  });
});
