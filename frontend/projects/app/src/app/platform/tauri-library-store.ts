import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import type { LibraryStore } from '../library/library-store';
import {
  toLibraryFailure,
  type AnimationFilter,
  type AnimationSummary,
  type Page,
  type PageQuery,
  type Project,
} from '../library/library-types';
import { decodeBase64, encodeBase64 } from './base64';

/** Runs `request`, rejecting with a plain `{ code, params }` as the port promises. */
async function failingAsLibrary<T>(request: () => Promise<T>): Promise<T> {
  try {
    return await request();
  } catch (error: unknown) {
    throw toLibraryFailure(error);
  }
}

/**
 * The desktop's library (desktop.md, T2): `LibraryStore` over T1's `library_*` commands, in
 * process on the local library. Documents travel as base64 strings; a command's `CommandError`
 * already carries the plain `{ code, params }` every store rejects with.
 */
@Injectable()
export class TauriLibraryStore implements LibraryStore {
  listProjects(page: PageQuery = {}): Promise<Page<Project>> {
    return failingAsLibrary(() =>
      invoke('library_list_projects', { cursor: page.cursor, limit: page.limit }),
    );
  }

  createProject(name: string): Promise<Project> {
    return failingAsLibrary(() => invoke('library_create_project', { name }));
  }

  renameProject(id: string, name: string): Promise<Project> {
    return failingAsLibrary(() => invoke('library_rename_project', { id, name }));
  }

  duplicateProject(id: string, name: string): Promise<Project> {
    return failingAsLibrary(() => invoke('library_duplicate_project', { id, name }));
  }

  deleteProject(id: string): Promise<void> {
    return failingAsLibrary(() => invoke('library_delete_project', { id }));
  }

  listAnimations(filter: AnimationFilter, page: PageQuery = {}): Promise<Page<AnimationSummary>> {
    return failingAsLibrary(() =>
      invoke('library_list_animations', {
        projectId: filter.projectId,
        query: filter.query || undefined,
        cursor: page.cursor,
        limit: page.limit,
      }),
    );
  }

  createAnimation(projectId: string, document: Uint8Array): Promise<AnimationSummary> {
    return failingAsLibrary(() =>
      invoke('library_create_animation', { projectId, document: encodeBase64(document) }),
    );
  }

  openDocument(id: string): Promise<{ summary: AnimationSummary; document: Uint8Array }> {
    return failingAsLibrary(async () => {
      const opened = await invoke<{ summary: AnimationSummary; document: string }>(
        'library_open_document',
        { id },
      );
      return { summary: opened.summary, document: decodeBase64(opened.document) };
    });
  }

  saveDocument(id: string, document: Uint8Array, version: number): Promise<AnimationSummary> {
    return failingAsLibrary(() =>
      invoke('library_save_document', { id, version, document: encodeBase64(document) }),
    );
  }

  renameAnimation(id: string, title: string, version: number): Promise<AnimationSummary> {
    return failingAsLibrary(() => invoke('library_rename_animation', { id, version, title }));
  }

  moveAnimation(id: string, projectId: string): Promise<AnimationSummary> {
    return failingAsLibrary(() => invoke('library_move_animation', { id, projectId }));
  }

  duplicateAnimation(id: string, title: string, projectId?: string): Promise<AnimationSummary> {
    return failingAsLibrary(() => invoke('library_duplicate_animation', { id, title, projectId }));
  }

  deleteAnimation(id: string): Promise<void> {
    return failingAsLibrary(() => invoke('library_delete_animation', { id }));
  }

  usage(): Promise<{ usedBytes: number; limitBytes: number | null }> {
    return failingAsLibrary(() => invoke('library_usage'));
  }
}
