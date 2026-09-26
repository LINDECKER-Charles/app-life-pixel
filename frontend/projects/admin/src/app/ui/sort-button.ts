import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';

/**
 * The button of a sortable column's header: its label, and the direction it is sorted in. The
 * header itself carries `aria-sort`.
 */
@Component({
  selector: 'lp-sort-button',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" class="sort" (click)="toggled.emit()">
      <ng-content />
      <span class="direction" aria-hidden="true">
        @switch (direction()) {
          @case ('ascending') {
            ▲
          }
          @case ('descending') {
            ▼
          }
          @default {
            ↕
          }
        }
      </span>
      <span class="lp-visually-hidden">{{ 'admin.table.sort' | transloco }}</span>
    </button>
  `,
  styles: `
    .sort {
      display: inline-flex;
      gap: var(--lp-space-1);
      align-items: center;
      padding: 0;
      font: inherit;
      font-weight: var(--lp-font-weight-bold);
      color: inherit;
      background: none;
      border: 0;
      cursor: pointer;
    }
    .direction {
      color: var(--lp-color-text-muted);
      font-size: 0.75em;
    }
  `,
})
export class SortButton {
  readonly direction = input<'ascending' | 'descending' | null>(null);
  readonly toggled = output<void>();
}
