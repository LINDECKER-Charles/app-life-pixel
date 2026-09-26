import { computed, inject, InjectionToken, type Signal } from '@angular/core';
import { SessionStore } from '../account/session-store';

/** Whether the library can be used now: on the hosted app, only once signed in. */
export interface LibraryAccess {
  readonly signedIn: Signal<boolean>;
}

function sessionAccess(): LibraryAccess {
  const session = inject(SessionStore);
  return { signedIn: computed(() => session.account() !== null) };
}

/**
 * The hosted app's answer, from H7's `SessionStore`; T2 provides one that is always signed in,
 * since the desktop's library needs no account.
 */
export const LIBRARY_ACCESS = new InjectionToken<LibraryAccess>('LibraryAccess', {
  providedIn: 'root',
  factory: sessionAccess,
});
