import { inject, Injectable } from '@angular/core';
import { AccountApi, LibraryApi } from 'shared';
import type { LibraryStore } from '../library-store';
import {
  toLibraryFailure,
  type AnimationFilter,
  type AnimationSummary,
  type Page,
  type PageQuery,
  type Project,
} from '../library-types';

/** Runs `request`, rejecting with a plain `{ code, params }` as the port promises. */
async function failingAsLibrary<T>(request: () => Promise<T>): Promise<T> {
  try {
    return await request();
  } catch (error: unknown) {
    throw toLibraryFailure(error);
  }
}

/** A page as the API answers it, with `nextCursor` always present. */
function pageOf<T>(page: { items: T[]; nextCursor?: string | null }): Page<T> {
  return { items: page.items, nextCursor: page.nextCursor ?? null };
}

/**
 * The hosted account's library (accounts.md, H8): `LibraryStore` over H6's routes through
 * `LibraryApi`, the account's usage through `AccountApi`. An `ApiProblem` becomes the plain
 * `{ code, params }` every store rejects with.
 */
@Injectable({ providedIn: 'root' })
export class HttpLibraryStore implements LibraryStore {
  private readonly api = inject(LibraryApi);
  private readonly accountApi = inject(AccountApi);

  listProjects(page: PageQuery = {}): Promise<Page<Project>> {
    return failingAsLibrary(async () => pageOf(await this.api.listProjects(page)));
  }

  createProject(name: string): Promise<Project> {
    return failingAsLibrary(() => this.api.createProject(name));
  }

  renameProject(id: string, name: string): Promise<Project> {
    return failingAsLibrary(() => this.api.renameProject(id, name));
  }

  duplicateProject(id: string, name: string): Promise<Project> {
    return failingAsLibrary(() => this.api.duplicateProject(id, name));
  }

  deleteProject(id: string): Promise<void> {
    return failingAsLibrary(() => this.api.deleteProject(id));
  }

  listAnimations(filter: AnimationFilter, page: PageQuery = {}): Promise<Page<AnimationSummary>> {
    const query = { project: filter.projectId, q: filter.query || undefined, ...page };
    return failingAsLibrary(async () => pageOf(await this.api.listAnimations(query)));
  }

  createAnimation(projectId: string, document: Uint8Array): Promise<AnimationSummary> {
    return failingAsLibrary(() => this.api.createAnimation(projectId, document));
  }

  /** The summary and the document; the document's `ETag` is the version a save then names. */
  openDocument(id: string): Promise<{ summary: AnimationSummary; document: Uint8Array }> {
    return failingAsLibrary(async () => {
      const [summary, { document, version }] = await Promise.all([
        this.api.getAnimation(id),
        this.api.getDocument(id),
      ]);
      return { summary: { ...summary, version }, document };
    });
  }

  saveDocument(id: string, document: Uint8Array, version: number): Promise<AnimationSummary> {
    return failingAsLibrary(() => this.api.saveDocument(id, document, version));
  }

  renameAnimation(id: string, title: string, version: number): Promise<AnimationSummary> {
    return failingAsLibrary(() => this.api.renameAnimation(id, title, version));
  }

  moveAnimation(id: string, projectId: string): Promise<AnimationSummary> {
    return failingAsLibrary(() => this.api.moveAnimation(id, projectId));
  }

  duplicateAnimation(id: string, title: string, projectId?: string): Promise<AnimationSummary> {
    return failingAsLibrary(() => this.api.duplicateAnimation(id, { title, projectId }));
  }

  deleteAnimation(id: string): Promise<void> {
    return failingAsLibrary(() => this.api.deleteAnimation(id));
  }

  usage(): Promise<{ usedBytes: number; limitBytes: number | null }> {
    return failingAsLibrary(async () => {
      const { storage } = await this.accountApi.get();
      return { usedBytes: storage.usedBytes, limitBytes: storage.limitBytes ?? null };
    });
  }
}
