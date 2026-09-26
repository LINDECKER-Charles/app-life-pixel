/** A column of a CSV export: its header, and the text of each row's cell. */
export interface CsvColumn<T> {
  readonly header: string;
  readonly value: (row: T) => string | number | boolean | null | undefined;
}

// A cell starting with one of these is a formula to a spreadsheet: it is prefixed with a quote.
const FORMULA_START = /^[=+\-@\t\r]/;
const NEEDS_QUOTES = /[",\r\n]/;

/** One cell, as RFC 4180 writes it, and neutralised against formula injection. */
export function csvCell(value: string | number | boolean | null | undefined): string {
  if (value === null || value === undefined) {
    return '';
  }
  let text = String(value);
  if (typeof value === 'string' && FORMULA_START.test(text)) {
    text = `'${text}`;
  }
  return NEEDS_QUOTES.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

/** The rows as CSV: a header line, then one line per row, each ended by CRLF. */
export function toCsv<T>(rows: readonly T[], columns: readonly CsvColumn<T>[]): string {
  const lines = [
    columns.map((column) => csvCell(column.header)),
    ...rows.map((row) => columns.map((column) => csvCell(column.value(row)))),
  ];
  return lines.map((cells) => `${cells.join(',')}\r\n`).join('');
}

/** `users-production-2026-09-26.csv`: what a table is, where from, and when. */
export function csvFileName(table: string, environment: string, now: Date): string {
  return `${table}-${environment}-${now.toISOString().slice(0, 10)}.csv`;
}
