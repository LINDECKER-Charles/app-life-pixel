import type { Signal } from '@angular/core';
import type { MotionPreference, ThemePreference } from 'shared';

/**
 * A person's language, theme and motion, as the settings page reads and changes them. Each
 * implementation decides where they are kept: the web one keeps them for the page (D37), H7 saves a
 * signed-in person's language in the account, T2 the desktop's choices in its settings file. The
 * app binds one in `app.config.ts`; `provideAppearance()` applies theme and motion.
 */
export abstract class PreferencesStore {
  /** The code of the active language, following every switch. */
  abstract readonly language: Signal<string>;
  abstract readonly theme: Signal<ThemePreference>;
  abstract readonly motion: Signal<MotionPreference>;

  /** Loads the language's catalogue, then re-renders the interface in it without reloading. */
  abstract setLanguage(code: string): Promise<void>;
  abstract setTheme(theme: ThemePreference): Promise<void>;
  abstract setMotion(motion: MotionPreference): Promise<void>;
}
