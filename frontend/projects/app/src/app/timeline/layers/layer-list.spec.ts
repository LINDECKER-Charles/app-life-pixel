import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { LayerList } from './layer-list';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;
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

  function part<T extends HTMLElement>(row: HTMLElement, selector: string): T {
    const found = row.querySelector<T>(selector);
    if (!found) throw new Error(`no ${selector} in the row`);
    return found;
  }

  function key(name: string): KeyboardEvent {
    return new KeyboardEvent('keydown', { key: name, bubbles: true, cancelable: true });
  }

  afterEach(() => document.body.replaceChildren());

  it('adds a layer on top, with the translated default name', async () => {
    const { fixture, engine } = await setup();

    fixture.debugElement.query(By.css('.toolbar button')).nativeElement.click();
    await fixture.whenStable();

    expect(engine.document()?.layers).toHaveLength(2);
    expect(engine.document()?.layers.at(-1)?.name).toBe(en['timeline.layers.new_name']);
  });

  it('deletes a layer, the focus going to the layer then active', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Extra' });
    fixture.detectChanges();

    const deleteButton = part<HTMLButtonElement>(rows(fixture)[0], '.delete');
    deleteButton.focus();
    deleteButton.click();
    await fixture.whenStable();

    expect(engine.document()?.layers).toHaveLength(1);
    expect(document.activeElement).toBe(part(rows(fixture)[0], '.select'));
  });

  it('makes the clicked layer the active one, marked by aria-current and in words', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    fixture.detectChanges();
    const [base, added] = engine.document()?.layers.map((layer) => layer.id) ?? [];
    const editor = TestBed.inject(EditorStore);
    expect(editor.activeLayer()).toBe(base);

    part(rows(fixture)[0], '.select').click();
    fixture.detectChanges();

    expect(editor.activeLayer()).toBe(added);
    const [top, bottom] = rows(fixture);
    expect(part(top, '.select').getAttribute('aria-current')).toBe('true');
    expect(top.textContent).toContain(TEXTS['timeline.layers.active']);
    expect(part(bottom, '.select').hasAttribute('aria-current')).toBe(false);
    expect(bottom.textContent).not.toContain(TEXTS['timeline.layers.active']);
  });

  it('shows the name as text, and renames it on demand: Enter applies the new name', async () => {
    const { fixture, engine } = await setup();
    expect(rows(fixture)[0].querySelector('input')).toBeNull();

    part(rows(fixture)[0], '.rename').click();
    fixture.detectChanges();
    const field = part<HTMLInputElement>(rows(fixture)[0], 'input[type="text"]');
    field.value = 'Renamed';
    field.dispatchEvent(key('Enter'));
    await fixture.whenStable();

    expect(engine.document()?.layers[0].name).toBe('Renamed');
    expect(rows(fixture)[0].querySelector('input')).toBeNull();
    expect(part(rows(fixture)[0], '.select').textContent).toContain('Renamed');
  });

  it('reverts a rename with Escape, without a change', async () => {
    const { fixture, engine } = await setup();
    const apply = vi.spyOn(engine, 'apply');

    part(rows(fixture)[0], '.select').dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
    fixture.detectChanges();
    const field = part<HTMLInputElement>(rows(fixture)[0], 'input[type="text"]');
    field.value = 'Ignored';
    field.dispatchEvent(key('Escape'));
    field.dispatchEvent(new Event('blur'));
    await fixture.whenStable();

    expect(apply).not.toHaveBeenCalled();
    expect(engine.document()?.layers[0].name).toBe('Base');
    expect(rows(fixture)[0].querySelector('input')).toBeNull();
  });

  it('never deletes the only layer: its delete button is disabled', async () => {
    const { fixture } = await setup();

    expect(part<HTMLButtonElement>(rows(fixture)[0], '.delete').disabled).toBe(true);
  });

  it('toggles visibility, reflected in aria-pressed and, once hidden, in words', async () => {
    const { fixture, engine } = await setup();
    const button = part<HTMLButtonElement>(rows(fixture)[0], '.visibility');
    expect(button.getAttribute('aria-pressed')).toBe('true');

    button.click();
    await fixture.whenStable();

    expect(engine.document()?.layers[0].visible).toBe(false);
    fixture.detectChanges();
    expect(button.getAttribute('aria-pressed')).toBe('false');
    expect(rows(fixture)[0].textContent).toContain(TEXTS['timeline.layers.hidden']);
  });

  it('reorders with Alt and the arrow keys', async () => {
    const { fixture, engine } = await setup();
    await engine.apply({ kind: 'addLayer', position: 1, name: 'Top' });
    fixture.detectChanges();
    const topId = engine.document()?.layers[1].id;
    const baseId = engine.document()?.layers[0].id;

    part(rows(fixture)[0], '.select').dispatchEvent(
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
