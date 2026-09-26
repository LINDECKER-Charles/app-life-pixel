import { DOCUMENT, inject, Injectable } from '@angular/core';
import { csvFileName, CsvColumn, toCsv } from './csv';

const CSV_TYPE = 'text/csv;charset=utf-8';

/** Hands a file to the browser as a download: a table's CSV, or a user's export. */
@Injectable({ providedIn: 'root' })
export class FileSaver {
  private readonly document = inject(DOCUMENT);

  save(blob: Blob, fileName: string): void {
    const url = URL.createObjectURL(blob);
    const link = this.document.createElement('a');
    link.href = url;
    link.download = fileName;
    link.rel = 'noopener';
    this.document.body.append(link);
    link.click();
    link.remove();
    // The download has started: the URL is no longer needed.
    setTimeout(() => URL.revokeObjectURL(url));
  }

  /** Saves `rows` as `<table>-<environment>-<date>.csv`. */
  saveCsv<T>(
    rows: readonly T[],
    columns: readonly CsvColumn<T>[],
    name: { readonly table: string; readonly environment: string },
  ): void {
    const blob = new Blob([toCsv(rows, columns)], { type: CSV_TYPE });
    this.save(blob, csvFileName(name.table, name.environment, new Date()));
  }
}
