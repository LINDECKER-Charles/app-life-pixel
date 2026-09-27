import { importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { AnimationTitle } from './animation-title';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

const NEW_ANIMATION = { title: 'Walk', width: 8, height: 8, layerName: 'Base' };

describe('AnimationTitle', () => {
  async function render(): Promise<ComponentFixture<AnimationTitle>> {
    const fixture = TestBed.createComponent(AnimationTitle);
    await fixture.whenStable();
    return fixture;
  }

  async function setup(): Promise<EngineStore> {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
    const engine = TestBed.inject(EngineStore);
    await engine.create(NEW_ANIMATION);
    return engine;
  }

  function titleField(fixture: ComponentFixture<AnimationTitle>): HTMLInputElement {
    return fixture.nativeElement.querySelector('input');
  }

  it('renames the animation in place', async () => {
    const engine = await setup();
    const field = titleField(await render());
    expect(field.value).toBe(NEW_ANIMATION.title);

    field.value = '  Walk cycle ';
    field.dispatchEvent(new Event('change'));

    await vi.waitFor(() => expect(field.value).toBe('Walk cycle'));
    expect(engine.document()?.title).toBe('Walk cycle');
  });

  it('keeps the title when the field is emptied', async () => {
    const engine = await setup();
    const fixture = await render();
    const field = titleField(fixture);

    field.value = '   ';
    field.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    expect(engine.document()?.title).toBe(NEW_ANIMATION.title);
    expect(engine.hasUnsavedWork()).toBe(false);
    expect(field.value).toBe(NEW_ANIMATION.title);
  });

  it('restores the title on Escape', async () => {
    await setup();
    const field = titleField(await render());

    field.value = 'Half typed';
    field.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));

    expect(field.value).toBe(NEW_ANIMATION.title);
  });

  it('says when no animation is open', async () => {
    TestBed.configureTestingModule({
      providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));

    const fixture = await render();

    expect(fixture.nativeElement.textContent.trim()).toBe(en['editor.empty.hint']);
  });
});
