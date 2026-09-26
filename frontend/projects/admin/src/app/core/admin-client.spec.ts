import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ApiProblem } from 'shared';
import { SESSION } from '../testing/admin-test-support';
import { AdminClient, CSRF_HEADER, fileNameOf } from './admin-client';
import { SessionState } from './session-state';

describe('AdminClient', () => {
  let client: AdminClient;
  let http: HttpTestingController;
  let state: SessionState;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    client = TestBed.inject(AdminClient);
    http = TestBed.inject(HttpTestingController);
    state = TestBed.inject(SessionState);
    state.set(SESSION);
  });

  afterEach(() => http.verify());

  it('reads without the CSRF token, and drops empty query parameters', async () => {
    const answer = client.request('get', '/api/admin/v1/users', {
      query: { q: 'lee', status: undefined, cursor: '' },
    });
    const request = http.expectOne((candidate) => candidate.url === '/api/admin/v1/users');
    expect(request.request.method).toBe('GET');
    expect(request.request.params.keys()).toEqual(['q']);
    expect(request.request.headers.has(CSRF_HEADER)).toBe(false);
    request.flush({ items: [], nextCursor: null });
    expect(await answer).toEqual({ items: [], nextCursor: null });
  });

  it('sends every change with the session’s CSRF token, and escapes path parameters', async () => {
    const answer = client.request('post', '/api/admin/v1/users/{id}/suspend', {
      path: { id: 'a b' },
      body: { reason: 'Spam' },
    });
    const request = http.expectOne('/api/admin/v1/users/a%20b/suspend');
    expect(request.request.headers.get(CSRF_HEADER)).toBe('csrf-1');
    expect(request.request.body).toEqual({ reason: 'Spam' });
    request.flush(null, { status: 204, statusText: 'No Content' });
    await answer;
  });

  it('turns a failure into its problem', async () => {
    const answer = client.request('delete', '/api/admin/v1/users/{id}', {
      path: { id: 'u1' },
      body: { reason: '' },
    });
    http
      .expectOne('/api/admin/v1/users/u1')
      .flush(
        { code: 'admin.reason_length', params: { min: 1, max: 1000 } },
        { status: 400, statusText: 'Bad Request' },
      );
    await expect(answer).rejects.toMatchObject({ code: 'admin.reason_length', status: 400 });
    expect(state.session()).toEqual(SESSION);
  });

  it('ends the session when the admin server says it is over', async () => {
    const answer = client.request('get', '/api/admin/v1/alerts', { query: { env: 'staging' } });
    http
      .expectOne((candidate) => candidate.url === '/api/admin/v1/alerts')
      .flush(
        { code: 'admin.unauthenticated', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );
    await expect(answer).rejects.toBeInstanceOf(ApiProblem);
    expect(state.session()).toBeNull();
  });

  it('downloads a file under the name the server gives it', async () => {
    const answer = client.download('/api/admin/v1/users/{id}/export', { id: 'u1' });
    const request = http.expectOne('/api/admin/v1/users/u1/export');
    expect(request.request.responseType).toBe('blob');
    request.flush(new Blob(['zip']), {
      headers: { 'Content-Disposition': 'attachment; filename="life-pixel-u1.zip"' },
    });
    const file = await answer;
    expect(file.fileName).toBe('life-pixel-u1.zip');
    expect(await file.blob.text()).toBe('zip');
  });

  it('reads the problem of a failed download from its body', async () => {
    const answer = client.download('/api/admin/v1/support-requests/{id}/screenshot', { id: 'r1' });
    const body = new Blob([JSON.stringify({ code: 'admin.screenshot_not_found', params: {} })]);
    http
      .expectOne('/api/admin/v1/support-requests/r1/screenshot')
      .flush(body, { status: 404, statusText: 'Not Found' });
    await expect(answer).rejects.toMatchObject({ code: 'admin.screenshot_not_found' });
  });
});

describe('fileNameOf', () => {
  it('reads the file name of a Content-Disposition header', () => {
    expect(fileNameOf('attachment; filename="export.zip"')).toBe('export.zip');
    expect(fileNameOf('attachment; filename=export.zip')).toBe('export.zip');
    expect(fileNameOf('attachment')).toBeNull();
    expect(fileNameOf(null)).toBeNull();
  });
});
