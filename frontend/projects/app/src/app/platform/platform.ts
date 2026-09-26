import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

/** Whether the app runs on the desktop: Tauri's own global on the webview (desktop.md, T1). */
export function isDesktop(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** T1's `platform_info`: the desktop's version, system and folders. */
export interface PlatformInfo {
  readonly version: string;
  readonly os: 'macos' | 'windows' | 'linux';
  readonly libraryPath: string;
  readonly cliPath: string | null;
}

/**
 * What the app knows about the platform it runs on beyond the library, exports and preferences
 * (desktop.md, T2): the desktop's own information, and the system's folder dialog for the
 * library. `isDesktop()` answers everywhere without injection; this service only does anything on
 * the desktop.
 */
@Injectable({ providedIn: 'root' })
export class PlatformService {
  readonly desktop = isDesktop();

  /** The desktop's version, system, library folder and bundled CLI (T1, T3). */
  info(): Promise<PlatformInfo> {
    return invoke<PlatformInfo>('platform_info');
  }

  /** The folder chosen in the system's dialog for the library, or `null` when cancelled. */
  pickLibraryFolder(): Promise<string | null> {
    return invoke<string | null>('settings_pick_library_folder');
  }
}
