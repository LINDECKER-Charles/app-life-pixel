import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { PaletteEntryFlow } from './palette-entry-flow';
import { PalettePanel } from './palette-panel';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;
const NEW_ANIMATION = { title: 'Palette', width: 8, height: 8, layerName: 'Base' };

describe('PalettePanel', () => {
  let fixture: ComponentFixture<PalettePanel>;
  let engine: EngineStore;

  function handle(event: KeyboardEvent): void {
    TestBed.inject(Shortcuts).handle(event);
  }

  async function setup(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    document.addEventListener('keydown', handle);
    fixture = TestBed.createComponent(PalettePanel);
    await fixture.whenStable();
  }

  function swatches(): NodeListOf<HTMLButtonElement> {
    return fixture.nativeElement.querySelectorAll('.swatches .swatch');
  }

  /** The button of the selected colour's actions, or of Add, by its visible text. */
  function action(key: 'palette.edit' | 'palette.remove' | 'palette.add'): HTMLButtonElement {
    const buttons = fixture.nativeElement.querySelectorAll('.actions button');
    const found = Array.from<HTMLButtonElement>(buttons).find(
      (button) => button.textContent?.trim() === TEXTS[key],
    );
    if (!found) throw new Error(`no ${key} button`);
    return found;
  }

  function current(): string {
    return (fixture.nativeElement.querySelector('.current') as HTMLElement).textContent ?? '';
  }

  function paletteLength(): number {
    return engine.document()?.palette.length ?? 0;
  }

  afterEach(() => {
    document.removeEventListener('keydown', handle);
    document.body.replaceChildren();
  });

  it('selects a swatch by setting the colour index, with no engine operation', async () => {
    await setup();
    const apply = vi.spyOn(engine, 'apply');

    swatches()[3].click();
    fixture.detectChanges();

    expect(TestBed.inject(EditorStore).colorIndex()).toBe(3);
    expect(apply).not.toHaveBeenCalled();
  });

  it('rings and checks the selected swatch, and names it with its index and hex value', async () => {
    await setup();
    const color = engine.document()?.palette[3];

    swatches()[3].click();
    fixture.detectChanges();

    expect(swatches()[3].getAttribute('aria-pressed')).toBe('true');
    expect(swatches()[3].querySelector('.check')).not.toBeNull();
    expect(swatches()[1].querySelector('.check')).toBeNull();
    expect(current()).toContain('Colour 3');
    expect(current()).toContain(color);
  });

  it('describes entry 0 as the transparency, selectable but neither edited nor removed', async () => {
    await setup();

    const first = swatches()[0];
    expect(first.classList.contains('transparent')).toBe(true);
    expect(first.getAttribute('aria-label')).toBe(TEXTS['palette.swatch_eraser']);

    first.click();
    fixture.detectChanges();
    expect(TestBed.inject(EditorStore).colorIndex()).toBe(0);
    expect(current()).toContain(TEXTS['palette.transparent_hint']);
    expect(action('palette.edit').disabled).toBe(true);
    expect(action('palette.remove').disabled).toBe(true);
  });

  it('opens the edit dialog on the selected colour, from the one Edit button', async () => {
    await setup();
    const open = vi.spyOn(TestBed.inject(PaletteEntryFlow), 'openEdit');
    swatches()[2].click();
    fixture.detectChanges();

    action('palette.edit').click();

    expect(action('palette.edit').getAttribute('aria-label')).toBe('Edit colour 2');
    expect(open).toHaveBeenCalledWith(2, engine.document()?.palette[2]);
  });

  it('disables Add once the palette reaches the limit', async () => {
    await setup();
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    for (let index = paletteLength(); index < limits.maxPaletteEntries; index++) {
      await engine.apply({ kind: 'addPaletteEntry', color: '#00000000' });
    }
    fixture.detectChanges();

    expect(action('palette.add').disabled).toBe(true);
  });

  it('removes the selected entry, applying removePaletteEntry', async () => {
    await setup();
    const before = engine.document()?.palette ?? [];
    swatches()[2].click();
    fixture.detectChanges();

    action('palette.remove').click();
    await fixture.whenStable();

    expect(paletteLength()).toBe(before.length - 1);
    expect(engine.document()?.palette).toEqual(before.filter((_, index) => index !== 2));
  });

  it('reorders with Alt and an arrow key, applying movePaletteEntry', async () => {
    await setup();
    const target = engine.document()?.palette[2];

    swatches()[2].dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowLeft', altKey: true, bubbles: true }),
    );
    await fixture.whenStable();

    expect(engine.document()?.palette[1]).toBe(target);
  });

  it('keeps the moved colour selected and focused, so that Alt with an arrow moves it again', async () => {
    await setup();
    const target = engine.document()?.palette[3];
    swatches()[3].click();
    swatches()[3].focus();
    fixture.detectChanges();

    swatches()[3].dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowLeft', altKey: true, bubbles: true }),
    );
    await fixture.whenStable();

    expect(engine.document()?.palette[2]).toBe(target);
    expect(TestBed.inject(EditorStore).colorIndex()).toBe(2);
    expect(document.activeElement).toBe(swatches()[2]);
  });

  it('never moves entry 0 with Alt and an arrow key', async () => {
    await setup();
    const before = engine.document()?.palette;

    swatches()[1].dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowLeft', altKey: true, bubbles: true }),
    );
    await fixture.whenStable();

    expect(engine.document()?.palette).toEqual(before);
  });

  it('moves through the palette with [ and ]', async () => {
    await setup();
    TestBed.inject(EditorStore).colorIndex.set(2);

    document.dispatchEvent(new KeyboardEvent('keydown', { key: ']', bubbles: true }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '[', bubbles: true }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '[', bubbles: true }));

    expect(TestBed.inject(EditorStore).colorIndex()).toBe(1);
  });

  it('ignores [ and ] typed into a text field', async () => {
    await setup();
    TestBed.inject(EditorStore).colorIndex.set(2);
    const field = document.createElement('input');
    document.body.append(field);

    field.dispatchEvent(new KeyboardEvent('keydown', { key: ']', bubbles: true }));

    expect(TestBed.inject(EditorStore).colorIndex()).toBe(2);
  });
});
