import {
  DestroyRef,
  EnvironmentProviders,
  inject,
  Injector,
  makeEnvironmentProviders,
  provideAppInitializer,
  signal,
} from '@angular/core';
import { PREFERRED_LANGUAGE } from 'shared';
import { HostedExportObserver } from '../events/hosted-export-observer';
import { EXPORT_OBSERVER } from '../export/export-observer';
import { EXPORT_SAVER } from '../export/export-saver';
import { HttpLibraryStore } from '../library/http/http-library-store';
import type { LibraryAccess } from '../library/library-access';
import { LIBRARY_ACCESS } from '../library/library-access';
import { LIBRARY_STORE } from '../library/library-store';
import { PreferencesStore } from '../settings/preferences-store';
import { WebPreferencesStore } from '../settings/web-preferences-store';
import { desktopPreferredLanguage, DesktopPreferencesStore } from './desktop-preferences';
import { DesktopExportSaver } from './desktop-export-saver';
import { isDesktop } from './platform';
import { TauriLibraryStore } from './tauri-library-store';

/** The desktop's library needs no account: it is always usable (desktop.md, T2). */
const DESKTOP_LIBRARY_ACCESS: LibraryAccess = { signedIn: signal(true) };

/**
 * Picks the desktop or the web implementations of the library, the exports and the preferences
 * (desktop.md, T2), and keeps U5's silent `EXPORT_OBSERVER` on the desktop, H13's hosted one on
 * the web. On the desktop, the app also follows the library's changes (T3). Call once, in
 * `app.config.ts`.
 */
export function providePlatform(): EnvironmentProviders {
  return isDesktop() ? provideDesktopPlatform() : provideWebPlatform();
}

function provideDesktopPlatform(): EnvironmentProviders {
  return makeEnvironmentProviders([
    TauriLibraryStore,
    DesktopExportSaver,
    DesktopPreferencesStore,
    { provide: LIBRARY_STORE, useExisting: TauriLibraryStore },
    { provide: EXPORT_SAVER, useExisting: DesktopExportSaver },
    { provide: PreferencesStore, useExisting: DesktopPreferencesStore },
    { provide: LIBRARY_ACCESS, useValue: DESKTOP_LIBRARY_ACCESS },
    { provide: PREFERRED_LANGUAGE, useValue: desktopPreferredLanguage, multi: true },
    provideAppInitializer(followLibraryChanges),
  ]);
}

/** Follows the library's changes (T3), from a chunk only the desktop loads. */
async function followLibraryChanges(): Promise<void> {
  const injector = inject(Injector);
  const destroyRef = inject(DestroyRef);
  const { DesktopLibraryEvents } = await import('./library-events/desktop-library-events');
  if (!destroyRef.destroyed) await injector.get(DesktopLibraryEvents).follow();
}

function provideWebPlatform(): EnvironmentProviders {
  return makeEnvironmentProviders([
    { provide: LIBRARY_STORE, useExisting: HttpLibraryStore },
    { provide: PreferencesStore, useClass: WebPreferencesStore },
    { provide: EXPORT_OBSERVER, useClass: HostedExportObserver },
  ]);
}
