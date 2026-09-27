import type { Appearance } from './appearance';

/**
 * Sets `data-theme` and `data-motion` on the root element, which select the dark set and reduced
 * motion in the design tokens (`styles/`): a manual choice takes effect without reloading.
 */
export function applyAppearance(root: HTMLElement, appearance: Appearance): void {
  root.dataset['theme'] = appearance.theme;
  root.dataset['motion'] = appearance.motion;
}
