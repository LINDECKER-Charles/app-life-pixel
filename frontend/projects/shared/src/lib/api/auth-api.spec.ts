import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { AuthApi } from './auth-api';

function setUp(): { api: AuthApi; http: HttpTestingController } {
  TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
  return { api: TestBed.inject(AuthApi), http: TestBed.inject(HttpTestingController) };
}

const SESSION = {
  account: {
    id: 'a1',
    email: 'lee@example.com',
    emailVerified: false,
    language: 'en',
    plan: 'free',
    storage: { usedBytes: 0, limitBytes: 100 },
    createdAt: '2024-01-01T00:00:00Z',
  },
  csrfToken: 't0k',
};

describe('AuthApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('reads the current session', async () => {
    const { api, http } = setUp();
    const answer = api.session();
    const request = http.expectOne('/api/v1/auth/session');
    expect(request.request.method).toBe('GET');
    request.flush(SESSION);
    await expect(answer).resolves.toEqual(SESSION);
  });

  it('signs up with the email, password and language', async () => {
    const { api, http } = setUp();
    const answer = api.signUp('lee@example.com', 'a very long password', 'fr');
    const request = http.expectOne('/api/v1/auth/sign-up');
    expect(request.request.method).toBe('POST');
    expect(request.request.body).toEqual({
      email: 'lee@example.com',
      password: 'a very long password',
      language: 'fr',
    });
    request.flush(SESSION, { status: 201, statusText: 'Created' });
    await expect(answer).resolves.toEqual(SESSION);
  });

  it('signs in with the email and password', async () => {
    const { api, http } = setUp();
    const answer = api.signIn('lee@example.com', 'a very long password');
    const request = http.expectOne('/api/v1/auth/sign-in');
    expect(request.request.body).toEqual({
      email: 'lee@example.com',
      password: 'a very long password',
    });
    request.flush(SESSION);
    await expect(answer).resolves.toEqual(SESSION);
  });

  it('signs out', async () => {
    const { api, http } = setUp();
    const answer = api.signOut();
    const request = http.expectOne('/api/v1/auth/sign-out');
    expect(request.request.method).toBe('POST');
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('verifies an email with its token', async () => {
    const { api, http } = setUp();
    const answer = api.verifyEmail('tok-1');
    const request = http.expectOne('/api/v1/auth/verify-email');
    expect(request.request.body).toEqual({ token: 'tok-1' });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('resends the verification email', async () => {
    const { api, http } = setUp();
    const answer = api.resendVerificationEmail();
    const request = http.expectOne('/api/v1/auth/verify-email/resend');
    expect(request.request.body).toBeNull();
    request.flush(null, { status: 202, statusText: 'Accepted' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('requests a password reset link', async () => {
    const { api, http } = setUp();
    const answer = api.requestPasswordReset('lee@example.com');
    const request = http.expectOne('/api/v1/auth/password-reset');
    expect(request.request.body).toEqual({ email: 'lee@example.com' });
    request.flush(null, { status: 202, statusText: 'Accepted' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('confirms a password reset with its token', async () => {
    const { api, http } = setUp();
    const answer = api.confirmPasswordReset('tok-1', 'a very long password');
    const request = http.expectOne('/api/v1/auth/password-reset/confirm');
    expect(request.request.body).toEqual({ token: 'tok-1', password: 'a very long password' });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('changes the signed-in account password', async () => {
    const { api, http } = setUp();
    const answer = api.changePassword('old password 1', 'new password 12');
    const request = http.expectOne('/api/v1/auth/password');
    expect(request.request.method).toBe('PUT');
    expect(request.request.body).toEqual({
      currentPassword: 'old password 1',
      newPassword: 'new password 12',
    });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });
});
