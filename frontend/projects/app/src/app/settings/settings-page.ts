import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AvailableLanguages, idScope, type MotionPreference, type ThemePreference } from 'shared';
import { SessionStore } from '../account/session-store';
import { DesktopPreferencesStore } from '../platform/desktop-preferences';
import { isDesktop, PlatformService, type PlatformInfo } from '../platform/platform';
import { PreferencesStore } from './preferences-store';

/** A value to choose, and the i18n key of its label. */
interface Choice<Value> {
  readonly value: Value;
  readonly label: string;
}

/** A theme to choose: its label's and its hint's i18n keys. */
interface ThemeChoice extends Choice<ThemePreference> {
  readonly hint: string;
}

const THEME_CHOICES: readonly ThemeChoice[] = [
  { value: 'system', label: 'settings.theme.system', hint: 'settings.theme.system_hint' },
  { value: 'light', label: 'settings.theme.light', hint: 'settings.theme.light_hint' },
  { value: 'dark', label: 'settings.theme.dark', hint: 'settings.theme.dark_hint' },
];

const MOTION_CHOICES: readonly Choice<MotionPreference>[] = [
  { value: 'system', label: 'settings.motion.system' },
  { value: 'reduce', label: 'settings.motion.reduce' },
];

/**
 * Settings grouped by purpose (design-system/docs/journeys.md, §7): language, appearance, motion
 * — each applied as soon as it is chosen, without reloading —, then the desktop's library folder
 * and agents, or a signed-in account's access tokens.
 */
@Component({
  selector: 'lp-settings-page',
  imports: [IonContent, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './settings-page.html',
  styleUrl: './settings-page.scss',
})
export class SettingsPage {
  protected readonly id = idScope('settings-page');
  protected readonly preferences = inject(PreferencesStore);
  protected readonly languages = inject(AvailableLanguages).languages;
  protected readonly themes = THEME_CHOICES;
  protected readonly motions = MOTION_CHOICES;
  /** Only the desktop gains a library folder to choose (desktop.md, T2). */
  protected readonly desktop = isDesktop();
  protected readonly platformInfo = signal<PlatformInfo | null>(null);
  /** A signed-in visitor of the hosted app gains a link to their access tokens (mcp-cli.md, A3). */
  protected readonly account = inject(SessionStore).account;
  private readonly platform = inject(PlatformService);
  private readonly desktopPreferences = inject(DesktopPreferencesStore, { optional: true });

  constructor() {
    if (this.desktop) void this.loadPlatformInfo();
  }

  /** Opens the system's folder dialog, then persists the choice unless it is cancelled. */
  protected async chooseLibraryFolder(): Promise<void> {
    const folder = await this.platform.pickLibraryFolder();
    if (folder === null) return;
    await this.desktopPreferences?.setLibraryFolder(folder);
    await this.loadPlatformInfo();
  }

  private async loadPlatformInfo(): Promise<void> {
    this.platformInfo.set(await this.platform.info());
  }
}
