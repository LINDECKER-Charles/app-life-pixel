import { inject } from '@angular/core';
import { type CanActivateFn, Router } from '@angular/router';
import { LIBRARY_ACCESS } from './library-access';

/** Where a visitor signs in; `returnUrl` brings them back (accounts.md, H7). */
const SIGN_IN_PATH = '/sign-in';

/**
 * Keeps the library pages for a person who can use the library; a visitor is sent to sign in,
 * then back. Safe to check synchronously: the hosted app reads its session before routing.
 */
export const requireLibraryAccess: CanActivateFn = (_route, state) => {
  if (inject(LIBRARY_ACCESS).signedIn()) return true;
  return inject(Router).createUrlTree([SIGN_IN_PATH], { queryParams: { returnUrl: state.url } });
};
