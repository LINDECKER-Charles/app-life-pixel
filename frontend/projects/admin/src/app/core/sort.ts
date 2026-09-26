/** Which column a table is sorted by, and which way. */
export interface SortState<K extends string> {
  readonly key: K;
  readonly direction: 'ascending' | 'descending';
}

/** A cell's value as the sort compares it: `null` always sorts last. */
export type SortValue = string | number | boolean | null | undefined;

function compareValues(a: SortValue, b: SortValue): number {
  if (a === b) {
    return 0;
  }
  if (a === null || a === undefined) {
    return 1;
  }
  if (b === null || b === undefined) {
    return -1;
  }
  if (typeof a === 'string' && typeof b === 'string') {
    return a.localeCompare(b);
  }
  return a < b ? -1 : 1;
}

/** A sorted copy of `rows`: stable, with empty values last whichever the direction. */
export function sortRows<T, K extends string>(
  rows: readonly T[],
  sort: SortState<K>,
  valueOf: (row: T, key: K) => SortValue,
): T[] {
  const sign = sort.direction === 'ascending' ? 1 : -1;
  return rows
    .map((row, index) => ({ row, index, value: valueOf(row, sort.key) }))
    .sort((a, b) => {
      const empty = (a.value ?? null) === null || (b.value ?? null) === null;
      const order = compareValues(a.value, b.value);
      return (empty ? order : sign * order) || a.index - b.index;
    })
    .map(({ row }) => row);
}

/** The sort after a click on the header of `key`: the same column flips, another starts. */
export function nextSort<K extends string>(sort: SortState<K>, key: K): SortState<K> {
  if (sort.key !== key) {
    return { key, direction: 'ascending' };
  }
  return { key, direction: sort.direction === 'ascending' ? 'descending' : 'ascending' };
}

/** The `aria-sort` of a header: only the sorted column has one. */
export function ariaSort<K extends string>(
  sort: SortState<K>,
  key: K,
): 'ascending' | 'descending' | null {
  return sort.key === key ? sort.direction : null;
}
