import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { Provider } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { ApiClient } from './api-client';
import { API_HEADERS, ApiHeaderSource, ApiRequestInfo, LIFE_PIXEL_CLIENT } from './api-headers';
import { ApiProblem } from './api-problem';

type UntypedRequest = (method: string, path: string, options?: object) => Promise<unknown>;

function setUp(...providers: Provider[]): { client: ApiClient; http: HttpTestingController } {
  TestBed.configureTestingModule({
    providers: [provideHttpClient(), provideHttpClientTesting(), ...providers],
  });
  return { client: TestBed.inject(ApiClient), http: TestBed.inject(HttpTestingController) };
}

function headerSource(source: ApiHeaderSource): Provider {
  return { provide: API_HEADERS, useValue: source, multi: true };
}

/** The client without its types, for routes later tasks add to the description. */
function untyped(client: ApiClient): UntypedRequest {
  return client.request.bind(client) as unknown as UntypedRequest;
}

/** A CSRF header on every request that may change something, as H5's source adds it. */
function csrfHeader({ method }: ApiRequestInfo): Readonly<Record<string, string>> {
  if (method === 'get') {
    return {};
  }
  return { 'X-CSRF-Token': 't0k' };
}

async function failureOf(answer: Promise<unknown>): Promise<ApiProblem> {
  const failure: unknown = await answer.then(
    () => new Error('the request succeeded'),
    (error: unknown) => error,
  );
  expect(failure).toBeInstanceOf(ApiProblem);
  return failure as ApiProblem;
}

describe('ApiClient', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('sends Life-Pixel-Client and resolves to the typed answer', async () => {
    const { client, http } = setUp();
    const answer = client.request('get', '/healthz');
    const request = http.expectOne('/healthz');
    expect(request.request.method).toBe('GET');
    expect(request.request.headers.get('Life-Pixel-Client')).toBe('web/0.0.0');
    request.flush({ status: 'ok' });
    const health: { status: 'ok' } = await answer;
    expect(health).toEqual({ status: 'ok' });
  });

  it('types each path by the methods of its operations', () => {
    const { client } = setUp();
    // @ts-expect-error: /healthz has no POST operation.
    const post = () => client.request('post', '/healthz');
    // @ts-expect-error: no operation has this path.
    const unknown = () => client.request('get', '/api/v1/nothing');
    expect([post, unknown]).toHaveLength(2);
  });

  it('sends the platform and version the app provides', async () => {
    const { client, http } = setUp({ provide: LIFE_PIXEL_CLIENT, useValue: 'desktop/1.2.3' });
    const answer = client.request('get', '/healthz');
    const request = http.expectOne('/healthz');
    expect(request.request.headers.get('Life-Pixel-Client')).toBe('desktop/1.2.3');
    request.flush({ status: 'ok' });
    await answer;
  });

  it('fills the path, encodes the query and sends the JSON body', async () => {
    const { client, http } = setUp();
    const answer = untyped(client)('post', '/api/v1/projects/{projectId}/animations', {
      path: { projectId: 'a b/c' },
      query: { limit: 2, after: undefined, tags: ['x', 'y'] },
      body: { title: 'Walk cycle' },
    });
    const request = http.expectOne((candidate) => candidate.url.startsWith('/api/v1/projects/'));
    expect(request.request.method).toBe('POST');
    expect(request.request.url).toBe('/api/v1/projects/a%20b%2Fc/animations');
    expect(request.request.params.get('limit')).toBe('2');
    expect(request.request.params.has('after')).toBe(false);
    expect(request.request.params.getAll('tags')).toEqual(['x', 'y']);
    expect(request.request.body).toEqual({ title: 'Walk cycle' });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await expect(answer).resolves.toBeUndefined();
  });

  it('adds the headers of every source, such as the CSRF token', async () => {
    const csrf = headerSource(csrfHeader);
    const trace = headerSource(({ url }) => ({ 'X-Test-Url': url }));
    const { client, http } = setUp(csrf, trace);
    const post = untyped(client)('post', '/api/v1/session');
    const get = client.request('get', '/healthz');
    const [posted, got] = [http.expectOne('/api/v1/session'), http.expectOne('/healthz')];
    expect(posted.request.headers.get('X-CSRF-Token')).toBe('t0k');
    expect(posted.request.headers.get('X-Test-Url')).toBe('/api/v1/session');
    expect(posted.request.headers.get('Life-Pixel-Client')).toBe('web/0.0.0');
    expect(got.request.headers.has('X-CSRF-Token')).toBe(false);
    posted.flush({});
    got.flush({ status: 'ok' });
    await Promise.all([post, get]);
  });

  it('throws the problem the server answers', async () => {
    const { client, http } = setUp();
    const answer = untyped(client)('put', '/api/v1/animations/{id}', { path: { id: 'a1' } });
    const params = { used: 99, limit: 100, requested: 2 };
    const body = { type: 'urn:life-pixel:problem:quota.storage_exceeded', status: 409, params };
    http
      .expectOne('/api/v1/animations/a1')
      .flush({ ...body, code: 'quota.storage_exceeded' }, { status: 409, statusText: 'Conflict' });
    const problem = await failureOf(answer);
    expect(problem.status).toBe(409);
    expect(problem.code).toBe('quota.storage_exceeded');
    expect(problem.params).toEqual(params);
  });

  it('turns a failure without a problem into service.unavailable or internal.error', async () => {
    const { client, http } = setUp();
    const failures = [
      { status: 0, code: 'service.unavailable' },
      { status: 502, code: 'service.unavailable' },
      { status: 504, code: 'service.unavailable' },
      { status: 500, code: 'internal.error' },
    ];
    for (const { status, code } of failures) {
      const answer = client.request('get', '/healthz');
      const request = http.expectOne('/healthz');
      if (status === 0) {
        request.error(new ProgressEvent('error'));
      } else {
        request.flush('<html>Bad gateway</html>', { status, statusText: 'Failure' });
      }
      const problem = await failureOf(answer);
      expect([problem.status, problem.code, problem.params]).toEqual([status, code, {}]);
    }
  });
});
