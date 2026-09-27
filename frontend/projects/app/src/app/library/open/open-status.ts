import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Icon } from '../../ui/icon/icon';
import { OpenAnimationFlow } from './open-animation-flow';

/**
 * Says, in the editor's document bar, that a saved animation is being opened, or why it could not
 * be, with a way to dismiss the message.
 */
@Component({
  selector: 'lp-open-status',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <span role="status">
      @if (flow.opening()) {
        <span class="lp-pill lp-pill--info">
          <span class="lp-pill__dot"></span>
          {{ 'library.open.opening' | transloco }}
        </span>
      }
    </span>
    @if (flow.failure(); as failure) {
      <span class="failure" role="alert">
        <lp-icon name="error" size="small" />
        {{ 'errors.' + failure.code | transloco: failure.params }}
        <button
          type="button"
          class="lp-button lp-button--quiet lp-button--compact"
          (click)="flow.dismissFailure()"
        >
          {{ 'common.close' | transloco }}
        </button>
      </span>
    }
  `,
  styles: `
    :host {
      display: inline-flex;
      flex-wrap: wrap;
      align-items: center;
      gap: var(--lp-space-2);
      min-width: 0;
      font-size: var(--lp-font-size-small);
    }
    .failure {
      display: inline-flex;
      align-items: center;
      gap: var(--lp-space-2);
      padding-inline-start: var(--lp-space-2);
      font-weight: var(--lp-font-weight-semibold);
      color: var(--lp-color-danger);
      background: var(--lp-color-danger-bg);
      border-radius: var(--lp-radius-medium);
    }
  `,
})
export class OpenStatus {
  protected readonly flow = inject(OpenAnimationFlow);
}
