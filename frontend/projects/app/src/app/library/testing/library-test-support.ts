import { importProvidersFrom, signal, type Provider, type WritableSignal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { EngineStore } from '../../engine/engine-store';
import { LIBRARY_ACCESS } from '../library-access';
import { LIBRARY_STORE } from '../library-store';
import { FakeLibraryStore } from './fake-library-store';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

/** What a library test works with: the fake store, and whether the person is signed in. */
export interface LibraryTestBed {
  readonly store: FakeLibraryStore;
  readonly signedIn: WritableSignal<boolean>;
}

/**
 * Configures the library's collaborators for a test: Ionic without animations, the English
 * catalogue with the app's message format, a router accepting any path, `FakeLibraryStore` and a signed-in person.
 */
export async function configureLibrary(...providers: Provider[]): Promise<LibraryTestBed> {
  const store = new FakeLibraryStore();
  const signedIn = signal(true);
  TestBed.configureTestingModule({
    providers: [
      provideIonicAngular({ animated: false }),
      importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      provideTranslocoMessageformat(),
      provideRouter([{ path: '**', children: [] }]),
      { provide: LIBRARY_STORE, useValue: store },
      { provide: LIBRARY_ACCESS, useValue: { signedIn } },
      ...providers,
    ],
  });
  // Angular 22 no longer runs the testing module's initializer: the catalogue loads here.
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  return { store, signedIn };
}

/** Starts an animation titled `title` in the editor, then edits it: unsaved work. */
export async function startUnsavedWork(title = 'Work'): Promise<EngineStore> {
  const engine = TestBed.inject(EngineStore);
  await engine.create({ title: 'Untitled', width: 8, height: 8, layerName: 'Base' });
  await engine.apply({ kind: 'setTitle', title });
  return engine;
}

/** Serious and critical accessibility violations: what the tests forbid. */
export const SERIOUS_IMPACTS = ['serious', 'critical'];
