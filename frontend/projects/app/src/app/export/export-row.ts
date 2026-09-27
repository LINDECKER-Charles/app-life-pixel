import type { ExportedFile, ExportFormat } from '../engine/engine-types';

export type ExportRowStatus = 'loading' | 'ready' | 'failed';

/** One export format's row in the dialog: its files and sizes once the export answers. */
export interface ExportRow {
  readonly format: ExportFormat;
  readonly status: ExportRowStatus;
  readonly files: readonly ExportedFile[];
  readonly rawBytes: number;
  readonly gzipBytes: number;
}

export function loadingRow(format: ExportFormat): ExportRow {
  return { format, status: 'loading', files: [], rawBytes: 0, gzipBytes: 0 };
}

export function failedRow(format: ExportFormat): ExportRow {
  return { format, status: 'failed', files: [], rawBytes: 0, gzipBytes: 0 };
}
