import { computed, signal } from '@angular/core';
import { toLibraryFailure, type LibraryFailure, type Page, type PageQuery } from '../library-types';

/** How many items a list reads at a time; "Load more" reads the next as many. */
export const PAGE_SIZE = 50;

/** An item a list can find again by its id. */
interface Identified {
  readonly id: string;
}

/**
 * A list read page by page (accounts.md, H8): the items so far, whether more follow, and the last
 * failure, as signals. `read` fetches one page; `reload` starts over, `loadMore` reads the next.
 * The item functions keep the list in step with a change made through the store.
 */
export class PagedList<T extends Identified> {
  private readonly itemsSignal = signal<readonly T[]>([]);
  private readonly cursorSignal = signal<string | null>(null);
  private readonly loadingSignal = signal(false);
  private readonly loadedSignal = signal(false);
  private readonly failureSignal = signal<LibraryFailure | null>(null);
  private generation = 0;

  readonly items = this.itemsSignal.asReadonly();
  readonly loading = this.loadingSignal.asReadonly();
  /** Whether the first page has arrived, so that an empty list means an empty library. */
  readonly loaded = this.loadedSignal.asReadonly();
  readonly failure = this.failureSignal.asReadonly();
  readonly hasMore = computed(() => this.cursorSignal() !== null);

  constructor(private readonly read: (page: PageQuery) => Promise<Page<T>>) {}

  /** Reads the first page again, dropping what was read. */
  reload(): Promise<void> {
    this.generation += 1;
    this.itemsSignal.set([]);
    this.cursorSignal.set(null);
    this.loadedSignal.set(false);
    return this.fetch({ limit: PAGE_SIZE });
  }

  /** Reads the page after the last one read. */
  loadMore(): Promise<void> {
    const cursor = this.cursorSignal();
    if (cursor === null || this.loadingSignal()) return Promise.resolve();
    return this.fetch({ cursor, limit: PAGE_SIZE });
  }

  /** Puts `item` first, as the most recently updated. */
  prepend(item: T): void {
    this.itemsSignal.update((items) => [item, ...items.filter(({ id }) => id !== item.id)]);
  }

  /** Replaces the item with `item`'s id, where it stands. */
  replace(item: T): void {
    this.itemsSignal.update((items) => items.map((each) => (each.id === item.id ? item : each)));
  }

  remove(id: string): void {
    this.itemsSignal.update((items) => items.filter((item) => item.id !== id));
  }

  /** Records a failure of an action on the list, to show beside it. */
  fail(error: unknown): void {
    this.failureSignal.set(toLibraryFailure(error));
  }

  dismissFailure(): void {
    this.failureSignal.set(null);
  }

  private async fetch(query: PageQuery): Promise<void> {
    const generation = this.generation;
    this.loadingSignal.set(true);
    this.failureSignal.set(null);
    try {
      const page = await this.read(query);
      if (generation !== this.generation) return;
      this.itemsSignal.update((items) => [...items, ...page.items]);
      this.cursorSignal.set(page.nextCursor);
      this.loadedSignal.set(true);
    } catch (error: unknown) {
      if (generation === this.generation) this.fail(error);
    } finally {
      if (generation === this.generation) this.loadingSignal.set(false);
    }
  }
}
