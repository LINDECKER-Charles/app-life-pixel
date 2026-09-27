import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { ApiProblem, AuthApi } from 'shared';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { SessionStore } from './session-store';

type Status = 'pending' | 'checking' | 'success' | 'failure';

/**
 * Reads `token` from the emailed link, spends it and says the result (accounts.md, H7). Updates
 * the signed-in account's `emailVerified` in place, without asking the session again.
 */
@Component({
  selector: 'lp-verify-email-page',
  imports: [IonContent, RouterLink, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="lp-page lp-page--reading account-form-page">
        <header class="lp-page-header">
          <div class="lp-page-header__text">
            <h1 class="lp-page-header__title">{{ 'auth.verify_email.title' | transloco }}</h1>
          </div>
        </header>
        @switch (status()) {
          @case ('pending') {
            <p>{{ 'auth.verify_email.pending' | transloco }}</p>
          }
          @case ('checking') {
            <p role="status">{{ 'common.loading' | transloco }}</p>
          }
          @case ('success') {
            <lp-status-banner variant="success">
              <p>{{ 'auth.verify_email.success' | transloco }}</p>
            </lp-status-banner>
            <p>
              <a class="lp-button lp-button--primary" routerLink="/account">
                {{ 'auth.verify_email.account_link' | transloco }}
              </a>
            </p>
          }
          @case ('failure') {
            <lp-status-banner variant="danger">
              <p>{{ errorMessage() }}</p>
            </lp-status-banner>
            <p>
              <a class="lp-button lp-button--secondary" routerLink="/sign-in">
                {{ 'auth.verify_email.sign_in_link' | transloco }}
              </a>
            </p>
          }
        }
      </div>
    </ion-content>
  `,
  styleUrl: './form/account-form.scss',
})
export class VerifyEmailPage {
  private readonly authApi = inject(AuthApi);
  private readonly session = inject(SessionStore);
  private readonly transloco = inject(TranslocoService);

  readonly token = input<string>();

  protected readonly status = signal<Status>('pending');
  private readonly error = signal<ApiProblem | undefined>(undefined);

  constructor() {
    effect(() => {
      const token = this.token();
      untracked(() => void this.verify(token));
    });
  }

  protected errorMessage(): string {
    const error = this.error();
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : '';
  }

  private async verify(token: string | undefined): Promise<void> {
    if (!token) {
      this.status.set('pending');
      return;
    }
    this.status.set('checking');
    try {
      await this.authApi.verifyEmail(token);
      this.status.set('success');
      const account = this.session.account();
      if (account) {
        this.session.setAccount({ ...account, emailVerified: true });
      }
    } catch (error) {
      if (!(error instanceof ApiProblem)) {
        throw error;
      }
      this.error.set(error);
      this.status.set('failure');
    }
  }
}
