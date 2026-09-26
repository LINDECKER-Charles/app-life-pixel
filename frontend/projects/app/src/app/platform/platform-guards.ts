import { inject } from '@angular/core';
import { type CanActivateFn, Router } from '@angular/router';
import { isDesktop } from './platform';

/** Where a route this platform cannot serve bounces to: the editor works everywhere. */
const FALLBACK_PATH = '/editor';

/**
 * Keeps a route to the hosted app (desktop.md, T2): the account, sign-in and support routes, and
 * A3's `settings/tokens`. The desktop bounces to the editor; it has no account at all.
 */
export const hostedOnly: CanActivateFn = () => {
  if (!isDesktop()) return true;
  return inject(Router).createUrlTree([FALLBACK_PATH]);
};

/**
 * Keeps a route to the desktop app: T3's `settings/agents`, which shows the bundled CLI's local
 * setup. The hosted web bounces to the editor.
 */
export const desktopOnly: CanActivateFn = () => {
  if (isDesktop()) return true;
  return inject(Router).createUrlTree([FALLBACK_PATH]);
};
