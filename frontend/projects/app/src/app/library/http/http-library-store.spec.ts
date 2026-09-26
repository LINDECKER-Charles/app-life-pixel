import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ApiClient, ApiProblem } from 'shared';
import { HttpLibraryStore } from './http-library-store';

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

type Request = (method: string, path: string, options?: object) => Promise<unknown>;

function setUp(): {
  store: HttpLibraryStore;
  request: ReturnType<typeof vi.fn<Request>>;
  http: HttpTestingController;
} {
  const request = vi.fn<Request>();
  TestBed.configureTestingModule({
    providers: [
      provideHttpClient(),
      provideHttpClientTesting(),
      { provide: ApiClient, useValue: { request } },
    ],
  });
  return {
    store: TestBed.inject(HttpLibraryStore),
    request,
    http: TestBed.inject(HttpTestingController),
  };
}

describe('HttpLibraryStore', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('lists projects by page, with a cursor even on the last page', async () => {
    const { store, request } = setUp();
    request.mockResolvedValue({ items: [PROJECT] });

    await expect(store.listProjects({ cursor: 'c1', limit: 50 })).resolves.toEqual({
      items: [PROJECT],
      nextCursor: null,
    });
    expect(request).toHaveBeenCalledWith('get', '/api/v1/projects', {
      query: { cursor: 'c1', limit: 50 },
    });
  });

  it('lists the animations of a filter: a project, a search, or all', async () => {
    const { store, request } = setUp();
    request.mockResolvedValue({ items: [ANIMATION], nextCursor: 'next' });

    await store.listAnimations({ projectId: 'p1' }, { limit: 50 });
    await store.listAnimations({ query: 'mas' }, { cursor: 'next', limit: 50 });
    await store.listAnimations({ query: '' });

    expect(request.mock.calls.map(([, , options]) => options)).toEqual([
      { query: { project: 'p1', q: undefined, limit: 50 } },
      { query: { project: undefined, q: 'mas', cursor: 'next', limit: 50 } },
      { query: { project: undefined, q: undefined } },
    ]);
  });

  it('creates, renames, duplicates and deletes projects', async () => {
    const { store, request } = setUp();
    request.mockResolvedValue(PROJECT);

    await store.createProject('Sprites');
    await store.renameProject('p1', 'Heroes');
    await store.duplicateProject('p1', 'Copy of Heroes');
    request.mockResolvedValue(undefined);
    await store.deleteProject('p1');

    expect(request.mock.calls).toEqual([
      ['post', '/api/v1/projects', { body: { name: 'Sprites' } }],
      ['patch', '/api/v1/projects/{id}', { path: { id: 'p1' }, body: { name: 'Heroes' } }],
      [
        'post',
        '/api/v1/projects/{id}/duplicate',
        { path: { id: 'p1' }, body: { name: 'Copy of Heroes' } },
      ],
      ['delete', '/api/v1/projects/{id}', { path: { id: 'p1' } }],
    ]);
  });

  it('moves, duplicates and deletes animations', async () => {
    const { store, request } = setUp();
    request.mockResolvedValue(ANIMATION);

    await store.moveAnimation('a1', 'p2');
    await store.duplicateAnimation('a1', 'Copy of Mascot');
    request.mockResolvedValue(undefined);
    await store.deleteAnimation('a1');

    expect(request.mock.calls).toEqual([
      ['patch', '/api/v1/animations/{id}', { path: { id: 'a1' }, body: { projectId: 'p2' } }],
      [
        'post',
        '/api/v1/animations/{id}/duplicate',
        { path: { id: 'a1' }, body: { title: 'Copy of Mascot', projectId: undefined } },
      ],
      ['delete', '/api/v1/animations/{id}', { path: { id: 'a1' } }],
    ]);
  });

  it('opens a document with the version its ETag names', async () => {
    const { store, request, http } = setUp();
    request.mockResolvedValue(ANIMATION);

    const opened = store.openDocument('a1');
    const document = await vi.waitFor(() => http.expectOne('/api/v1/animations/a1/document'));
    document.flush(bytes('{"title":"Mascot"}').buffer, { headers: { ETag: '"5"' } });

    const { summary, document: body } = await opened;
    expect(request).toHaveBeenCalledWith('get', '/api/v1/animations/{id}', { path: { id: 'a1' } });
    expect(summary).toEqual({ ...ANIMATION, version: 5 });
    expect(Array.from(body)).toEqual(Array.from(bytes('{"title":"Mascot"}')));
  });

  it('creates and saves documents, and renames under the version', async () => {
    const { store, http } = setUp();

    const created = store.createAnimation('p1', bytes('{}'));
    http
      .expectOne('/api/v1/projects/p1/animations')
      .flush(ANIMATION, { status: 201, statusText: 'Created' });
    const saved = store.saveDocument('a1', bytes('{}'), 4);
    const save = http.expectOne({ method: 'PUT', url: '/api/v1/animations/a1/document' });
    expect(save.request.headers.get('If-Match')).toBe('"4"');
    save.flush({ ...ANIMATION, version: 5 });
    const renamed = store.renameAnimation('a1', 'Hero', 5);
    const rename = http.expectOne({ method: 'PATCH', url: '/api/v1/animations/a1' });
    expect(rename.request.headers.get('If-Match')).toBe('"5"');
    rename.flush({ ...ANIMATION, title: 'Hero', version: 6 });

    await expect(created).resolves.toEqual(ANIMATION);
    await expect(saved).resolves.toMatchObject({ version: 5 });
    await expect(renamed).resolves.toMatchObject({ title: 'Hero', version: 6 });
  });

  it('rejects with the plain code and params of a problem', async () => {
    const { store, http } = setUp();

    const saved = store.saveDocument('a1', bytes('{}'), 3);
    http
      .expectOne('/api/v1/animations/a1/document')
      .flush(
        { code: 'document.version_conflict', params: { current: 4 } },
        { status: 412, statusText: 'Precondition Failed' },
      );

    const failure: unknown = await saved.catch((error: unknown) => error);
    expect(failure).toEqual({ code: 'document.version_conflict', params: { current: 4 } });
    expect(failure).not.toBeInstanceOf(ApiProblem);
  });

  it('rejects with internal.error for a failure that is not a problem', async () => {
    const { store, request } = setUp();
    request.mockRejectedValue(new TypeError('boom'));

    await expect(store.listProjects()).rejects.toEqual({ code: 'internal.error', params: {} });
  });

  it('reads the usage of the account, its limit null when it has none', async () => {
    const { store, request } = setUp();
    request.mockResolvedValue({ id: 'u1', storage: { usedBytes: 512 } });

    await expect(store.usage()).resolves.toEqual({ usedBytes: 512, limitBytes: null });
    expect(request).toHaveBeenCalledWith('get', '/api/v1/account');
  });
});
