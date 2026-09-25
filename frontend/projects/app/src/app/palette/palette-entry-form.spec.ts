import { ComponentRef, importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EngineStore } from '../engine/engine-store';
import { PaletteEntryFlow, type PaletteEntryMode } from './palette-entry-flow';
import { PaletteEntryForm } from './palette-entry-form';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Entry', width: 8, height: 8, layerName: 'Base' };

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

  function field(selector: string): HTMLInputElement {
    return fixture.nativeElement.querySelector(selector);
  }

  it('keeps the colour input, the alpha slider and the hex field in sync from the hex field', async () => {
    await setup({ kind: 'add' });

    const hex = field('#palette-entry-hex');
    hex.value = '#1a2b3cab';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(field('#palette-entry-color').value).toBe('#1a2b3c');
    expect(field('#palette-entry-alpha').valueAsNumber).toBe(0xab);
  });

  it('keeps the fields in sync from the colour input and the alpha slider', async () => {
    await setup({ kind: 'add' });

    const color = field('#palette-entry-color');
    color.value = '#334455';
    color.dispatchEvent(new Event('input'));
    const alpha = field('#palette-entry-alpha');
    alpha.value = '128';
    alpha.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(field('#palette-entry-hex').value).toBe('#33445580');
  });

  it('marks an invalid hex value invalid, without touching the other fields', async () => {
    await setup({ kind: 'add' });

    const hex = field('#palette-entry-hex');
    hex.value = 'not a colour';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(hex.getAttribute('aria-invalid')).toBe('true');
    expect(fixture.nativeElement.querySelector('button[type="submit"]').disabled).toBe(true);
  });

  it('starts from the entry being edited', async () => {
    await setup({ kind: 'edit', index: 1, color: '#00ff00cc' });

    expect(field('#palette-entry-hex').value).toBe('#00ff00cc');
    expect(field('#palette-entry-color').value).toBe('#00ff00');
    expect(field('#palette-entry-alpha').valueAsNumber).toBe(0xcc);
  });

  it('submits the entry through PaletteEntryFlow', async () => {
    await setup({ kind: 'add' });
    const submit = vi.spyOn(TestBed.inject(PaletteEntryFlow), 'submit');
    const hex = field('#palette-entry-hex');
    hex.value = '#1a2b3cab';
    hex.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    fixture.nativeElement.querySelector('form').dispatchEvent(new Event('submit'));

    expect(submit).toHaveBeenCalledWith('#1a2b3cab');
  });
});
