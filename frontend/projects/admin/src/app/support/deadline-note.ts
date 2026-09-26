import { ChangeDetectionStrategy, Component, computed, inject, input } from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { SupportRequest } from '../core/admin-types';
import { formatDate } from '../core/format';
import { legalDeadline } from './legal-deadline';

/**
 * The legal deadline of a data-protection request (admin-console.md), one month after its
 * creation: the days left, or how late it is. Nothing for another category.
 */
@Component({
  selector: 'lp-deadline-note',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (deadline(); as deadline) {
      <span class="deadline" [class.overdue]="deadline.overdue">
        @if (deadline.settled) {
          {{ 'admin.support.deadline_settled' | transloco: { date: date() } }}
        } @else if (deadline.overdue) {
          {{ 'admin.support.deadline_overdue' | transloco: { date: date() } }}
        } @else {
          {{ 'admin.support.deadline' | transloco: { date: date(), days: deadline.daysLeft } }}
        }
      </span>
    }
  `,
  styles: `
    .deadline {
      font-weight: var(--lp-font-weight-bold);
    }
    .overdue {
      color: var(--lp-color-danger);
    }
  `,
})
export class DeadlineNote {
  private readonly transloco = inject(TranslocoService);

  readonly request = input.required<Pick<SupportRequest, 'category' | 'createdAt' | 'status'>>();
  /** The present, for tests; the clock by default. */
  readonly now = input<Date | null>(null);

  protected readonly deadline = computed(() =>
    legalDeadline(this.request(), this.now() ?? new Date()),
  );
  protected readonly date = computed(() => {
    const deadline = this.deadline();
    return deadline
      ? formatDate(deadline.deadline.toISOString(), this.transloco.getActiveLang(), 'UTC')
      : '';
  });
}
