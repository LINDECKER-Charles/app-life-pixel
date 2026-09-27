import { computed, Signal, signal, WritableSignal } from '@angular/core';
import { ariaSort, nextSort, sortRows, SortState, SortValue } from './sort';

/**
 * The sort of one table: which column, which way, and its rows in that order. A click on a
 * header flips the column it names, or sorts by it.
 */
export class TableSort<T, K extends string> {
  private readonly state: WritableSignal<SortState<K>>;

  readonly rows: Signal<T[]>;

  constructor(
    rows: Signal<readonly T[]>,
    initial: SortState<K>,
    valueOf: (row: T, key: K) => SortValue,
  ) {
    this.state = signal(initial);
    this.rows = computed(() => sortRows(rows(), this.state(), valueOf));
  }

  toggle(key: K): void {
    this.state.update((sort) => nextSort(sort, key));
  }

  ariaSort(key: K): 'ascending' | 'descending' | null {
    return ariaSort(this.state(), key);
  }
}
