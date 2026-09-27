import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ApiProblem } from './api-problem';
import { CreatedAccessToken, TokensApi } from './tokens-api';

function setUp(): { api: TokensApi; http: HttpTestingController } {
  TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
  return { api: TestBed.inject(TokensApi), http: TestBed.inject(HttpTestingController) };
}

const TOKEN = {
  id: 't1',
  name: 'Claude Code',
  prefix: 'lp_pat_AbCd',
  scopes: ['read', 'write'],
  createdAt: '2026-09-01T10:00:00Z',
  expiresAt: '2026-11-30T10:00:00Z',
  lastUsedAt: null,
} as const;

const CREATED: CreatedAccessToken = {
  ...TOKEN,
  scopes: ['read', 'write'],
  token: `lp_pat_AbCd${'e'.repeat(39)}`,
  mcp: { serverName: 'life-pixel', url: 'https://life-pixel.app/mcp' },
};

describe('TokensApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('lists the active tokens', async () => {
    const { api, http } = setUp();
    const answer = api.list();
    const request = http.expectOne('/api/v1/tokens');
    expect(request.request.method).toBe('GET');
    request.flush([TOKEN]);
    await expect(answer).resolves.toEqual([TOKEN]);
  });

  it('creates a token and resolves to its secret and server', async () => {
    const { api, http } = setUp();
    const body = { name: 'Claude Code', scopes: ['read', 'write'], expiresInDays: 90 } as const;
    const answer = api.create({ ...body, scopes: [...body.scopes] });
    const request = http.expectOne('/api/v1/tokens');
    expect(request.request.method).toBe('POST');
    expect(request.request.body).toEqual(body);
    request.flush(CREATED, { status: 201, statusText: 'Created' });
    await expect(answer).resolves.toEqual(CREATED);
  });

  it('revokes a token by its id', async () => {
    const { api, http } = setUp();
    const answer = api.revoke('a b');
    const request = http.expectOne('/api/v1/tokens/a%20b');
    expect(request.request.method).toBe('DELETE');
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('rejects with the problem of a refused creation', async () => {
    const { api, http } = setUp();
    const answer = api.create({ name: 'CI', scopes: ['read'], expiresInDays: 90 });
    const problem = { code: 'token.limit', params: { max: 20 }, status: 409 };
    http.expectOne('/api/v1/tokens').flush(problem, { status: 409, statusText: 'Conflict' });
    const error = await answer.catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(ApiProblem);
    expect((error as ApiProblem).code).toBe('token.limit');
  });
});
