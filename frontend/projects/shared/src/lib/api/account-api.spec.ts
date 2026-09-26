import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { AccountApi } from './account-api';

function setUp(): { api: AccountApi; http: HttpTestingController } {
  TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
  return { api: TestBed.inject(AccountApi), http: TestBed.inject(HttpTestingController) };
}

const ACCOUNT = {
  id: 'a1',
  email: 'lee@example.com',
  emailVerified: true,
  language: 'en',
  plan: 'free',
  storage: { usedBytes: 512, limitBytes: 100_000 },
  createdAt: '2024-01-01T00:00:00Z',
};

describe('AccountApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('reads the signed-in account', async () => {
    const { api, http } = setUp();
    const answer = api.get();
    const request = http.expectOne('/api/v1/account');
    expect(request.request.method).toBe('GET');
    request.flush(ACCOUNT);
    await expect(answer).resolves.toEqual(ACCOUNT);
  });

  it('patches the language', async () => {
    const { api, http } = setUp();
    const answer = api.updateLanguage('fr');
    const request = http.expectOne('/api/v1/account');
    expect(request.request.method).toBe('PATCH');
    expect(request.request.body).toEqual({ language: 'fr' });
    request.flush({ ...ACCOUNT, language: 'fr' });
    await expect(answer).resolves.toEqual({ ...ACCOUNT, language: 'fr' });
  });

  it('deletes the account with its password', async () => {
    const { api, http } = setUp();
    const answer = api.delete('a very long password');
    const request = http.expectOne('/api/v1/account');
    expect(request.request.method).toBe('DELETE');
    expect(request.request.body).toEqual({ password: 'a very long password' });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('names the export route, never fetched as JSON', () => {
    const { api } = setUp();
    expect(api.exportUrl).toBe('/api/v1/account/export');
  });
});
