import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import type { MotionPreference, ThemePreference } from 'shared';
import { PreferencesStore } from '../settings/preferences-store';
import { SessionStore } from './session-store';

class FakePreferences extends PreferencesStore {
  private readonly languageSignal = signal('en');
  readonly language = this.languageSignal.asReadonly();
  readonly theme = signal<ThemePreference>('system').asReadonly();
  readonly motion = signal<MotionPreference>('system').asReadonly();
  readonly setLanguageCalls: string[] = [];

  override async setLanguage(code: string): Promise<void> {
    this.setLanguageCalls.push(code);
    this.languageSignal.set(code);
  }

  override setTheme(): Promise<void> {
    return Promise.resolve();
  }

  override setMotion(): Promise<void> {
    return Promise.resolve();
  }
}

const ACCOUNT = {
  id: 'a1',
  email: 'lee@example.com',
  emailVerified: false,
  language: 'fr',
  plan: 'free',
  storage: { usedBytes: 0, limitBytes: 100 },
  createdAt: '2024-01-01T00:00:00Z',
};

function setUp(): {
  session: SessionStore;
  http: HttpTestingController;
  preferences: FakePreferences;
} {
  const preferences = new FakePreferences();
  TestBed.configureTestingModule({
    providers: [
      provideHttpClient(),
      provideHttpClientTesting(),
      { provide: PreferencesStore, useValue: preferences },
    ],
  });
  return {
    session: TestBed.inject(SessionStore),
    http: TestBed.inject(HttpTestingController),
    preferences,
  };
}

describe('SessionStore', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('loads the session at start-up and syncs the language', async () => {
    const { session, http, preferences } = setUp();
    const loaded = session.load();
    http.expectOne('/api/v1/auth/session').flush({ account: ACCOUNT, csrfToken: 't0k' });
    await loaded;

    expect(session.account()).toEqual(ACCOUNT);
    expect(preferences.setLanguageCalls).toEqual(['fr']);
  });

  it('asks only once even when load() is called again', async () => {
    const { session, http } = setUp();
    const first = session.load();
    const second = session.load();
    http.expectOne('/api/v1/auth/session').flush({ account: ACCOUNT, csrfToken: 't0k' });
    await Promise.all([first, second]);
  });

  it('leaves the account empty when there is no session', async () => {
    const { session, http } = setUp();
    const loaded = session.load();
    http
      .expectOne('/api/v1/auth/session')
      .flush(
        { code: 'auth.unauthenticated', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );
    await loaded;

    expect(session.account()).toBeNull();
  });

  it('signs in, keeping the CSRF token for unsafe requests only', async () => {
    const { session, http } = setUp();
    const signingIn = session.signIn('lee@example.com', 'a very long password');
    http.expectOne('/api/v1/auth/sign-in').flush({ account: ACCOUNT, csrfToken: 't0k' });
    await signingIn;

    expect(session.account()).toEqual(ACCOUNT);
    expect(session.csrfHeader({ method: 'post', url: '/api/v1/account' })).toEqual({
      'X-CSRF-Token': 't0k',
    });
    expect(session.csrfHeader({ method: 'get', url: '/api/v1/account' })).toEqual({});
  });

  it('signs out and clears the session', async () => {
    const { session, http } = setUp();
    const signingIn = session.signIn('lee@example.com', 'a very long password');
    http.expectOne('/api/v1/auth/sign-in').flush({ account: ACCOUNT, csrfToken: 't0k' });
    await signingIn;

    const signingOut = session.signOut();
    http.expectOne('/api/v1/auth/sign-out').flush(null, { status: 204, statusText: 'No Content' });
    await signingOut;

    expect(session.account()).toBeNull();
    expect(session.csrfHeader({ method: 'post', url: '/api/v1/account' })).toEqual({});
  });

  it('clears the session on a 401 reported elsewhere', async () => {
    const { session, http } = setUp();
    const signingIn = session.signIn('lee@example.com', 'a very long password');
    http.expectOne('/api/v1/auth/sign-in').flush({ account: ACCOUNT, csrfToken: 't0k' });
    await signingIn;

    session.handleUnauthenticated();

    expect(session.account()).toBeNull();
  });

  it('opens the reload dialog on a 426 reported elsewhere', () => {
    const { session } = setUp();
    expect(session.reloadRequired()).toBe(false);

    session.handleUpdateRequired();

    expect(session.reloadRequired()).toBe(true);
  });

  it('reflects an account changed elsewhere, syncing its language', () => {
    const { session, preferences } = setUp();
    session.setAccount({ ...ACCOUNT, emailVerified: true });

    expect(session.account()?.emailVerified).toBe(true);
    expect(preferences.setLanguageCalls).toEqual(['fr']);
  });
});
