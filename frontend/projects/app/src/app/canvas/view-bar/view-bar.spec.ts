import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore, type OnionSkin } from '../../editor/editor-store';
import { Shortcuts } from '../../editor/shortcuts';
import { EngineStore } from '../../engine/engine-store';
import type { Point } from '../../engine/engine-types';
import { Timeline } from '../../timeline/timeline';
import { ShortcutsHelpState } from '../../tools/shortcuts-help-state';
import { buildToolShortcuts } from '../../tools/tool-shortcuts';
import { Canvas } from '../canvas';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'View', width: 8, height: 8, layerName: 'Base' };
const ZOOM = 8;

/** What the view bar changes: the store's view state, compared after a button and a key. */
interface ViewState {
  readonly zoom: number;
  readonly pan: Point;
  readonly showGrid: boolean;
  readonly onionSkin: OnionSkin;
}

describe('ViewBar', () => {
  let fixture: ComponentFixture<Canvas>;
  let store: EditorStore;

  // The canvas holds the view bar and shares its viewport; the tool bar registers Shift G, the
  // timeline O, as on the editor page.
  async function open(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    store = TestBed.inject(EditorStore);
    const help = TestBed.inject(ShortcutsHelpState);
    TestBed.inject(Shortcuts).register(buildToolShortcuts({ editor: store, engine, help }));
    TestBed.createComponent(Timeline).detectChanges();
    fixture = TestBed.createComponent(Canvas);
    await fixture.whenStable();
  }

  afterEach(() => {
    fixture.destroy();
    document.body.replaceChildren();
  });

  function bar(): HTMLElement {
    return fixture.nativeElement.querySelector('lp-view-bar');
  }

  function button(name: string): HTMLButtonElement {
    const found = [...bar().querySelectorAll('button')].find(
      (candidate) => candidate.getAttribute('aria-label') === name,
    );
    if (!found) throw new Error(`No button named ${name}`);
    return found;
  }

  function press(init: KeyboardEventInit): void {
    TestBed.inject(Shortcuts).handle(new KeyboardEvent('keydown', { bubbles: true, ...init }));
  }

  function state(): ViewState {
    return {
      zoom: store.zoom(),
      pan: store.pan(),
      showGrid: store.showGrid(),
      onionSkin: store.onionSkin(),
    };
  }

  /** The state after the key, then after the button from the same start: they must agree. */
  function compare(key: KeyboardEventInit, name: string): [ViewState, ViewState] {
    const start = state();
    press(key);
    const byKey = state();
    store.zoom.set(start.zoom);
    store.pan.set(start.pan);
    store.showGrid.set(start.showGrid);
    store.onionSkin.set(start.onionSkin);
    button(name).click();
    return [byKey, state()];
  }

  it('zooms in, out and to fit as +, - and 0 do', async () => {
    await open();
    store.pan.set({ x: 12, y: -4 });

    const [zoomInByKey, zoomIn] = compare({ key: '+' }, 'Zoom in');
    expect(zoomIn).toEqual(zoomInByKey);
    expect(zoomIn.zoom).toBe(ZOOM + 1);

    const [zoomOutByKey, zoomOut] = compare({ key: '-' }, 'Zoom out');
    expect(zoomOut).toEqual(zoomOutByKey);
    expect(zoomOut.zoom).toBe(ZOOM);

    const [fitByKey, fit] = compare({ key: '0' }, 'Fit zoom');
    expect(fit).toEqual(fitByKey);
    expect(fit.pan).toEqual({ x: 0, y: 0 });
  });

  it('switches the grid as Shift G does, and the onion skin as O does', async () => {
    await open();

    const [gridByKey, grid] = compare({ key: 'G', shiftKey: true }, 'Grid');
    expect(grid).toEqual(gridByKey);
    expect(grid.showGrid).toBe(false);

    const [onionByKey, onion] = compare({ key: 'o' }, 'Onion skin');
    expect(onion).toEqual(onionByKey);
    expect(onion.onionSkin.enabled).toBe(true);
  });

  it('shows the zoom in percent and each switch by aria-pressed', async () => {
    await open();

    store.zoom.set(12);
    store.toggleOnionSkin();
    await fixture.whenStable();

    expect(bar().querySelector('output')?.textContent?.trim()).toBe('1200%');
    expect(button('Grid').getAttribute('aria-pressed')).toBe('true');
    expect(button('Onion skin').getAttribute('aria-pressed')).toBe('true');
  });

  it('sets how many frames the onion skin shows before and after', async () => {
    await open();
    const [before, after] = [...bar().querySelectorAll('select')];

    before.value = '2';
    before.dispatchEvent(new Event('change'));
    after.value = '0';
    after.dispatchEvent(new Event('change'));

    expect(store.onionSkin()).toEqual({ enabled: false, before: 2, after: 0 });
  });

  it('shows the pixel under the pointer, the active frame and layer, and the tool', async () => {
    await open();
    const view: HTMLCanvasElement = fixture.nativeElement.querySelector('.view');
    // The view has no layout in jsdom (0×0): the 8×8 content's origin sits at -32, -32.
    const origin = -(NEW_ANIMATION.width * ZOOM) / 2;

    view.dispatchEvent(
      new PointerEvent('pointermove', { clientX: origin + 2 * ZOOM + 1, clientY: origin + 5 }),
    );
    await fixture.whenStable();
    expect(bar().textContent).toContain('Pixel 2, 0');
    expect(bar().textContent).toContain('Frame 1 of 1');
    expect(bar().textContent).toContain('Layer: Base');
    expect(bar().textContent).toContain('Pencil');

    view.dispatchEvent(new PointerEvent('pointerleave'));
    await fixture.whenStable();
    expect(bar().textContent).toContain('Pixel —');
  });

  // Its width is reserved in CSS, so the bar only keeps its height if the reading stays in that
  // same field: no reading may add, drop or restyle a part of the status line.
  it('keeps the pixel reading in the same field as the pointer comes and goes', async () => {
    await open();
    const view: HTMLCanvasElement = fixture.nativeElement.querySelector('.view');
    const status = (): HTMLElement => bar().querySelector('.status') as HTMLElement;
    const shape = (): string[] => [...status().children].map((part) => part.className);
    const field = status().querySelector('.position');
    const before = shape();

    view.dispatchEvent(new PointerEvent('pointermove', { clientX: 0, clientY: 0 }));
    await fixture.whenStable();
    expect(field?.textContent?.trim()).toMatch(/^Pixel \d+, \d+$/);
    expect(status().querySelector('.position')).toBe(field);
    expect(shape()).toEqual(before);

    view.dispatchEvent(new PointerEvent('pointerleave'));
    await fixture.whenStable();
    expect(status().querySelector('.position')).toBe(field);
    expect(shape()).toEqual(before);
  });
});
