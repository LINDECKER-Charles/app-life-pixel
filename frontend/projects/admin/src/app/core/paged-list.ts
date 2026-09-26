import { computed, signal } from '@angular/core';
import type { ApiProblem } from 'shared';
import { asProblem } from './problem';

/** A page of a list, as the admin API answers it. */
export interface Page<T> {
  readonly items: readonly T[];
  readonly nextCursor?: string | null;
}

/** Reads the page of `query` after `cursor`, or the first. */
export type PageReader<T, Q> = (query: Q, cursor?: string) => Promise<Page<T>>;

/**
 * A list read page after page by cursor: `reload` starts over for a new query, `more` appends the
 * next page. An answer to an older query is dropped.
 */
export class PagedList<T, Q> {
  private generation = 0;
  private query: Q | null = null;
  private readonly cursor = signal<string | null>(null);

  readonly items = signal<readonly T[]>([]);
  readonly problem = signal<ApiProblem | null>(null);
  readonly loading = signal(false);
  readonly hasMore = computed(() => this.cursor() !== null);

  constructor(private readonly read: PageReader<T, Q>) {}

  reload(query: Q): Promise<void> {
    this.query = query;
    return this.load(query, undefined);
  }

  more(): Promise<void> {
    const cursor = this.cursor();
    return this.query === null || cursor === null
      ? Promise.resolve()
      : this.load(this.query, cursor);
  }

  private async load(query: Q, cursor: string | undefined): Promise<void> {
    const generation = ++this.generation;
    this.loading.set(true);
    let page: Page<T> | null = null;
    let problem: ApiProblem | null = null;
    try {
      page = await this.read(query, cursor);
    } catch (error: unknown) {
      problem = asProblem(error);
    }
    if (generation !== this.generation) {
      return;
    }
    this.problem.set(problem);
    if (page) {
      const items = page.items;
      this.items.update((current) => (cursor ? [...current, ...items] : items));
      this.cursor.set(page.nextCursor ?? null);
    } else if (!cursor) {
      this.items.set([]);
      this.cursor.set(null);
    }
    this.loading.set(false);
  }
}
