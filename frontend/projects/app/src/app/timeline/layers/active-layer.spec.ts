import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { Canvas } from '../../canvas/canvas';
import { EditorStore } from '../../editor/editor-store';
import { EDITOR_ENGINE } from '../../engine/editor-engine';
import { EngineStore } from '../../engine/engine-store';
import type { EditOperation } from '../../engine/engine-types';
import { LayerList } from './layer-list';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Layers', width: 8, height: 8, layerName: 'Base' };

// Plan C8: choosing a layer in the list is what the next drawing lands on, the mock engine
// standing in for the real one.
describe('the active layer, chosen in the layer list', () => {
  let layers: ComponentFixture<LayerList>;
  let canvas: ComponentFixture<Canvas>;
  let applied: EditOperation[];

  async function setup(): Promise<EngineStore> {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    applied = [];
    const mock = TestBed.inject(EDITOR_ENGINE);
    const apply = mock.apply.bind(mock);
    vi.spyOn(mock, 'apply').mockImplementation((operation) => {
      applied.push(operation);
      return apply(operation);
    });
    layers = TestBed.createComponent(LayerList);
    canvas = TestBed.createComponent(Canvas);
    layers.detectChanges();
    await canvas.whenStable();
    return engine;
  }

  afterEach(() => document.body.replaceChildren());

  /** The layer row's name button, by the layer's name. */
  function layerButton(name: string): HTMLButtonElement {
    const buttons = layers.nativeElement.querySelectorAll('.select') as NodeListOf<HTMLElement>;
    const found = Array.from(buttons).find((button) => button.textContent?.includes(name));
    if (!found) throw new Error(`no layer named ${name}`);
    return found as HTMLButtonElement;
  }

  /** Draws the pixel under the canvas's keyboard cursor, as a keyboard user does. */
  async function drawWithTheKeyboard(): Promise<void> {
    const surface = canvas.nativeElement.querySelector('[role="application"]') as HTMLElement;
    surface.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    await canvas.whenStable();
  }

  it('draws on the topmost layer until another is clicked, then on that one', async () => {
    const engine = await setup();
    const [base, top] = engine.document()?.layers.map((layer) => layer.id) ?? [];

    await drawWithTheKeyboard();
    expect(applied.at(-1)).toMatchObject({ kind: 'paintStroke', layer: top });

    layerButton('Base').click();
    layers.detectChanges();
    expect(TestBed.inject(EditorStore).activeLayer()).toBe(base);
    expect(layerButton('Base').getAttribute('aria-current')).toBe('true');

    await drawWithTheKeyboard();
    expect(applied.at(-1)).toMatchObject({ kind: 'paintStroke', layer: base });
  });
});
