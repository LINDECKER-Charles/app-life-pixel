import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { API_HEADERS } from './api-headers';
import { ApiProblem } from './api-problem';
import { ANIMATION_DOCUMENT_TYPE, LibraryApi } from './library-api';

const CSRF = { provide: API_HEADERS, useValue: () => ({ 'X-CSRF-Token': 't0k' }), multi: true };

function setUp(): { api: LibraryApi; http: HttpTestingController } {
  TestBed.configureTestingModule({
    providers: [provideHttpClient(), provideHttpClientTesting(), CSRF],
  });
  return { api: TestBed.inject(LibraryApi), http: TestBed.inject(HttpTestingController) };
}

const PROJECT = {
  id: 'p1',
  name: 'Sprites',
  animationCount: 2,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-02T00:00:00Z',
};

const ANIMATION = {
  id: 'a1',
  projectId: 'p1',
  title: 'Mascot',
  width: 32,
  height: 32,
  frameCount: 8,
  documentBytes: 5120,
  version: 4,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-02T00:00:00Z',
};

/** The bytes of an ASCII text, in this realm's `ArrayBuffer` as `HttpClient` expects it. */
function bytes(text: string): Uint8Array<ArrayBuffer> {
  return Uint8Array.from(text, (character) => character.charCodeAt(0));
}

function textOf(body: unknown): string {
  expect(Object.prototype.toString.call(body)).toBe('[object ArrayBuffer]');
  return String.fromCharCode(...new Uint8Array(body as ArrayBuffer));
}

const DOCUMENT = bytes('{"title":"Mascot"}');

function problemBytes(code: string, params: object): ArrayBuffer {
  return bytes(JSON.stringify({ code, params })).buffer;
}

