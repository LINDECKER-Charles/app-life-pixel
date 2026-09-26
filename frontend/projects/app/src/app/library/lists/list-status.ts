import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { StatusBanner } from '../../ui/status-banner/status-banner';
import type { PagedList } from './paged-list';

/**
 * Where a library list stands, above its items (design-system/docs/patterns.md, "Cross-surface
 * state matrix"): reading its first page, failing to — an error with Retry, never an empty
 * list —, or an action on it that failed, with Dismiss. Nothing once the list reads well.
 */
@Component({
  selector: 'lp-list-status',
  imports: [StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (list().failure(); as failure) {
      <lp-status-banner variant="danger">
        <p class="message">{{ 'errors.' + failure.code | transloco: failure.params }}</p>
        <div lpBannerActions>
          @if (list().loaded()) {
            <button
              type="button"
              class="lp-button lp-button--secondary lp-button--compact"
              (click)="list().dismissFailure()"
            >
              {{ 'common.dismiss' | transloco }}
            </button>
          } @else {
            <button
              type="button"
              class="lp-button lp-button--secondary lp-button--compact"
              [disabled]="list().loading()"
              (click)="list().reload()"
            >
              {{ 'common.retry' | transloco }}
            </button>
          }
        </div>
      </lp-status-banner>
    } @else if (!list().loaded()) {
      <p class="message loading" role="status">{{ 'common.loading' | transloco }}</p>
    }
  `,
  styles: `
    .message {
      margin: 0;
    }
    .loading {
      color: var(--lp-color-text-muted);
    }
  `,
})
export class ListStatus {
  // Any item type: the component reads the list's state, never its items.
  readonly list = input.required<PagedList<{ readonly id: string }>>();
}
