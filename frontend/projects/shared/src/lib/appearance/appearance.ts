/** The colours: the system's scheme, or a set chosen by hand. */
export type ThemePreference = 'system' | 'light' | 'dark';

/** Motion follows the system's setting, or is reduced whatever the system says. */
export type MotionPreference = 'system' | 'reduce';

export const THEME_PREFERENCES: readonly ThemePreference[] = ['system', 'light', 'dark'];
export const MOTION_PREFERENCES: readonly MotionPreference[] = ['system', 'reduce'];

/** What the design tokens of `styles/` read from the root element. */
export interface Appearance {
  readonly theme: ThemePreference;
  readonly motion: MotionPreference;
}
