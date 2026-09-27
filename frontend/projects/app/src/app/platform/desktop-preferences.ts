import { computed, inject, Injectable, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { invoke } from '@tauri-apps/api/core';
import { TranslocoService } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import type { MotionPreference, ThemePreference } from 'shared';
import { PreferencesStore } from '../settings/preferences-store';

/** `settings.json`'s shape (desktop.md, T1); a field missing from an answer takes its default. */
interface DesktopSettings {
  readonly libraryPath: string | null;
  readonly language: string | null;
  readonly theme: ThemePreference;
  readonly motion: MotionPreference;
}

const DEFAULT_SETTINGS: DesktopSettings = {
  libraryPath: null,
  language: null,
  theme: 'system',
  motion: 'system',
};

/** The language a person chose earlier in the desktop's settings, for `PREFERRED_LANGUAGE`. */
export function desktopPreferredLanguage(): Promise<string | undefined> {
  return invoke<DesktopSettings>('settings_get').then((settings) => settings.language ?? undefined);
}

/**
 * The desktop's preferences (desktop.md, T2): `PreferencesStore` over T1's `settings_get` and
 * `settings_set`, kept in `settings.json`. Also keeps the library folder there, applied the same
 * way; `PlatformService.pickLibraryFolder()` only asks the system's dialog, this store persists
 * the choice.
 */
@Injectable()
export class DesktopPreferencesStore extends PreferencesStore {
  private readonly transloco = inject(TranslocoService);
  private readonly settingsSignal = signal<DesktopSettings>(DEFAULT_SETTINGS);
  private readonly loaded: Promise<void> = this.refresh();

  readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });
  readonly theme = computed(() => this.settingsSignal().theme);
  readonly motion = computed(() => this.settingsSignal().motion);
  /** The library folder in `settings.json`; `null` until a folder was chosen. */
  readonly libraryPath = computed(() => this.settingsSignal().libraryPath);

  async setLanguage(code: string): Promise<void> {
    await firstValueFrom(this.transloco.load(code));
    this.transloco.setActiveLang(code);
    await this.update({ language: code });
  }

  async setTheme(theme: ThemePreference): Promise<void> {
    await this.update({ theme });
  }

  async setMotion(motion: MotionPreference): Promise<void> {
    await this.update({ motion });
  }

  /** Persists the folder just chosen in the system's dialog (`settings_pick_library_folder`). */
  async setLibraryFolder(path: string): Promise<void> {
    await this.update({ libraryPath: path });
  }

  private async refresh(): Promise<void> {
    this.settingsSignal.set(await invoke<DesktopSettings>('settings_get'));
  }

  private async update(change: Partial<DesktopSettings>): Promise<void> {
    await this.loaded;
    const settings = { ...this.settingsSignal(), ...change };
    this.settingsSignal.set(await invoke<DesktopSettings>('settings_set', { settings }));
  }
}
