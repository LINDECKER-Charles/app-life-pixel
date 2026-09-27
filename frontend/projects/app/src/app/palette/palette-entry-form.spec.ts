import { ComponentRef, importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EngineStore } from '../engine/engine-store';
import { PaletteEntryFlow, type PaletteEntryMode } from './palette-entry-flow';
import { PaletteEntryForm } from './palette-entry-form';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
/** The catalogue by any key, those of i18n-pending/ included once merged. */
const TEXTS: Readonly<Record<string, string>> = en;
const NEW_ANIMATION = { title: 'Entry', width: 8, height: 8, layerName: 'Base' };
/** The fields, by the i18n key of their label. */
const COLOR = 'palette.dialog.color';
const ALPHA = 'palette.dialog.alpha';
const HEX = 'palette.dialog.hex';

describe('PaletteEntryForm', () => {
  let fixture: ComponentFixture<PaletteEntryForm>;
  let engine: EngineStore;

  async function setup(mode: PaletteEntryMode): Promise<void> {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    fixture = TestBed.createComponent(PaletteEntryForm);
    (fixture.componentRef as ComponentRef<PaletteEntryForm>).setInput('mode', mode);
    await fixture.whenStable();
  }

  /** The input its label names, as a person finds it: by the label's text. */
  function field(labelKey: string): HTMLInputElement {
    const labels = fixture.nativeElement.querySelectorAll('label');
    const label = Array.from<HTMLLabelElement>(labels).find(
      (candidate) => candidate.textContent?.trim() === TEXTS[labelKey],
    );
    const control = label?.control;
    if (!(control instanceof HTMLInputElement)) throw new Error(`no field labelled ${labelKey}`);
    return control;
  }

  it('keeps the colour input, the alpha slider and the hex field in sync from the hex field', async () => {
    await setup({ kind: 'add' });

    const hex = field(HEX);
    hex.value = '#1a2b3cab';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(field(COLOR).value).toBe('#1a2b3c');
    expect(field(ALPHA).valueAsNumber).toBe(0xab);
  });

  it('keeps the fields in sync from the colour input and the alpha slider', async () => {
    await setup({ kind: 'add' });

    const color = field(COLOR);
    color.value = '#334455';
    color.dispatchEvent(new Event('input'));
    const alpha = field(ALPHA);
    alpha.value = '128';
    alpha.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(field(HEX).value).toBe('#33445580');
  });

  it('marks an invalid hex value invalid, without touching the other fields', async () => {
    await setup({ kind: 'add' });

    const hex = field(HEX);
    hex.value = 'not a colour';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(hex.getAttribute('aria-invalid')).toBe('true');
    expect(fixture.nativeElement.querySelector('button[type="submit"]').disabled).toBe(true);
    expect(fixture.nativeElement.textContent).toContain(TEXTS['palette.dialog.hex_error']);
    const description: HTMLElement | null = fixture.nativeElement.querySelector(
      `[id="${hex.getAttribute('aria-describedby')}"]`,
    );
    expect(description?.textContent?.trim()).toBe(TEXTS['palette.dialog.hex_error']);
  });

  it('keeps the colour input and the slider on the last valid colour while the hex is invalid', async () => {
    await setup({ kind: 'edit', index: 1, color: '#00ff00cc' });

    const hex = field(HEX);
    hex.value = '#00ff';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(field(COLOR).value).toBe('#00ff00');
    expect(field(ALPHA).valueAsNumber).toBe(0xcc);
  });

  it('starts from the entry being edited', async () => {
    await setup({ kind: 'edit', index: 1, color: '#00ff00cc' });

    expect(field(HEX).value).toBe('#00ff00cc');
    expect(field(COLOR).value).toBe('#00ff00');
    expect(field(ALPHA).valueAsNumber).toBe(0xcc);
  });

  it('submits the entry through PaletteEntryFlow', async () => {
    await setup({ kind: 'add' });
    const submit = vi.spyOn(TestBed.inject(PaletteEntryFlow), 'submit');
    const hex = field(HEX);
    hex.value = '#1a2b3cab';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    fixture.nativeElement.querySelector('form').dispatchEvent(new Event('submit'));

    expect(submit).toHaveBeenCalledWith('#1a2b3cab');
  });
});
