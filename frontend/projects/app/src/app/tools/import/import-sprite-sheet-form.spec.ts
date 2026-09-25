import { ComponentRef, importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
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
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
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

  function field(selector: string): HTMLInputElement {
    return fixture.nativeElement.querySelector(selector);
  }

  it('bounds the cell size and defaults the duration to the engine limit', async () => {
    await setup();
    const limits = engine.limits();

    expect(field('#sprite-sheet-cell-width').max).toBe(String(limits?.importMaxSide));
    expect(field('#sprite-sheet-duration').valueAsNumber).toBe(limits?.defaultFrameDurationMs);
  });

  it('marks an out-of-bounds duration invalid, disabling submit', async () => {
    await setup();
    const limits = engine.limits();
    const duration = field('#sprite-sheet-duration');

    duration.value = String((limits?.maxFrameDurationMs ?? 0) + 1);
    duration.dispatchEvent(new Event('input'));
    fixture.detectChanges();

    expect(duration.getAttribute('aria-invalid')).toBe('true');
    expect(fixture.nativeElement.querySelector('button[type="submit"]').disabled).toBe(true);
  });

  it('submits the cell size and the duration through ImportSpriteSheetFlow', async () => {
    await setup();
    const submit = vi.spyOn(TestBed.inject(ImportSpriteSheetFlow), 'submit');
    field('#sprite-sheet-cell-width').value = '4';
    field('#sprite-sheet-cell-width').dispatchEvent(new Event('input'));
    field('#sprite-sheet-cell-height').value = '4';
    field('#sprite-sheet-cell-height').dispatchEvent(new Event('input'));
    fixture.detectChanges();

    fixture.nativeElement.querySelector('form').dispatchEvent(new Event('submit'));

    expect(submit).toHaveBeenCalledWith(expect.objectContaining({ cellWidth: 4, cellHeight: 4 }));
  });
});
