import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { ExportFlow } from './export-flow';
import { ExportFormatTable } from './export-format-table';
import { ExportOptionsForm } from './export-options-form';
import { ExportSnippet } from './export-snippet';

/**
 * The export dialog: every format side by side with its size, the options, the downloads and the
 * integration snippets (editor.md, U5). Opened by `ExportButton`, also on Ctrl/⌘ E.
 */
@Component({
  selector: 'lp-export-dialog',
  imports: [ExportFormatTable, ExportOptionsForm, ExportSnippet, IonModal, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'export.dialog.title' | transloco"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="dialog">
          <h2 class="heading">{{ 'export.dialog.title' | transloco }}</h2>
          @if (limits(); as limits) {
            <lp-export-options-form [limits]="limits" />
          }
          <lp-export-format-table
            [rows]="flow.rows()"
            [lightest]="flow.lightestFormat()"
            (download)="flow.download($event)"
          />
          <lp-export-snippet />
        </div>
      </ng-template>
    </ion-modal>
  `,
  styles: `
    .dialog {
      display: flex;
      flex-direction: column;
      gap: var(--lp-space-5);
      padding: var(--lp-space-5);
      overflow-y: auto;
      background: var(--lp-color-background);
    }
    .heading {
      margin: 0;
      font-size: var(--lp-font-size-large);
    }
  `,
})
export class ExportDialog {
  protected readonly flow = inject(ExportFlow);
  protected readonly limits = this.flow.limits;
}
