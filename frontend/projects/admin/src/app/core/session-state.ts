import { computed, Injectable, signal } from '@angular/core';
import type { ConsoleSession } from './admin-types';

/**
 * The live session, as the admin client and the session store share it: `undefined` before the
 * first read, `null` signed out. It depends on nothing, so that the client can read the CSRF
 * token and end the session without a cycle through the store.
 */
@Injectable({ providedIn: 'root' })
export class SessionState {
  private readonly current = signal<ConsoleSession | null | undefined>(undefined);

  readonly session = this.current.asReadonly();
  readonly csrfToken = computed(() => this.current()?.csrfToken ?? null);

  set(session: ConsoleSession | null): void {
    this.current.set(session);
  }

  /** The admin server answered `admin.unauthenticated`: the session is over. */
  end(): void {
    if (this.current() !== undefined) {
      this.current.set(null);
    }
  }
}
