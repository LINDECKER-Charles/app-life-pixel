import { inject, Injectable } from '@angular/core';
import { LIBRARY_STORE } from '../library-store';
import type { Project } from '../library-types';

/** The most pages read to find every project: past it, the rest are left out. */
const MAX_PAGES = 20;
/** The largest page the API serves. */
const LARGEST_PAGE = 100;

/**
 * The library's projects as a whole, read page after page: to name the project a page shows, and
 * to offer every project an animation can move to. `LibraryStore` lists them only by page.
 */
@Injectable({ providedIn: 'root' })
export class ProjectDirectory {
  private readonly store = inject(LIBRARY_STORE);

  /** Every project, from the most recently updated. */
  async all(): Promise<readonly Project[]> {
    return this.collect(() => false);
  }

  /** The project `id`, or `null` when the library holds none such. */
  async find(id: string): Promise<Project | null> {
    const projects = await this.collect((project) => project.id === id);
    return projects.find((project) => project.id === id) ?? null;
  }

  /** Reads pages until one holds a project `found` accepts, or none is left. */
  private async collect(found: (project: Project) => boolean): Promise<readonly Project[]> {
    const projects: Project[] = [];
    let cursor: string | undefined;
    for (let page = 0; page < MAX_PAGES; page += 1) {
      const { items, nextCursor } = await this.store.listProjects({ cursor, limit: LARGEST_PAGE });
      projects.push(...items);
      if (nextCursor === null || items.some(found)) break;
      cursor = nextCursor;
    }
    return projects;
  }
}
