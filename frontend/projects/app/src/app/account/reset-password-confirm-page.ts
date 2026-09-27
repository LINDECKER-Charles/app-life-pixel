import {
  ChangeDetectionStrategy,
  Component,
  inject,
  input,
  signal,
  viewChild,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AuthApi, idScope } from 'shared';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { ACCOUNT_LIMITS } from './account-limits';
import { ErrorSummary } from './form/error-summary';
import { fieldError } from './form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from './form/form-errors';
import { PasswordField } from './form/password-field';
import { SessionStore } from './session-store';

type Field = 'password' | 'form';

const FIELD_BY_CODE: FieldMap<Field> = {
  'auth.password_length': 'password',
  'auth.token_invalid': 'form',
  'auth.csrf': 'form',
};

/**
 * Sets a new password from an emailed link's `token`; every session of the account ends
 * (accounts.md, H7), this browser's included when it was signed in to it.
 */
@Component({
  selector: 'lp-reset-password-confirm-page',
  imports: [ErrorSummary, IonContent, PasswordField, RouterLink, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './reset-password-confirm-page.html',
  styleUrl: './form/account-form.scss',
})
export class ResetPasswordConfirmPage {
  private readonly authApi = inject(AuthApi);
  private readonly session = inject(SessionStore);
  private readonly summary = viewChild(ErrorSummary);

  readonly token = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly id = idScope('reset-password-confirm-page');
  protected readonly targets = { password: this.id('password') };
  protected readonly password = signal('');
  protected readonly pending = signal(false);
  protected readonly done = signal(false);
  protected readonly form = new FormErrors<Field>(FIELD_BY_CODE, 'form');

  protected onPassword(value: string): void {
    this.password.set(value);
    this.form.recheck('password', this.check()[1]);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const token = this.token();
    if (this.pending() || !token) {
      return;
    }
    if (!this.form.check([this.check()])) {
      this.summary()?.focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      await this.authApi.confirmPasswordReset(token, this.password());
      this.session.handleUnauthenticated();
      this.done.set(true);
    } catch (error) {
      this.form.set(error);
      this.summary()?.focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private check(): FieldCheck<Field> {
    const { passwordMinChars: min, passwordMaxChars: max } = this.limits;
    const length = [...this.password()].length;
    return [
      'password',
      fieldError(length >= min && length <= max, 'errors.auth.password_length', { min, max }),
    ];
  }
}
