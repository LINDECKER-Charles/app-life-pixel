import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { ExportFormat } from '../../engine/engine-types';

const PIP_SOURCE = '/design-system/illustrations/pip-export.svg';

/**
 * The quiet success of a download, beside the dialog's actions: Pip, decorative, and the format
 * whose files are out (design-system/docs/journeys.md §6). Nothing moves, so reduced motion has
 * nothing to remove; the parent's live region announces it.
 */
@Component({
  selector: 'lp-export-download-done',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <img class="pip" [src]="pipSource" alt="" width="48" height="48" />
    <p class="text">
      {{
        'export.download.done' | transloco: { format: ('export.format.' + format() | transloco) }
      }}
    </p>
  `,
  styles: `
    :host {
      display: flex;
      gap: var(--lp-space-3);
      align-items: center;
      min-width: 0;
    }
    .pip {
      flex-shrink: 0;
      image-rendering: pixelated;
    }
    .text {
      margin: 0;
      font-size: var(--lp-font-size-small);
      font-weight: var(--lp-font-weight-semibold);
    }
  `,
})
export class ExportDownloadDone {
  readonly format = input.required<ExportFormat>();

  protected readonly pipSource = PIP_SOURCE;
}
