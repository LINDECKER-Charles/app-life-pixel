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
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { AvailableLanguages } from 'shared';
import { Icon } from '../ui/icon/icon';
import { ACCOUNT_LIMITS } from './account-limits';
import { ErrorSummary } from './form/error-summary';
import { fieldError } from './form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from './form/form-errors';
import { PasswordField } from './form/password-field';
import { SessionStore } from './session-store';

type Field = 'email' | 'password' | 'form';

// `account.language` cannot happen from the select, whose options are always the available
// languages: it lands on the summary like the other codes with no field of their own.
const FIELD_BY_CODE: FieldMap<Field> = {
  'auth.email_invalid': 'email',
  'auth.email_taken': 'email',
  'auth.password_length': 'password',
  'auth.csrf': 'form',
  'rate_limit.exceeded': 'form',
};

/**
 * Creates an account (D29), signs in and sends the verification email; links H16's terms and
 * privacy policy. A `returnUrl` brings the person back to the work they left (D37).
 */
@Component({
  selector: 'lp-sign-up-page',
  imports: [ErrorSummary, Icon, IonContent, PasswordField, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sign-up-page.html',
  styleUrl: './form/account-form.scss',
})
export class SignUpPage {
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly summary = viewChild.required(ErrorSummary);

  readonly returnUrl = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly targets = { email: 'sign-up-email', password: 'sign-up-password' };
  protected readonly languages = inject(AvailableLanguages).languages;
  protected readonly email = signal('');
  protected readonly password = signal('');
  protected readonly language = signal(this.transloco.getActiveLang());
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
      await this.session.signUp(this.email(), this.password(), this.language());
    } catch (error) {
      this.form.set(error);
      this.summary().focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  /** The address is required; the password's length is the server's rule, said in the hint. */
  private checks(): FieldCheck<Field>[] {
    const { passwordMinChars: min, passwordMaxChars: max } = this.limits;
    const length = [...this.password()].length;
    return [
      ['email', fieldError(this.email().trim() !== '', 'auth.form.email_required')],
      [
        'password',
        fieldError(length >= min && length <= max, 'errors.auth.password_length', { min, max }),
      ],
    ];
  }
}
