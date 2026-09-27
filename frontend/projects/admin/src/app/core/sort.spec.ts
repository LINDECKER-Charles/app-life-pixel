import { signal } from '@angular/core';
import { ariaSort, nextSort, sortRows } from './sort';
import { TableSort } from './table-sort';

interface Row {
  readonly name: string;
  readonly size: number | null;
}

const ROWS: Row[] = [
  { name: 'b', size: 2 },
  { name: 'a', size: null },
  { name: 'c', size: 1 },
  { name: 'd', size: 2 },
];
const valueOf = (row: Row, key: 'name' | 'size'): string | number | null => row[key];

describe('sorting', () => {
  it('sorts either way, stably, with empty values last', () => {
    const names = (rows: Row[]): string[] => rows.map((row) => row.name);

    expect(names(sortRows(ROWS, { key: 'size', direction: 'ascending' }, valueOf))).toEqual([
      'c',
      'b',
      'd',
      'a',
    ]);
    expect(names(sortRows(ROWS, { key: 'size', direction: 'descending' }, valueOf))).toEqual([
      'b',
      'd',
      'c',
      'a',
    ]);
  });

  it('flips the sorted column, and starts another ascending', () => {
    const sort = { key: 'name', direction: 'ascending' } as const;

    expect(nextSort(sort, 'name')).toEqual({ key: 'name', direction: 'descending' });
    expect(nextSort(sort, 'size')).toEqual({ key: 'size', direction: 'ascending' });
  });

  it('gives only the sorted column an aria-sort', () => {
    const sort = { key: 'name', direction: 'descending' } as const;

    expect(ariaSort(sort, 'name')).toBe('descending');
    expect(ariaSort<'name' | 'size'>(sort, 'size')).toBeNull();
  });

  it('keeps a table’s rows in the order of its sorted column', () => {
    const rows = signal<readonly Row[]>(ROWS);
    const table = new TableSort<Row, 'name' | 'size'>(
      rows,
      { key: 'name', direction: 'ascending' },
      valueOf,
    );
    expect(table.rows().map((row) => row.name)).toEqual(['a', 'b', 'c', 'd']);

    table.toggle('name');
    expect(table.rows()[0].name).toBe('d');
    expect(table.ariaSort('name')).toBe('descending');

    rows.set([{ name: 'e', size: 0 }]);
    expect(table.rows().map((row) => row.name)).toEqual(['e']);
  });
});
