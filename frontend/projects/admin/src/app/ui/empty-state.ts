import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { emptyReason, EmptyReason, problemMessage } from '../core/problem';

const REASON_KEYS: Readonly<Record<Exclude<EmptyReason, 'nothing'>, string>> = {
  not_configured: 'admin.empty.not_configured',
  unavailable: 'admin.empty.unavailable',
  failed: 'admin.empty.failed',
};

/**
 * What an empty panel or table says instead of a blank: that there is nothing in the range, or
 * that its source is not configured, does not answer, or failed — with the admin server's reason.
 */
@Component({
  selector: 'lp-empty-state',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="empty" role="status" [attr.data-reason]="reason()">
      @if (problem(); as problem) {
        <p class="why">{{ reasonKey() | transloco }}</p>
        <p>{{ detail().key | transloco: detail().params }}</p>
      } @else {
        <p class="why">{{ nothing() | transloco: params() }}</p>
      }
    </div>
  `,
  styles: `
    .empty {
      padding: var(--lp-space-4);
      border: 1px dashed var(--lp-color-border);
      border-radius: var(--lp-radius-medium);
      color: var(--lp-color-text-muted);
    }
    .why {
      margin: 0;
      color: var(--lp-color-text);
      font-weight: var(--lp-font-weight-bold);
    }
  `,
})
export class EmptyState {
  /** The key saying why there is nothing, when no source failed: specific to the view. */
  readonly nothing = input.required<string>();
  readonly params = input<Readonly<Record<string, unknown>>>({});
  /** The failure that left the view empty, if any. */
  readonly problem = input<ApiProblem | null>(null);

  protected readonly reason = computed(() => emptyReason(this.problem()));
  protected readonly reasonKey = computed(() => {
    const reason = this.reason();
    return reason === 'nothing' ? this.nothing() : REASON_KEYS[reason];
  });
  protected readonly detail = computed(() => {
    const problem = this.problem();
    return problem ? problemMessage(problem) : { key: this.nothing(), params: {} };
  });
}
