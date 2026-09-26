import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { OpenAnimationFlow } from './open-animation-flow';

/** Says, in the editor's header, that a saved animation is being opened, or why it could not. */
@Component({
  selector: 'lp-open-status',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <span role="status">
      @if (flow.opening()) {
        {{ 'library.open.opening' | transloco }}
      }
    </span>
    @if (flow.failure(); as failure) {
      <span class="failure" role="alert">
        {{ 'errors.' + failure.code | transloco: failure.params }}
        <button type="button" (click)="flow.dismissFailure()">
          {{ 'common.close' | transloco }}
        </button>
      </span>
    }
  `,
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--lp-space-2);
      font-size: var(--lp-font-size-small);
    }
    .failure {
      display: inline-flex;
      align-items: center;
      gap: var(--lp-space-2);
      color: var(--lp-color-danger);
    }
  `,
})
export class OpenStatus {
  protected readonly flow = inject(OpenAnimationFlow);
}
