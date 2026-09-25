import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { PlaybackPreview } from './playback-preview';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
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

  it('the button stops the preview while it plays', async () => {
    const { fixture } = await setup();

    button(fixture).click();
    await waitForElement();

    button(fixture).click();
    fixture.detectChanges();

    expect(document.querySelector('life-pixel')).toBeNull();
  });
});
