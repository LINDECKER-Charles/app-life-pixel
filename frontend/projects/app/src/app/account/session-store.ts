import { inject, Injectable, Injector, signal } from '@angular/core';
import { AuthApi, type Account, type ApiHeaderSource } from 'shared';
import { PreferencesStore } from '../settings/preferences-store';

/**
 * The signed-in account as a signal, the CSRF token every unsafe request needs, and the sign-in,
 * sign-up and sign-out flows (accounts.md, H7). The hosted app's single source of truth for
 * whether a person is signed in; nothing is kept once the tab closes (D37).
 */
@Injectable({ providedIn: 'root' })
export class SessionStore {
  // Read lazily, through `injector`: `ApiClient` reads `csrfHeader` off this very store through
  // `API_HEADERS`, so injecting `AuthApi` (and so `ApiClient`) eagerly here would be circular.
  private readonly injector = inject(Injector);
  private readonly preferences = inject(PreferencesStore);

  private get authApi(): AuthApi {
    return this.injector.get(AuthApi);
  }

  private readonly accountSignal = signal<Account | null>(null);
  private readonly reloadRequiredSignal = signal(false);
  private csrfToken: string | null = null;
  private loadStarted: Promise<void> | undefined;

  readonly account = this.accountSignal.asReadonly();
  /** Set once a `426` answers a request: this build is behind the server's minimum. */
  readonly reloadRequired = this.reloadRequiredSignal.asReadonly();

  /** Gives `ApiClient` the CSRF header of the active session; none while signed out, or on a `GET`. */
  readonly csrfHeader: ApiHeaderSource = ({ method }) => {
    const headers: Record<string, string> = {};
    if (method !== 'get' && this.csrfToken !== null) {
      headers['X-CSRF-Token'] = this.csrfToken;
    }
    return headers;
  };

  /**
   * Reads the current session through `GET /auth/session`. Called from a single place at
   * start-up (`app.config.ts`), so that T2 can guard that one call for the desktop, which never
   * runs it; a later call returns the same settled promise instead of asking again.
   */
  load(): Promise<void> {
    this.loadStarted ??= this.refresh();
    return this.loadStarted;
  }

  async signIn(email: string, password: string): Promise<Account> {
    return this.open(this.authApi.signIn(email, password));
  }

  async signUp(email: string, password: string, language: string): Promise<Account> {
    return this.open(this.authApi.signUp(email, password, language));
  }

  async signOut(): Promise<void> {
    await this.authApi.signOut();
    this.clear();
  }

  /** Reflects an account changed elsewhere: a language saved, an address just verified. */
  setAccount(account: Account): void {
    this.accountSignal.set(account);
    this.syncLanguage(account);
  }

  /** A `401` answered some other request: the session ended elsewhere. Safe with none at all. */
  handleUnauthenticated(): void {
    this.clear();
  }

  /** A `426` answered some other request: this build is older than the server's minimum. */
  handleUpdateRequired(): void {
    this.reloadRequiredSignal.set(true);
  }

  private async refresh(): Promise<void> {
    try {
      const session = await this.authApi.session();
      this.apply(session.account, session.csrfToken);
    } catch {
      this.clear();
    }
  }

  private async open(request: Promise<{ account: Account; csrfToken: string }>): Promise<Account> {
    const session = await request;
    this.apply(session.account, session.csrfToken);
    return session.account;
  }

  private apply(account: Account, csrfToken: string): void {
    this.accountSignal.set(account);
    this.csrfToken = csrfToken;
    this.syncLanguage(account);
  }

  private syncLanguage(account: Account): void {
    if (account.language !== this.preferences.language()) {
      void this.preferences.setLanguage(account.language);
    }
  }

  private clear(): void {
    this.accountSignal.set(null);
    this.csrfToken = null;
  }
}
