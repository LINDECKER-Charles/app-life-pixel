import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { ImportSpriteSheetButton } from './import-sprite-sheet-button';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Button', width: 8, height: 8, layerName: 'Base' };

describe('ImportSpriteSheetButton', () => {
  let fixture: ComponentFixture<ImportSpriteSheetButton>;

  async function setup(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    await TestBed.inject(EngineStore).create(NEW_ANIMATION);
    fixture = TestBed.createComponent(ImportSpriteSheetButton);
    await fixture.whenStable();
  }

  afterEach(() => document.body.replaceChildren());

  it('forwards the picked file to ImportSpriteSheetFlow', async () => {
    await setup();
    const pickFile = vi
      .spyOn(TestBed.inject(ImportSpriteSheetFlow), 'pickFile')
      .mockResolvedValue();
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input[type="file"]');
    const file = new File([new Uint8Array(4)], 'sheet.png', { type: 'image/png' });
    Object.defineProperty(input, 'files', { value: [file] });

    input.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    expect(pickFile).toHaveBeenCalledWith(file);
  });
});
