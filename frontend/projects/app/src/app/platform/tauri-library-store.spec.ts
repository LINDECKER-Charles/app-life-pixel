import { TestBed } from '@angular/core/testing';
import { mockIPC } from '@tauri-apps/api/mocks';
import { encodeBase64 } from './base64';
import { clearTauriMocks } from './testing/clear-tauri-mocks';
import { TauriLibraryStore } from './tauri-library-store';

/** The bytes of an ASCII text, in this realm's `Uint8Array` (`TextEncoder`'s is a different one). */
function bytes(text: string): Uint8Array {
  return Uint8Array.from(text, (character) => character.charCodeAt(0));
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

interface Recorded {
  command: string;
  payload: Record<string, unknown>;
}

function setUp(handler: (call: Recorded) => unknown): {
  store: TauriLibraryStore;
  calls: Recorded[];
} {
  const calls: Recorded[] = [];
  mockIPC((command, payload) => {
    const call = { command, payload: (payload ?? {}) as Record<string, unknown> };
    calls.push(call);
    return handler(call);
  });
  TestBed.configureTestingModule({ providers: [TauriLibraryStore] });
  return { store: TestBed.inject(TauriLibraryStore), calls };
}

describe('TauriLibraryStore', () => {
  afterEach(() => clearTauriMocks());

  it('lists projects by page, cursor and limit as the command names them', async () => {
    const { store, calls } = setUp(() => ({ items: [PROJECT], nextCursor: null }));

    await expect(store.listProjects({ cursor: 'c1', limit: 50 })).resolves.toEqual({
      items: [PROJECT],
      nextCursor: null,
    });
    expect(calls).toEqual([
      { command: 'library_list_projects', payload: { cursor: 'c1', limit: 50 } },
    ]);
  });

  it('creates, renames, duplicates and deletes projects', async () => {
    const { store, calls } = setUp(({ command }) =>
      command === 'library_delete_project' ? null : PROJECT,
    );

    await store.createProject('Sprites');
    await store.renameProject('p1', 'Heroes');
    await store.duplicateProject('p1', 'Copy of Heroes');
    await store.deleteProject('p1');

    expect(calls).toEqual([
      { command: 'library_create_project', payload: { name: 'Sprites' } },
      { command: 'library_rename_project', payload: { id: 'p1', name: 'Heroes' } },
      { command: 'library_duplicate_project', payload: { id: 'p1', name: 'Copy of Heroes' } },
      { command: 'library_delete_project', payload: { id: 'p1' } },
    ]);
  });

  it('lists the animations of a filter, base64-encodes a created document', async () => {
    const document = bytes('{"title":"Mascot"}');
    const { store, calls } = setUp(({ command }) =>
      command === 'library_list_animations'
        ? { items: [ANIMATION], nextCursor: 'next' }
        : ANIMATION,
    );

    await store.listAnimations({ projectId: 'p1', query: '' }, { limit: 50 });
    await store.createAnimation('p1', document);

    expect(calls).toEqual([
      {
        command: 'library_list_animations',
        payload: { projectId: 'p1', query: undefined, cursor: undefined, limit: 50 },
      },
      {
        command: 'library_create_animation',
        payload: { projectId: 'p1', document: encodeBase64(document) },
      },
    ]);
  });

  it('opens a document, decoding the base64 body back to bytes', async () => {
    const document = bytes('{"title":"Mascot"}');
    const { store } = setUp(() => ({ summary: ANIMATION, document: encodeBase64(document) }));

    const opened = await store.openDocument('a1');

    expect(opened.summary).toEqual(ANIMATION);
    expect(opened.document).toEqual(document);
  });

  it('saves a document under its version, renames, moves, duplicates and deletes', async () => {
    const document = bytes('{}');
    const { store, calls } = setUp(({ command }) =>
      command === 'library_delete_animation' ? null : ANIMATION,
    );

    await store.saveDocument('a1', document, 4);
    await store.renameAnimation('a1', 'Hero', 5);
    await store.moveAnimation('a1', 'p2');
    await store.duplicateAnimation('a1', 'Copy of Mascot');
    await store.deleteAnimation('a1');

    expect(calls).toEqual([
      {
        command: 'library_save_document',
        payload: { id: 'a1', version: 4, document: encodeBase64(document) },
      },
      { command: 'library_rename_animation', payload: { id: 'a1', version: 5, title: 'Hero' } },
      { command: 'library_move_animation', payload: { id: 'a1', projectId: 'p2' } },
      {
        command: 'library_duplicate_animation',
        payload: { id: 'a1', title: 'Copy of Mascot', projectId: undefined },
      },
      { command: 'library_delete_animation', payload: { id: 'a1' } },
    ]);
  });

  it('reads the usage; the local library has no quota', async () => {
    const { store } = setUp(() => ({ usedBytes: 512, limitBytes: null }));

    await expect(store.usage()).resolves.toEqual({ usedBytes: 512, limitBytes: null });
  });

  it('rejects with the command’s plain code and params', async () => {
    const { store } = setUp(() => {
      throw { code: 'document.version_conflict', params: { current: 4 } };
    });

    await expect(store.saveDocument('a1', new Uint8Array(), 3)).rejects.toEqual({
      code: 'document.version_conflict',
      params: { current: 4 },
    });
  });

  it('rejects with internal.error for a failure that is not a command error', async () => {
    const { store } = setUp(() => {
      throw new TypeError('boom');
    });

    await expect(store.listProjects()).rejects.toEqual({ code: 'internal.error', params: {} });
  });
});
