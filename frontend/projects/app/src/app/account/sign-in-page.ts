import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Icon } from '../ui/icon/icon';
import { ACCOUNT_LIMITS } from './account-limits';
import { ErrorSummary } from './form/error-summary';
import { fieldError } from './form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from './form/form-errors';
import { PasswordField } from './form/password-field';
import { SessionStore } from './session-store';

type Field = 'email' | 'password' | 'form';

const FIELD_BY_CODE: FieldMap<Field> = {
  'auth.invalid_credentials': 'password',
  'auth.account_suspended': 'form',
  'auth.csrf': 'form',
  'rate_limit.exceeded': 'form',
};

/**
 * Signs in with an address and a password (D29); a `returnUrl` brings the person back to the work
 * they left (D37), whether they were already signed in or just signed in now.
 */
@Component({
  selector: 'lp-sign-in-page',
  imports: [ErrorSummary, Icon, IonContent, PasswordField, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sign-in-page.html',
  styleUrl: './form/account-form.scss',
})
export class SignInPage {
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly summary = viewChild.required(ErrorSummary);

  readonly returnUrl = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly targets = { email: 'sign-in-email', password: 'sign-in-password' };
  protected readonly email = signal('');
  protected readonly password = signal('');
  protected readonly pending = signal(false);
  protected readonly form = new FormErrors<Field>(FIELD_BY_CODE, 'form');

  constructor() {
    effect(() => {
      if (!this.session.account()) return;
      untracked(() => void this.router.navigateByUrl(this.returnUrl() ?? '/account'));
    });
  }

  protected onEmail(value: string): void {
    this.email.set(value);
    this.form.recheck('email', this.checks()[0][1]);
  }

  protected onPassword(value: string): void {
    this.password.set(value);
    this.form.recheck('password', this.checks()[1][1]);
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending()) {
      return;
    }
    if (!this.form.check(this.checks())) {
      this.summary().focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      await this.session.signIn(this.email(), this.password());
    } catch (error) {
      this.form.set(error);
      this.summary().focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private checks(): FieldCheck<Field>[] {
    return [
      ['email', fieldError(this.email().trim() !== '', 'auth.form.email_required')],
      ['password', fieldError(this.password() !== '', 'auth.form.password_required')],
    ];
  }
}
