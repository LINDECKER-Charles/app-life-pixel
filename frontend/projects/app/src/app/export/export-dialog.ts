import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { ExportDownloadDone } from './dialog/export-download-done';
import { ExportDownloads } from './dialog/export-downloads';
import { ExportFlow } from './export-flow';
import { ExportFormatTable } from './export-format-table';
import { ExportOptionsForm } from './export-options-form';
import { ExportSnippet } from './export-snippet';

/**
 * The export dialog: every format side by side with its size, the options, the downloads and the
 * integration snippets (editor.md, U5). Opened by `ExportButton`, also on Ctrl/⌘ E. It follows
 * the dialog anatomy of `_dialog.scss`: the title and its close button, the animation exported,
 * the scrolling content, a failed download, then the way back to editing — with Pip once a
 * download succeeded.
 */
@Component({
  selector: 'lp-export-dialog',
  imports: [
    ExportDownloadDone,
    ExportFormatTable,
    ExportOptionsForm,
    ExportSnippet,
    Icon,
    IonModal,
    StatusBanner,
    TranslocoPipe,
  ],
  providers: [ExportDownloads],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      class="lp-modal lp-modal--wide"
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'export.dialog.title' | transloco"
      (didDismiss)="onDismissed()"
    >
      <ng-template>
        <div class="lp-dialog">
          <div class="lp-dialog__header">
            <h2 class="lp-dialog__title">{{ 'export.dialog.title' | transloco }}</h2>
            <button
              type="button"
              class="lp-icon-button"
              [attr.aria-label]="'common.close' | transloco"
              (click)="flow.close()"
            >
              <lp-icon name="close" />
            </button>
          </div>
          @if (context(); as context) {
            <p class="lp-dialog__context">{{ 'export.dialog.context' | transloco: context }}</p>
          }
          <div class="lp-dialog__body">
            @if (limits(); as limits) {
              <lp-export-options-form [limits]="limits" />
            }
            <lp-status-banner variant="info">
              <p>{{ 'export.download_note' | transloco }}</p>
            </lp-status-banner>
            <lp-export-format-table
              [rows]="flow.rows()"
              [lightest]="flow.lightestFormat()"
              [downloading]="downloads.downloading()"
              (download)="downloads.download($event)"
            />
            <lp-export-snippet />
          </div>
          <div class="lp-dialog__failure">
            @if (failedFormat(); as format) {
              <lp-status-banner variant="danger">
                <p>
                  {{
                    'export.download.failed'
                      | transloco: { format: ('export.format.' + format | transloco) }
                  }}
                </p>
              </lp-status-banner>
            }
          </div>
          <div class="lp-dialog__actions">
            <div class="done" role="status">
              @if (doneFormat(); as format) {
                <lp-export-download-done [format]="format" />
              }
            </div>
            <button type="button" class="lp-button lp-button--primary" (click)="flow.close()">
              {{ 'export.dialog.back' | transloco }}
            </button>
          </div>
        </div>
      </ng-template>
    </ion-modal>
  `,
  styles: `
    .done {
      flex: 1 1 14rem;
      align-self: center;
      min-width: 0;
    }
  `,
})
export class ExportDialog {
  protected readonly flow = inject(ExportFlow);
  protected readonly downloads = inject(ExportDownloads);
  protected readonly limits = this.flow.limits;

  /** The animation exported, as the context line says it: title, size and frame count. */
  protected readonly context = computed(() => {
    const document = this.flow.document();
    if (!document) return null;
    const { title, width, height, frames } = document;
    return { title, width, height, frames: frames.length };
  });

  protected readonly doneFormat = computed(() => {
    const outcome = this.downloads.outcome();
    return outcome?.succeeded ? outcome.format : null;
  });

  protected readonly failedFormat = computed(() => {
    const outcome = this.downloads.outcome();
    return outcome && !outcome.succeeded ? outcome.format : null;
  });

  protected onDismissed(): void {
    this.flow.close();
    this.downloads.reset();
  }
}
