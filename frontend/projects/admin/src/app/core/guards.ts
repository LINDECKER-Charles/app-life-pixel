import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { SessionStore } from './session-store';

/** A page of the console: signed in, or sent to sign in and brought back afterwards. */
export const requireAdmin: CanActivateFn = (_route, state) => {
  if (inject(SessionStore).signedIn()) {
    return true;
  }
  return inject(Router).createUrlTree(['/sign-in'], { queryParams: { returnUrl: state.url } });
};

/** The sign-in page, for a signed-out visitor only. */
export const requireSignedOut: CanActivateFn = () =>
  inject(SessionStore).signedIn() ? inject(Router).createUrlTree(['/']) : true;
