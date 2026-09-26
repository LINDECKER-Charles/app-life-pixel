import { InjectionToken } from '@angular/core';

/** Reacts to a session ending (`401`) or this build falling behind the server's minimum (`426`). */
export interface SessionEventHandler {
  onUnauthenticated?(): void;
  onUpdateRequired?(): void;
}

/**
 * Where `SessionStore` (`app/account`) reacts to `sessionInterceptor`'s findings, the way
 * `API_HEADERS` lets it add the CSRF header: `{ provide: SESSION_EVENTS, useFactory: …, multi:
 * true }` in `app.config.ts`.
 */
export const SESSION_EVENTS = new InjectionToken<readonly SessionEventHandler[]>('SESSION_EVENTS');
