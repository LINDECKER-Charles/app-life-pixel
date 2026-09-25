import { ComponentRef, importProvidersFrom } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { EngineStore } from '../engine/engine-store';
import { EditorPage } from './editor-page';
import { DiscardConfirmation } from './new-animation/discard-confirmation';
import { NewAnimationFlow } from './new-animation/new-animation-flow';
import { Shortcuts } from './shortcuts';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

const REGION_STUBS = [
  'lp-tool-bar',
  'lp-palette-panel',
  'lp-canvas',
  'lp-timeline',
  'lp-export-button',
];

describe('EditorPage', () => {
  let confirm: ReturnType<typeof vi.fn<() => Promise<boolean>>>;

  async function configure(): Promise<void> {
    confirm = vi.fn<() => Promise<boolean>>();
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
        { provide: DiscardConfirmation, useValue: { confirm } },
      ],
    });
    // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  }

  async function open(animationId?: string): Promise<ComponentFixture<EditorPage>> {
    const fixture = TestBed.createComponent(EditorPage);
    if (animationId !== undefined) {
      (fixture.componentRef as ComponentRef<EditorPage>).setInput('animationId', animationId);
    }
    await fixture.whenStable();
    return fixture;
  }

  afterEach(() => document.body.replaceChildren());

  it('holds the five regions the features fill', async () => {
    await configure();
    const fixture = await open();

    for (const selector of REGION_STUBS) {
      expect(fixture.nativeElement.querySelector(selector), selector).not.toBeNull();
    }
  });

  it('asks for a new animation on a first visit', async () => {
    await configure();
    await open();

    expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true);
  });

  it('leaves a saved animation to open to its route', async () => {
    await configure();
    await open('42');

    expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(false);
  });

  it('starts a new animation from "New", asking first when there is unsaved work', async () => {
    await configure();
    const engine = TestBed.inject(EngineStore);
    await engine.create({ title: 'Work', width: 8, height: 8, layerName: 'Base' });
    await engine.apply({ kind: 'setTitle', title: 'Work in progress' });
    const fixture = await open();
    confirm.mockResolvedValue(true);

    fixture.nativeElement.querySelector('.header ion-button').click();

    await vi.waitFor(() => expect(TestBed.inject(NewAnimationFlow).isOpen()).toBe(true));
    expect(confirm).toHaveBeenCalledOnce();
  });

  it('hands key presses to the shortcuts while it is shown', async () => {
    await configure();
    const calls: string[] = [];
    TestBed.inject(Shortcuts).register([
      { key: 'b', label: 'test.pencil', action: () => calls.push('pencil') },
    ]);
    const fixture = await open();

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }));
    fixture.componentInstance.ionViewWillLeave();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'b' }));

    expect(calls).toEqual(['pencil']);
  });
});
