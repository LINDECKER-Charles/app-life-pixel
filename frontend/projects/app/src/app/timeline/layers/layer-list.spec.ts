import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { LayerList } from './layer-list';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Timeline', width: 4, height: 4, layerName: 'Base' };

describe('the layer list', () => {
  async function setup() {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    const fixture = TestBed.createComponent(LayerList);
    fixture.detectChanges();
    return { fixture, engine };
  }

  function rows(fixture: ComponentFixture<LayerList>): readonly HTMLElement[] {
    return fixture.debugElement.queryAll(By.css('.layer')).map((d) => d.nativeElement);
  }

  afterEach(() => document.body.replaceChildren());

  it('adds a layer on top, with the translated default name', async () => {
    const { fixture, engine } = await setup();

    fixture.debugElement.query(By.css('.toolbar button')).nativeElement.click();
    await fixture.whenStable();

    expect(engine.document()?.layers).toHaveLength(2);
    expect(engine.document()?.layers.at(-1)?.name).toBe(en['timeline.layers.new_name']);
  });

  it('deletes a layer', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Extra' });
    fixture.detectChanges();

    rows(fixture)[0].querySelector<HTMLButtonElement>('.delete')?.click();
    await fixture.whenStable();

    expect(engine.document()?.layers).toHaveLength(1);
  });

  it('renames a layer in place, and Escape reverts without a change', async () => {
    const { fixture, engine } = await setup();
    const field = rows(fixture)[0].querySelector<HTMLInputElement>('input[type="text"]');
    if (!field) throw new Error('no name field');

    field.value = 'Renamed';
    field.dispatchEvent(new Event('change'));
    await fixture.whenStable();
    expect(engine.document()?.layers[0].name).toBe('Renamed');

    field.value = 'Ignored';
    field.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await fixture.whenStable();
    expect(field.value).toBe('Renamed');
    expect(engine.document()?.layers[0].name).toBe('Renamed');
  });

  it('toggles visibility, reflected in aria-pressed', async () => {
    const { fixture, engine } = await setup();
    const button = rows(fixture)[0].querySelector<HTMLButtonElement>('.visibility');
    if (!button) throw new Error('no visibility button');
    expect(button.getAttribute('aria-pressed')).toBe('true');

    button.click();
    await fixture.whenStable();

    expect(engine.document()?.layers[0].visible).toBe(false);
    fixture.detectChanges();
    expect(button.getAttribute('aria-pressed')).toBe('false');
  });

  it('reorders with Alt and the arrow keys', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    fixture.detectChanges();
    const topId = engine.document()?.layers[1].id;
    const baseId = engine.document()?.layers[0].id;

    rows(fixture)[0].dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'ArrowDown',
        altKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    await fixture.whenStable();

    expect(engine.document()?.layers[0].id).toBe(topId);
    expect(engine.document()?.layers[1].id).toBe(baseId);
  });

  it('reorders by drag and drop', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    fixture.detectChanges();
    const topId = engine.document()?.layers[1].id;
    const items = rows(fixture);

    items[0].dispatchEvent(new Event('dragstart', { bubbles: true }));
    items[1].dispatchEvent(new Event('drop', { bubbles: true }));
    await fixture.whenStable();

    expect(engine.document()?.layers[0].id).toBe(topId);
  });
});
