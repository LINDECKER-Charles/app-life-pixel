import { TestBed } from '@angular/core/testing';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { SESSION } from '../testing/admin-test-support';
import { AdminClient } from './admin-client';
import { SessionStore } from './session-store';

describe('SessionStore', () => {
  const request = vi.fn();
  let store: SessionStore;

  beforeEach(() => {
    request.mockReset();
    TestBed.configureTestingModule({
      providers: [{ provide: AdminClient, useValue: { request } }],
    });
    store = TestBed.inject(SessionStore);
  });

  it('reads the live session: the admin and the environments', async () => {
    request.mockResolvedValue(SESSION);

    await store.load();

    expect(request).toHaveBeenCalledWith('get', '/api/admin/v1/auth/session');
    expect(store.signedIn()).toBe(true);
    expect(store.admin()).toEqual(SESSION.admin);
    expect(store.environment()).toBe('staging');
    expect(store.monitoredEnvironments()).toEqual(['staging', 'production']);
  });

  it('is signed out when there is no live session', async () => {
    request.mockRejectedValue(new ApiProblem(401, 'admin.unauthenticated'));

    await store.load();

    expect(store.session()).toBeNull();
    expect(store.signedIn()).toBe(false);
    expect(store.environment()).toBe('');
  });

  it('signs in with the address, the password and the code', async () => {
    request.mockResolvedValue(SESSION);
    const credentials = { email: 'ops@example.com', password: 'secret', code: '123456' };

    await store.signIn(credentials);

    expect(request).toHaveBeenCalledWith('post', '/api/admin/v1/auth/sign-in', {
      body: credentials,
    });
    expect(store.signedIn()).toBe(true);
  });

  it('keeps a wrong sign-in signed out', async () => {
    request.mockRejectedValue(new ApiProblem(401, 'admin.invalid_credentials'));

    await expect(
      store.signIn({ email: 'ops@example.com', password: 'wrong', code: '000000' }),
    ).rejects.toMatchObject({ code: 'admin.invalid_credentials' });
    expect(store.signedIn()).toBe(false);
  });

  it('is signed out after signing out, even when the server cannot be reached', async () => {
    request.mockResolvedValueOnce(SESSION);
    await store.load();
    request.mockRejectedValueOnce(new ApiProblem(0, 'service.unavailable'));

    await expect(store.signOut()).rejects.toBeInstanceOf(ApiProblem);

    expect(store.signedIn()).toBe(false);
  });
});
