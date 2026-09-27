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

  async function setup(withAnimation = true): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    if (withAnimation) await TestBed.inject(EngineStore).create(NEW_ANIMATION);
    fixture = TestBed.createComponent(ImportSpriteSheetButton);
    await fixture.whenStable();
  }

  afterEach(() => document.body.replaceChildren());

  it('exposes only the button to assistive technology, not the file input', async () => {
    await setup();
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input[type="file"]');
    const button: HTMLButtonElement = fixture.nativeElement.querySelector('button');

    expect(input.getAttribute('aria-hidden')).toBe('true');
    expect(input.tabIndex).toBe(-1);
    expect(button.getAttribute('aria-label')).toBe(en['tools.import_sprite_sheet']);
    expect(button.disabled).toBe(false);
    expect(input.disabled).toBe(false);
  });

  it('disables the button and its file input while there is no animation', async () => {
    await setup(false);
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input[type="file"]');
    const button: HTMLButtonElement = fixture.nativeElement.querySelector('button');

    expect(button.disabled).toBe(true);
    expect(input.disabled).toBe(true);
  });

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
