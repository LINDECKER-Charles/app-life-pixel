import { HttpErrorResponse, type HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { catchError, throwError } from 'rxjs';
import { SESSION_EVENTS } from './session-events';

const UNAUTHENTICATED_STATUS = 401;
const UPDATE_REQUIRED_STATUS = 426;

/**
 * Watches every response for a session ending or this build falling behind the server's minimum,
 * and tells `SESSION_EVENTS`'s handlers; the request's own failure still reaches its caller as an
 * `ApiProblem` (accounts.md, H7).
 */
export const sessionInterceptor: HttpInterceptorFn = (request, next) => {
  const handlers = inject(SESSION_EVENTS, { optional: true }) ?? [];
  return next(request).pipe(
    catchError((error: unknown) => {
      if (error instanceof HttpErrorResponse) {
        if (error.status === UNAUTHENTICATED_STATUS) {
          handlers.forEach((handler) => handler.onUnauthenticated?.());
        }
        if (error.status === UPDATE_REQUIRED_STATUS) {
          handlers.forEach((handler) => handler.onUpdateRequired?.());
        }
      }
      return throwError(() => error);
    }),
  );
};
