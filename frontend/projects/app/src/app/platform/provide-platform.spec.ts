import { TestBed } from '@angular/core/testing';
import { TranslocoService } from '@jsverse/transloco';
import { mockIPC } from '@tauri-apps/api/mocks';
import { of } from 'rxjs';
import { EventsApi, LIFE_PIXEL_CLIENT, PREFERRED_LANGUAGE } from 'shared';
import { HostedExportObserver } from '../events/hosted-export-observer';
import { EXPORT_OBSERVER } from '../export/export-observer';
import { EXPORT_SAVER } from '../export/export-saver';
import { HttpLibraryStore } from '../library/http/http-library-store';
import { LIBRARY_ACCESS } from '../library/library-access';
import { LIBRARY_STORE } from '../library/library-store';
import { PreferencesStore } from '../settings/preferences-store';
import { WebPreferencesStore } from '../settings/web-preferences-store';
import { DesktopExportSaver } from './desktop-export-saver';
import { DesktopPreferencesStore } from './desktop-preferences';
import { providePlatform } from './provide-platform';
import { TauriLibraryStore } from './tauri-library-store';
import { clearTauriMocks } from './testing/clear-tauri-mocks';

const TRANSLOCO = { langChanges$: of('en'), getActiveLang: () => 'en' };

function configure(): void {
  TestBed.configureTestingModule({
    providers: [
      providePlatform(),
      { provide: TranslocoService, useValue: TRANSLOCO },
      { provide: EventsApi, useValue: { exportCompleted: vi.fn() } },
      { provide: LIFE_PIXEL_CLIENT, useValue: 'web/0.0.0' },
    ],
  });
}

describe('providePlatform on the web', () => {
  it('picks the hosted library, preferences and export observer', () => {
    configure();

    expect(TestBed.inject(LIBRARY_STORE)).toBeInstanceOf(HttpLibraryStore);
    expect(TestBed.inject(PreferencesStore)).toBeInstanceOf(WebPreferencesStore);
    expect(TestBed.inject(EXPORT_OBSERVER)).toBeInstanceOf(HostedExportObserver);
  });

  it('lets the library’s own default answer whether a visitor can use it', () => {
    configure();

    expect(TestBed.inject(LIBRARY_ACCESS).signedIn()).toBe(false);
  });
});

describe('providePlatform on the desktop', () => {
  afterEach(() => clearTauriMocks());

  it('picks the desktop library, exports, preferences and library access', () => {
    mockIPC(() => ({ libraryPath: null, language: null, theme: 'system', motion: 'system' }));
    configure();

    expect(TestBed.inject(LIBRARY_STORE)).toBeInstanceOf(TauriLibraryStore);
    expect(TestBed.inject(EXPORT_SAVER)).toBeInstanceOf(DesktopExportSaver);
    expect(TestBed.inject(PreferencesStore)).toBeInstanceOf(DesktopPreferencesStore);
    expect(TestBed.inject(LIBRARY_ACCESS).signedIn()).toBe(true);
  });

  it('adds the desktop settings as a source of the preferred language', async () => {
    mockIPC(() => ({ libraryPath: null, language: 'fr', theme: 'system', motion: 'system' }));
    configure();

    const sources = TestBed.inject(PREFERRED_LANGUAGE);
    await expect(sources[sources.length - 1]()).resolves.toBe('fr');
  });
});
