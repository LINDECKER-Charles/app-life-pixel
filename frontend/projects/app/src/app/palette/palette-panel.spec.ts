import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EditorStore } from '../editor/editor-store';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { PalettePanel } from './palette-panel';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
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
    return fixture.nativeElement.querySelectorAll('.swatch');
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

  it('renders entry 0 as a checkerboard, selectable but with no edit or remove action', async () => {
    await setup();

    const first = swatches()[0];
    const firstItem = fixture.nativeElement.querySelectorAll('.swatch-item')[0];
    expect(first.classList.contains('checkerboard')).toBe(true);
    expect(firstItem.querySelector('.icon-button')).toBeNull();

    first.click();
    fixture.detectChanges();
    expect(TestBed.inject(EditorStore).colorIndex()).toBe(0);
  });

  it('disables Add once the palette reaches the limit', async () => {
    await setup();
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    for (let index = paletteLength(); index < limits.maxPaletteEntries; index++) {
      await engine.apply({ kind: 'addPaletteEntry', color: '#00000000' });
    }
    fixture.detectChanges();

    expect(fixture.nativeElement.querySelector('.add').disabled).toBe(true);
  });

  it('removes an entry, applying removePaletteEntry', async () => {
    await setup();
    const before = paletteLength();
    const removeButtons = fixture.nativeElement.querySelectorAll('.swatch-actions .icon-button');

    removeButtons[1].click();
    await fixture.whenStable();

    expect(paletteLength()).toBe(before - 1);
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
