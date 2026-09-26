import { inject, Injectable } from '@angular/core';
import { ApiClient } from './api-client';
import type { components } from './schema';

/** A session: the account signed in, and the CSRF token its unsafe requests must send. */
export type Session = components['schemas']['Session'];

/**
 * H5's routes, typed from `schema.d.ts`: sign-up, sign-in, sign-out, the current session,
 * verification and password reset. `SessionStore` (`app/account`) is the only caller: it turns
 * `Session` into the signed-in account and the CSRF header every other request needs.
 */
@Injectable({ providedIn: 'root' })
export class AuthApi {
  private readonly client = inject(ApiClient);

  /** The session the request carries, `auth.unauthenticated` otherwise. */
  session(): Promise<Session> {
    return this.client.request('get', '/api/v1/auth/session');
  }

  signUp(email: string, password: string, language: string): Promise<Session> {
    return this.client.request('post', '/api/v1/auth/sign-up', {
      body: { email, password, language },
    });
  }

  signIn(email: string, password: string): Promise<Session> {
    return this.client.request('post', '/api/v1/auth/sign-in', { body: { email, password } });
  }

  signOut(): Promise<void> {
    return this.client.request('post', '/api/v1/auth/sign-out');
  }

  verifyEmail(token: string): Promise<void> {
    return this.client.request('post', '/api/v1/auth/verify-email', { body: { token } });
  }

  /** Nothing once the address is verified: the answer never says which. */
  resendVerificationEmail(): Promise<void> {
    return this.client.request('post', '/api/v1/auth/verify-email/resend');
  }

  /** A link is on its way, whether an active account has the address or not. */
  requestPasswordReset(email: string): Promise<void> {
    return this.client.request('post', '/api/v1/auth/password-reset', { body: { email } });
  }

  /** Sets a new password from an emailed link's token; every session of the account ends. */
  confirmPasswordReset(token: string, password: string): Promise<void> {
    return this.client.request('post', '/api/v1/auth/password-reset/confirm', {
      body: { token, password },
    });
  }

  /** Replaces the signed-in account's password; its other sessions end. */
  changePassword(currentPassword: string, newPassword: string): Promise<void> {
    return this.client.request('put', '/api/v1/auth/password', {
      body: { currentPassword, newPassword },
    });
  }
}
