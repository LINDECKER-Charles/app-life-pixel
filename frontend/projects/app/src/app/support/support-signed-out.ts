import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';

/** Why support needs an account, and the ways to one; signing in comes back to `returnUrl`. */
@Component({
  selector: 'lp-support-signed-out',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <p>{{ 'support.signed_out.message' | transloco }}</p>
    <p class="links">
      <a class="button" routerLink="/sign-in" [queryParams]="{ returnUrl: returnUrl() }">
        {{ 'support.signed_out.sign_in' | transloco }}
      </a>
      <a class="button" routerLink="/sign-up">{{ 'support.signed_out.sign_up' | transloco }}</a>
    </p>
  `,
  styles: `
    .links {
      display: flex;
      gap: var(--lp-space-3);
    }
  `,
})
export class SupportSignedOut {
  readonly returnUrl = input('/support');
}
