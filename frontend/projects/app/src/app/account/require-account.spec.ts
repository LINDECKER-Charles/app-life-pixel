import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { provideRouter, Router, UrlTree } from '@angular/router';
import { PreferencesStore } from '../settings/preferences-store';
import { FakePreferences } from './testing/account-test-support';
import { SessionStore } from './session-store';
import { requireAccount } from './require-account';

const ACCOUNT = {
  id: 'a1',
  email: 'lee@example.com',
  emailVerified: true,
  language: 'en',
  plan: 'free',
  storage: { usedBytes: 0, limitBytes: 1 },
  createdAt: '2024-01-01T00:00:00Z',
};

function setUp(): SessionStore {
  TestBed.configureTestingModule({
    providers: [
      provideHttpClient(),
      provideHttpClientTesting(),
      provideRouter([]),
      { provide: PreferencesStore, useClass: FakePreferences },
    ],
  });
  return TestBed.inject(SessionStore);
}

describe('requireAccount', () => {
  it('lets a signed-in visitor through', () => {
    const session = setUp();
    session.setAccount(ACCOUNT);

    const result = TestBed.runInInjectionContext(() =>
      requireAccount({} as never, { url: '/account' } as never),
    );

    expect(result).toBe(true);
  });

  it('sends a stranger to sign in, with a returnUrl back to the page they asked for', () => {
    setUp();

    const result = TestBed.runInInjectionContext(() =>
      requireAccount({} as never, { url: '/account' } as never),
    );

    expect(result).toBeInstanceOf(UrlTree);
    expect(TestBed.inject(Router).serializeUrl(result as UrlTree)).toBe(
      '/sign-in?returnUrl=%2Faccount',
    );
  });
});
