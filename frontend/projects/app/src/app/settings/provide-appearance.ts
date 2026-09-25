import {
  DestroyRef,
  DOCUMENT,
  effect,
  EnvironmentProviders,
  inject,
  makeEnvironmentProviders,
  provideEnvironmentInitializer,
  signal,
  type Signal,
} from '@angular/core';
import { applyAppearance } from 'shared';
import { PreferencesStore } from './preferences-store';

const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)';

/** The part of `window.Ionic.config` that turns Ionic's own animations on and off at run time. */
interface IonicGlobal {
  readonly config?: { set(key: 'animated', value: boolean): void };
}

/** Whether the system asks for reduced motion, following its changes. */
function systemReducesMotion(view: Window | null): Signal<boolean> {
  const query = view?.matchMedia?.(REDUCED_MOTION_QUERY);
  const reduces = signal(query?.matches ?? false);
  if (!query) return reduces;
  const follow = (event: MediaQueryListEvent): void => reduces.set(event.matches);
  query.addEventListener('change', follow);
  inject(DestroyRef).onDestroy(() => query.removeEventListener('change', follow));
  return reduces;
}

function setIonicAnimations(view: Window | null, isAnimated: boolean): void {
  const ionic = (view as { Ionic?: IonicGlobal } | null)?.Ionic;
  ionic?.config?.set('animated', isAnimated);
}

function applyPreferences(): void {
  const preferences = inject(PreferencesStore);
  const document = inject(DOCUMENT);
  const view = document.defaultView;
  const systemReduces = systemReducesMotion(view);
  effect(() => {
    const motion = preferences.motion();
    applyAppearance(document.documentElement, { theme: preferences.theme(), motion });
    setIonicAnimations(view, motion !== 'reduce' && !systemReduces());
  });
}

/**
 * Applies the theme and the motion of `PreferencesStore` as they change: the design tokens'
 * attributes on the root element, and Ionic's animations off whenever motion is reduced — by the
 * settings or by the system.
 */
export function provideAppearance(): EnvironmentProviders {
  return makeEnvironmentProviders([provideEnvironmentInitializer(applyPreferences)]);
}
