import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';

/** Why support needs an account, and the ways to one; signing in comes back to `returnUrl`. */
@Component({
  selector: 'lp-support-signed-out',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="lp-panel lp-panel--soft lp-stack">
      <p>{{ 'support.signed_out.message' | transloco }}</p>
      <p class="lp-cluster">
        <a
          class="lp-button lp-button--primary"
          routerLink="/sign-in"
          [queryParams]="{ returnUrl: returnUrl() }"
        >
          {{ 'support.signed_out.sign_in' | transloco }}
        </a>
        <a
          class="lp-button lp-button--secondary"
          routerLink="/sign-up"
          [queryParams]="{ returnUrl: returnUrl() }"
        >
          {{ 'support.signed_out.sign_up' | transloco }}
        </a>
      </p>
    </div>
  `,
})
export class SupportSignedOut {
  readonly returnUrl = input('/support');
}
