import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  input,
  signal,
  viewChild,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { AuthApi } from 'shared';
import { ACCOUNT_LIMITS } from './account-limits';
import { FieldMap, FormErrors } from './form-errors';
import { PasswordField } from './password-field';
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
  imports: [PasswordField, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './reset-password-confirm-page.html',
  styleUrl: './account-form.scss',
})
export class ResetPasswordConfirmPage {
  private readonly authApi = inject(AuthApi);
  private readonly session = inject(SessionStore);
  private readonly transloco = inject(TranslocoService);

  private readonly passwordField = viewChild.required<PasswordField>('passwordField');
  private readonly formBanner = viewChild<ElementRef<HTMLElement>>('formBanner');

  readonly token = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly password = signal('');
  protected readonly pending = signal(false);
  protected readonly done = signal(false);
  protected readonly form = new FormErrors<Field>(FIELD_BY_CODE, 'form');

  protected errorText(field: Field): string | undefined {
    const error = this.form.of(field);
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : undefined;
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const token = this.token();
    if (this.pending() || !token) {
      return;
    }
    this.form.clear();
    this.pending.set(true);
    try {
      await this.authApi.confirmPasswordReset(token, this.password());
      this.session.handleUnauthenticated();
      this.done.set(true);
    } catch (error) {
      const field = this.form.set(error);
      if (field === 'password') {
        await this.passwordField().focus();
      } else {
        this.formBanner()?.nativeElement.focus();
      }
    } finally {
      this.pending.set(false);
    }
  }
}
