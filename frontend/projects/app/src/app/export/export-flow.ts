import { computed, inject, Injectable, signal } from '@angular/core';
import { EDITOR_ENGINE } from '../engine/editor-engine';
import { EngineStore } from '../engine/engine-store';
import { isEngineError, type ExportFormat } from '../engine/engine-types';
import { EXPORT_FORMATS, isRasterFormat } from './export-formats';
import { EXPORT_OBSERVER } from './export-observer';
import { failedRow, loadingRow, type ExportRow } from './export-row';
import { EXPORT_SAVER } from './export-saver';
import { gzipTotal } from './export-sizes';

/** No tag selected: the whole animation, every frame. */
const ALL_FRAMES = null;
const DEFAULT_SCALE = 1;

/**
 * The export dialog's state: exports every format when it opens, and again whenever the tag or
 * the scale changes; downloads through `ExportSaver`, then reports to `EXPORT_OBSERVER`
 * (editor.md, U5).
 */
@Injectable({ providedIn: 'root' })
export class ExportFlow {
  private readonly engine = inject(EDITOR_ENGINE);
  private readonly engineStore = inject(EngineStore);
  private readonly saver = inject(EXPORT_SAVER);
  private readonly observer = inject(EXPORT_OBSERVER);

  private readonly openSignal = signal(false);
  private readonly rowsSignal = signal<ReadonlyMap<ExportFormat, ExportRow>>(new Map());
  private readonly tagSignal = signal<string | null>(ALL_FRAMES);
  private readonly scaleSignal = signal(DEFAULT_SCALE);
  private requestId = 0;

  readonly isOpen = this.openSignal.asReadonly();
  readonly tag = this.tagSignal.asReadonly();
  readonly scale = this.scaleSignal.asReadonly();
  readonly limits = this.engineStore.limits;
  readonly document = this.engineStore.document;

  readonly rows = computed<readonly ExportRow[]>(() => {
    const rows = this.rowsSignal();
    return EXPORT_FORMATS.map((format) => rows.get(format) ?? loadingRow(format));
  });

  /** The format with the smallest gzip size among the ones that finished; `null` until one has. */
  readonly lightestFormat = computed<ExportFormat | null>(() => {
    const ready = this.rows().filter((row) => row.status === 'ready');
    if (ready.length === 0) return null;
    return ready.reduce((lightest, row) => (row.gzipBytes < lightest.gzipBytes ? row : lightest))
      .format;
  });

  /** Opens the dialog at the default options, and exports every format. */
  async open(): Promise<void> {
    this.tagSignal.set(ALL_FRAMES);
    this.scaleSignal.set(DEFAULT_SCALE);
    this.openSignal.set(true);
    await this.exportAll();
  }

  close(): void {
    this.openSignal.set(false);
  }

  async setTag(tag: string | null): Promise<void> {
    this.tagSignal.set(tag);
    await this.exportAll();
  }

  async setScale(scale: number): Promise<void> {
    this.scaleSignal.set(scale);
    await this.exportAll();
  }

  /** Downloads a ready row's files, then tells the observer the format and the raw size. */
  async download(format: ExportFormat): Promise<void> {
    const row = this.rowsSignal().get(format);
    if (!row || row.status !== 'ready') return;
    await this.saver.save(row.files);
    this.observer.record(format, row.rawBytes);
  }

  private async exportAll(): Promise<void> {
    const id = ++this.requestId;
    this.rowsSignal.set(new Map(EXPORT_FORMATS.map((format) => [format, loadingRow(format)])));
    await Promise.all(EXPORT_FORMATS.map((format) => this.exportOne(format, id)));
  }

  private async exportOne(format: ExportFormat, id: number): Promise<void> {
    try {
      const result = await this.engine.export({
        format,
        tag: this.tagSignal() ?? undefined,
        scale: isRasterFormat(format) ? this.scaleSignal() : undefined,
      });
      const gzipBytes = await gzipTotal(result.files);
      if (id !== this.requestId) return;
      this.setRow(format, {
        format,
        status: 'ready',
        files: result.files,
        rawBytes: result.totalBytes,
        gzipBytes,
      });
    } catch (error) {
      if (!isEngineError(error)) throw error;
      if (id !== this.requestId) return;
      this.setRow(format, failedRow(format));
    }
  }

  private setRow(format: ExportFormat, row: ExportRow): void {
    this.rowsSignal.update((rows) => {
      const next = new Map(rows);
      next.set(format, row);
      return next;
    });
  }
}
