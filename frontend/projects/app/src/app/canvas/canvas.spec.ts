import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { EngineStore } from '../engine/engine-store';
import type { EditOperation, Point, RenderRequest } from '../engine/engine-types';
import { Canvas } from './canvas';
import { originFor } from './geometry/canvas-geometry';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const ZOOM = 8;
const SIDE = 8;

describe('Canvas', () => {
  let fixture: ComponentFixture<Canvas>;
  let store: EditorStore;
  let applied: EditOperation[];
  let renders: RenderRequest[];

  async function open(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create({ title: 'Work', width: SIDE, height: SIDE, layerName: 'Base' });
    store = TestBed.inject(EditorStore);
    applied = [];
    vi.spyOn(engine, 'apply').mockImplementation(async (operation) => {
      applied.push(operation);
    });
    renders = [];
    const realRender = TestBed.inject(EDITOR_ENGINE).render.bind(TestBed.inject(EDITOR_ENGINE));
    vi.spyOn(TestBed.inject(EDITOR_ENGINE), 'render').mockImplementation((request) => {
      renders.push(request);
      return realRender(request);
    });
    fixture = TestBed.createComponent(Canvas);
    await fixture.whenStable();
  }

  afterEach(() => document.body.replaceChildren());

  function host(): HTMLDivElement {
    return fixture.nativeElement.querySelector('.workspace');
  }

  function view(): HTMLCanvasElement {
    return fixture.nativeElement.querySelector('.view');
  }

  // The view has no layout in jsdom, so it stays 0×0: the content's origin comes from that
  // alone, the default zoom and no pan, matching `currentViewport()`.
  function clientForPixel(pixel: Point): Point {
    const origin = originFor({
      view: { width: 0, height: 0 },
      content: { width: SIDE, height: SIDE },
      zoom: ZOOM,
      pan: { x: 0, y: 0 },
    });
    return { x: origin.x + pixel.x * ZOOM + 1, y: origin.y + pixel.y * ZOOM + 1 };
  }

  function pointer(
    type: string,
    pixel: Point,
    options: { readonly pointerId: number } & PointerEventInit = { pointerId: 1 },
  ): PointerEvent {
    const client = clientForPixel(pixel);
    return new PointerEvent(type, {
      clientX: client.x,
      clientY: client.y,
      button: 0,
      bubbles: true,
      ...options,
    });
  }

  function key(type: string, init: KeyboardEventInit): KeyboardEvent {
    return new KeyboardEvent(type, { bubbles: true, cancelable: true, ...init });
  }

  it('a pencil drag sends previews then one paintStroke with the dragged points', async () => {
    await open();
    const layer = store.activeLayer();
    const frame = store.activeFrame();

    view().dispatchEvent(pointer('pointerdown', { x: 0, y: 0 }));
    view().dispatchEvent(pointer('pointermove', { x: 1, y: 0 }));
    await fixture.whenStable();
    view().dispatchEvent(pointer('pointerup', { x: 1, y: 0 }));
    await fixture.whenStable();

    const points = [
      { x: 0, y: 0 },
      { x: 1, y: 0 },
    ];
    expect(renders).toContainEqual({
      frame,
      preview: { kind: 'paintStroke', layer, frame, points, index: store.colorIndex() },
    });
    expect(applied).toEqual([
      { kind: 'paintStroke', layer, frame, points, index: store.colorIndex() },
    ]);
  });

  it('a line sends its operation, constrained to 45° with Shift', async () => {
    await open();
    store.tool.set('line');

    view().dispatchEvent(pointer('pointerdown', { x: 0, y: 0 }));
    view().dispatchEvent(pointer('pointermove', { x: 3, y: 1 }, { pointerId: 1, shiftKey: true }));
    view().dispatchEvent(pointer('pointerup', { x: 3, y: 1 }, { pointerId: 1, shiftKey: true }));
    await fixture.whenStable();

    expect(applied).toEqual([
      expect.objectContaining({ kind: 'line', from: { x: 0, y: 0 }, to: { x: 3, y: 0 } }),
    ]);
  });

  it('a rectangle sends its operation', async () => {
    await open();
    store.tool.set('rectangle');

    view().dispatchEvent(pointer('pointerdown', { x: 0, y: 0 }));
    view().dispatchEvent(pointer('pointermove', { x: 2, y: 3 }));
    view().dispatchEvent(pointer('pointerup', { x: 2, y: 3 }));
    await fixture.whenStable();

    expect(applied).toEqual([
      expect.objectContaining({ kind: 'rectangle', from: { x: 0, y: 0 }, to: { x: 2, y: 3 } }),
    ]);
  });

  it('fill sends its operation on a click', async () => {
    await open();
    store.tool.set('fill');

    view().dispatchEvent(pointer('pointerdown', { x: 2, y: 2 }));
    await fixture.whenStable();

    expect(applied).toEqual([expect.objectContaining({ kind: 'fill', at: { x: 2, y: 2 } })]);
  });

  it('dragging inside a selection sends a moveSelection operation', async () => {
    await open();
    store.tool.set('select');
    store.selection.set({ x: 1, y: 1, width: 2, height: 2 });

    view().dispatchEvent(pointer('pointerdown', { x: 1, y: 1 }));
    view().dispatchEvent(pointer('pointermove', { x: 2, y: 1 }));
    view().dispatchEvent(pointer('pointerup', { x: 2, y: 1 }));
    await fixture.whenStable();

    expect(applied).toEqual([
      expect.objectContaining({
        kind: 'moveSelection',
        area: { x: 1, y: 1, width: 2, height: 2 },
        offset: { x: 1, y: 0 },
      }),
    ]);
    expect(store.selection()).toEqual({ x: 2, y: 1, width: 2, height: 2 });
  });

  it('Escape cancels the gesture in progress: nothing is applied', async () => {
    await open();
    store.tool.set('line');

    view().dispatchEvent(pointer('pointerdown', { x: 0, y: 0 }));
    view().dispatchEvent(pointer('pointermove', { x: 3, y: 3 }));
    host().dispatchEvent(key('keydown', { key: 'Escape' }));
    view().dispatchEvent(pointer('pointerup', { x: 3, y: 3 }));
    await fixture.whenStable();

    expect(applied).toEqual([]);
  });

  it('draws a pixel with the keyboard alone', async () => {
    await open();
    const layer = store.activeLayer();
    const frame = store.activeFrame();

    host().dispatchEvent(key('keydown', { key: 'ArrowRight' }));
    host().dispatchEvent(key('keydown', { key: 'ArrowDown' }));
    host().dispatchEvent(key('keydown', { key: 'Enter' }));
    await fixture.whenStable();

    expect(applied).toEqual([
      { kind: 'paintStroke', layer, frame, points: [{ x: 1, y: 1 }], index: store.colorIndex() },
    ]);
  });

  it('the "+" and "-" shortcuts zoom in and out', async () => {
    await open();

    TestBed.inject(Shortcuts).handle(key('keydown', { key: '+' }));
    expect(store.zoom()).toBe(ZOOM + 1);

    TestBed.inject(Shortcuts).handle(key('keydown', { key: '-' }));
    expect(store.zoom()).toBe(ZOOM);
  });
});
