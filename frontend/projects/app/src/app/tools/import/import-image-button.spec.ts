import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { ImportImage } from './import-image';
import { ImportImageButton } from './import-image-button';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };
const NEW_ANIMATION = { title: 'Button', width: 8, height: 8, layerName: 'Base' };

describe('ImportImageButton', () => {
  let fixture: ComponentFixture<ImportImageButton>;

  async function setup(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    await TestBed.inject(EngineStore).create(NEW_ANIMATION);
    fixture = TestBed.createComponent(ImportImageButton);
    await fixture.whenStable();
  }

  it('opens the file picker from the button', async () => {
    await setup();
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input[type="file"]');
    const click = vi.spyOn(input, 'click');

    fixture.nativeElement.querySelector('button').click();

    expect(click).toHaveBeenCalled();
  });

  it('forwards the picked file to ImportImage', async () => {
    await setup();
    const importFile = vi.spyOn(TestBed.inject(ImportImage), 'importFile').mockResolvedValue();
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input[type="file"]');
    const file = new File([new Uint8Array(4)], 'a.png', { type: 'image/png' });
    Object.defineProperty(input, 'files', { value: [file] });

    input.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    expect(importFile).toHaveBeenCalledWith(file);
  });
});
