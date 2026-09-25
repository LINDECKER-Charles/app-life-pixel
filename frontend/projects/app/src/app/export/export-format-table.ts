import { ChangeDetectionStrategy, Component, inject, input, output } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ExportFormat } from '../engine/engine-types';
import type { ExportRow } from './export-row';
import { formatBytes } from './export-sizes';

/** Every export format side by side, with its raw and gzip sizes, the lightest one marked. */
@Component({
  selector: 'lp-export-format-table',
  imports: [TranslocoPipe],
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
  readonly download = output<ExportFormat>();

  protected rawSize(row: ExportRow): string | null {
    return row.status === 'ready' ? formatBytes(row.rawBytes, this.locale()) : null;
  }

  protected gzipSize(row: ExportRow): string | null {
    return row.status === 'ready' ? formatBytes(row.gzipBytes, this.locale()) : null;
  }
}