describe('LibraryApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('lists projects with a cursor and a page size', async () => {
    const { api, http } = setUp();
    const answer = api.listProjects({ cursor: 'c1', limit: 50 });
    const request = http.expectOne((candidate) => candidate.url === '/api/v1/projects');
    expect(request.request.params.get('cursor')).toBe('c1');
    expect(request.request.params.get('limit')).toBe('50');
    request.flush({ items: [PROJECT], nextCursor: null });
    await expect(answer).resolves.toEqual({ items: [PROJECT], nextCursor: null });
  });

  it('creates, renames, duplicates and deletes projects', async () => {
    const { api, http } = setUp();
    const created = api.createProject('Sprites');
    const renamed = api.renameProject('p1', 'Heroes');
    const copied = api.duplicateProject('p1', 'Copy of Heroes');
    const deleted = api.deleteProject('p1');
    const [create, duplicate] = http.match((candidate) => candidate.method === 'POST');
    expect([create?.request.url, create?.request.body]).toEqual([
      '/api/v1/projects',
      { name: 'Sprites' },
    ]);
    expect(duplicate?.request.url).toBe('/api/v1/projects/p1/duplicate');
    const rename = http.expectOne({ method: 'PATCH', url: '/api/v1/projects/p1' });
    expect(rename.request.body).toEqual({ name: 'Heroes' });
    create?.flush(PROJECT);
    duplicate?.flush(PROJECT);
    rename.flush(PROJECT);
    http
      .expectOne({ method: 'DELETE', url: '/api/v1/projects/p1' })
      .flush(null, { status: 204, statusText: 'No Content' });
    await Promise.all([created, renamed, copied]);
    await expect(deleted).resolves.toBeUndefined();
  });

  it('lists animations of a project, or matching a search', async () => {
    const { api, http } = setUp();
    const answer = api.listAnimations({ project: 'p1', q: 'mas', limit: 50 });
    const request = http.expectOne((candidate) => candidate.url === '/api/v1/animations');
    expect(request.request.params.get('project')).toBe('p1');
    expect(request.request.params.get('q')).toBe('mas');
    request.flush({ items: [ANIMATION], nextCursor: 'next' });
    await expect(answer).resolves.toEqual({ items: [ANIMATION], nextCursor: 'next' });
  });

  it('creates an animation from the bytes of its document', async () => {
    const { api, http } = setUp();
    const answer = api.createAnimation('p1', DOCUMENT);
    const request = http.expectOne({ method: 'POST', url: '/api/v1/projects/p1/animations' });
    expect(request.request.headers.get('Content-Type')).toBe(ANIMATION_DOCUMENT_TYPE);
    expect(request.request.headers.get('X-CSRF-Token')).toBe('t0k');
    expect(request.request.headers.get('Life-Pixel-Client')).toBe('web/0.0.0');
    expect(textOf(request.request.body)).toBe('{"title":"Mascot"}');
    request.flush(ANIMATION, { status: 201, statusText: 'Created', headers: { ETag: '"4"' } });
    await expect(answer).resolves.toEqual(ANIMATION);
  });

  it('reads a document and the version its ETag names', async () => {
    const { api, http } = setUp();
    const answer = api.getDocument('a1');
    const request = http.expectOne('/api/v1/animations/a1/document');
    expect(request.request.responseType).toBe('arraybuffer');
    request.flush(bytes('{"title":"Mascot"}').buffer, { headers: { ETag: '"4"' } });
    const { document, version } = await answer;
    expect(textOf(document.slice().buffer)).toBe('{"title":"Mascot"}');
    expect(version).toBe(4);
  });

  it('saves a document under If-Match', async () => {
    const { api, http } = setUp();
    const answer = api.saveDocument('a1', DOCUMENT, 4);
    const request = http.expectOne({ method: 'PUT', url: '/api/v1/animations/a1/document' });
    expect(request.request.headers.get('If-Match')).toBe('"4"');
    expect(request.request.headers.get('Content-Type')).toBe(ANIMATION_DOCUMENT_TYPE);
    request.flush({ ...ANIMATION, version: 5 }, { headers: { ETag: '"5"' } });
    await expect(answer).resolves.toEqual({ ...ANIMATION, version: 5 });
  });

  it('renames under If-Match, and moves without', async () => {
    const { api, http } = setUp();
    const renamed = api.renameAnimation('a1', 'Hero', 4);
    const moved = api.moveAnimation('a1', 'p2');
    const [rename, move] = http.match({ method: 'PATCH', url: '/api/v1/animations/a1' });
    expect(rename?.request.headers.get('If-Match')).toBe('"4"');
    expect(rename?.request.body).toEqual({ title: 'Hero' });
    expect(move?.request.headers.has('If-Match')).toBe(false);
    expect(move?.request.body).toEqual({ projectId: 'p2' });
    rename?.flush({ ...ANIMATION, title: 'Hero', version: 5 });
    move?.flush({ ...ANIMATION, projectId: 'p2' });
    await expect(renamed).resolves.toMatchObject({ title: 'Hero', version: 5 });
    await expect(moved).resolves.toMatchObject({ projectId: 'p2' });
  });

  it('duplicates and deletes animations', async () => {
    const { api, http } = setUp();
    const copied = api.duplicateAnimation('a1', { title: 'Copy of Mascot', projectId: 'p2' });
    const deleted = api.deleteAnimation('a1');
    const copy = http.expectOne('/api/v1/animations/a1/duplicate');
    expect(copy.request.body).toEqual({ title: 'Copy of Mascot', projectId: 'p2' });
    copy.flush({ ...ANIMATION, id: 'a2' }, { status: 201, statusText: 'Created' });
    http
      .expectOne({ method: 'DELETE', url: '/api/v1/animations/a1' })
      .flush(null, { status: 204, statusText: 'No Content' });
    await expect(copied).resolves.toMatchObject({ id: 'a2' });
    await expect(deleted).resolves.toBeUndefined();
  });

  it('throws the problem of a stale save', async () => {
    const { api, http } = setUp();
    const answer = api.saveDocument('a1', DOCUMENT, 3);
    http
      .expectOne('/api/v1/animations/a1/document')
      .flush(
        { code: 'document.version_conflict', params: { current: 4 } },
        { status: 412, statusText: 'Precondition Failed' },
      );
    await expect(answer).rejects.toEqual(
      new ApiProblem(412, 'document.version_conflict', { current: 4 }),
    );
  });

  it('reads the problem of a document it asked as bytes', async () => {
    const { api, http } = setUp();
    const answer = api.getDocument('gone');
    http
      .expectOne('/api/v1/animations/gone/document')
      .flush(problemBytes('library.animation_not_found', {}), {
        status: 404,
        statusText: 'Not Found',
      });
    const failure: unknown = await answer.catch((error: unknown) => error);
    expect(failure).toBeInstanceOf(ApiProblem);
    expect((failure as ApiProblem).code).toBe('library.animation_not_found');
  });
});
