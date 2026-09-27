import { ComponentRef, importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';
import { ImportSpriteSheetForm } from './import-sprite-sheet-form';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Form', width: 8, height: 8, layerName: 'Base' };

describe('ImportSpriteSheetForm', () => {
  let fixture: ComponentFixture<ImportSpriteSheetForm>;
  let engine: EngineStore;

  async function setup(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        provideTranslocoMessageformat(),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    fixture = TestBed.createComponent(ImportSpriteSheetForm);
    const limits = engine.limits();
    if (!limits) throw new Error('limits missing');
    (fixture.componentRef as ComponentRef<ImportSpriteSheetForm>).setInput('limits', limits);
    await fixture.whenStable();
  }

  /** The field a label names, found as assistive technology finds it: through its `for`. */
  function field(label: string): HTMLInputElement {
    const host = fixture.nativeElement as HTMLElement;
    const found = [...host.querySelectorAll('label')].find(
      (candidate) => candidate.textContent?.trim() === label,
    )?.control;
    if (!(found instanceof HTMLInputElement)) throw new Error(`no field labelled ${label}`);
    return found;
  }

  it('bounds the cell size and defaults the duration to the engine limit', async () => {
    await setup();
    const limits = engine.limits();

    expect(field('Cell width').max).toBe(String(limits?.importMaxSide));
    expect(field('Frame duration (ms)').valueAsNumber).toBe(limits?.defaultFrameDurationMs);
  });

  it('marks an out-of-bounds duration invalid, disabling submit', async () => {
    await setup();
    const limits = engine.limits();
    const duration = field('Frame duration (ms)');

    duration.value = String((limits?.maxFrameDurationMs ?? 0) + 1);
    duration.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(duration.getAttribute('aria-invalid')).toBe('true');
    expect(fixture.nativeElement.querySelector('button[type="submit"]').disabled).toBe(true);
  });

  it('explains an out-of-bounds cell width beside the field', async () => {
    await setup();
    const width = field('Cell width');

    width.value = '0';
    width.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    const error = width.closest('.lp-field')?.querySelector<HTMLElement>('.lp-field__error');
    expect(width.getAttribute('aria-describedby')?.split(' ')).toContain(error?.id);
    expect(error?.textContent).toMatch(/1.*4,?096/);
  });

  it('submits the cell size and the duration through ImportSpriteSheetFlow', async () => {
    await setup();
    const submit = vi.spyOn(TestBed.inject(ImportSpriteSheetFlow), 'submit');
    field('Cell width').value = '4';
    field('Cell width').dispatchEvent(new Event('input'));
    field('Cell height').value = '4';
    field('Cell height').dispatchEvent(new Event('input'));
    fixture.detectChanges();

    fixture.nativeElement.querySelector('form').dispatchEvent(new Event('submit'));

    expect(submit).toHaveBeenCalledWith(expect.objectContaining({ cellWidth: 4, cellHeight: 4 }));
  });
});
