import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AvailableLanguages, type MotionPreference, type ThemePreference } from 'shared';
import { PreferencesStore } from './preferences-store';

/** A value to choose, and the i18n key of its label. */
interface Choice<Value> {
  readonly value: Value;
  readonly label: string;
}

const THEME_CHOICES: readonly Choice<ThemePreference>[] = [
  { value: 'system', label: 'settings.theme.system' },
  { value: 'light', label: 'settings.theme.light' },
  { value: 'dark', label: 'settings.theme.dark' },
];

const MOTION_CHOICES: readonly Choice<MotionPreference>[] = [
  { value: 'system', label: 'settings.motion.system' },
  { value: 'reduce', label: 'settings.motion.reduce' },
];

/** Language, theme and motion, each applied as soon as it is chosen. */
@Component({
  selector: 'lp-settings-page',
  imports: [IonContent, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './settings-page.html',
  styleUrl: './settings-page.scss',
})
export class SettingsPage {
  protected readonly preferences = inject(PreferencesStore);
  protected readonly languages = inject(AvailableLanguages).languages;
  protected readonly themes = THEME_CHOICES;
  protected readonly motions = MOTION_CHOICES;
}
