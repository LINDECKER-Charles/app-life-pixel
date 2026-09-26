import { inject, Injectable, signal } from '@angular/core';
import type { ExportFormat } from '../../engine/engine-types';
import { ExportFlow } from '../export-flow';

/** How the last download ended: its files handed to the device, or refused. */
export interface DownloadOutcome {
  readonly format: ExportFormat;
  readonly succeeded: boolean;
}

/**
 * The export dialog's downloads (design-system/docs/components.md, "Export"): the format being
 * downloaded, then whether it succeeded, so that the dialog says so only once the files are out.
 * Provided by the dialog, cleared when it closes.
 */
@Injectable()
export class ExportDownloads {
  private readonly flow = inject(ExportFlow);
  private readonly downloadingSignal = signal<ExportFormat | null>(null);
  private readonly outcomeSignal = signal<DownloadOutcome | null>(null);

  /** The format whose files are being downloaded, or `null`. */
  readonly downloading = this.downloadingSignal.asReadonly();
  /** The last download's result, or `null` before any. */
  readonly outcome = this.outcomeSignal.asReadonly();

  /** Downloads a format through `ExportFlow`, one at a time; a failure keeps the dialog open. */
  async download(format: ExportFormat): Promise<void> {
    if (this.downloadingSignal() !== null) return;
    this.downloadingSignal.set(format);
    this.outcomeSignal.set(null);
    try {
      await this.flow.download(format);
      this.outcomeSignal.set({ format, succeeded: true });
    } catch {
      this.outcomeSignal.set({ format, succeeded: false });
    } finally {
      this.downloadingSignal.set(null);
    }
  }

  reset(): void {
    this.outcomeSignal.set(null);
  }
}
