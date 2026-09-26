import { inject } from '@angular/core';
import { type CanActivateFn, Router } from '@angular/router';
import { SessionStore } from './session-store';

/**
 * Keeps `/account` for a signed-in visitor; a stranger is sent to sign in, with a `returnUrl`
 * bringing them back. Safe to check synchronously: `app.config.ts` awaits `SessionStore.load()`
 * before the router starts routing at all.
 */
export const requireAccount: CanActivateFn = (_route, state) => {
  const session = inject(SessionStore);
  if (session.account()) {
    return true;
  }
  const router = inject(Router);
  return router.createUrlTree(['/sign-in'], { queryParams: { returnUrl: state.url } });
};
