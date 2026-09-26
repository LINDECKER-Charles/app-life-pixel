import { ChangeDetectionStrategy, Component, inject, signal, viewChild } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AuthApi } from 'shared';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { ACCOUNT_LIMITS } from './account-limits';
import { ErrorSummary } from './form/error-summary';
import { fieldError } from './form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from './form/form-errors';

type Field = 'email' | 'form';

const FIELD_BY_CODE: FieldMap<Field> = {};

/**
 * Asks for a link setting a new password: the answer is the same, and as quick, whether an
 * active account has the address or not — no field says otherwise (accounts.md, H7).
 */
@Component({
  selector: 'lp-reset-password-page',
  imports: [ErrorSummary, Icon, IonContent, RouterLink, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './reset-password-page.html',
  styleUrl: './form/account-form.scss',
})
export class ResetPasswordPage {
  private readonly authApi = inject(AuthApi);
  private readonly summary = viewChild(ErrorSummary);

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly targets = { email: 'reset-password-email' };
  protected readonly email = signal('');
  protected readonly pending = signal(false);
  protected readonly sent = signal(false);
  protected readonly form = new FormErrors<Field>(FIELD_BY_CODE, 'form');

  protected onEmail(value: string): void {
    this.email.set(value);
    this.form.recheck('email', this.check()[1]);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending()) {
      return;
    }
    if (!this.form.check([this.check()])) {
      this.summary()?.focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      await this.authApi.requestPasswordReset(this.email());
      this.sent.set(true);
    } catch (error) {
      this.form.set(error);
      this.summary()?.focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private check(): FieldCheck<Field> {
    return ['email', fieldError(this.email().trim() !== '', 'auth.form.email_required')];
  }
}
