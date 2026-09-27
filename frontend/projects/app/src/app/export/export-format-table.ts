import { ChangeDetectionStrategy, Component, computed, inject, input, output } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { idScope } from 'shared';
import type { ExportFormat } from '../engine/engine-types';
import { Icon } from '../ui/icon/icon';
import type { ExportRow } from './export-row';
import { formatBytes } from './export-sizes';

/**
 * Every export format side by side: what it is for, its raw and gzip sizes as they arrive —
 * preparing, ready or failed, each row on its own —, and its download button. The smallest is
 * named in words, once every format has answered, so that the claim holds.
 */
@Component({
  selector: 'lp-export-format-table',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './export-format-table.html',
  styleUrl: './export-format-table.scss',
})
export class ExportFormatTable {
  private readonly transloco = inject(TranslocoService);
  private readonly locale = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  readonly rows = input.required<readonly ExportRow[]>();
  readonly lightest = input<ExportFormat | null>(null);
  /** The format being downloaded, whose button shows it; the others wait. */
  readonly downloading = input<ExportFormat | null>(null);
  readonly download = output<ExportFormat>();

  protected readonly id = idScope('export-format-table');

  protected readonly isPreparing = computed(() =>
    this.rows().some((row) => row.status === 'loading'),
  );

  /** The lightest format, once no size is still to come. */
  protected readonly smallest = computed(() => (this.isPreparing() ? null : this.lightest()));

  protected isDownloading(row: ExportRow): boolean {
    return this.downloading() === row.format;
  }

  protected rawSize(row: ExportRow): string {
    return formatBytes(row.rawBytes, this.locale());
  }

  protected gzipSize(row: ExportRow): string {
    return formatBytes(row.gzipBytes, this.locale());
  }
}
