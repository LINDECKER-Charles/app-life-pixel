import {
  ChangeDetectionStrategy,
  Component,
  effect,
  ElementRef,
  inject,
  input,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import {
  IonInput,
  IonSelect,
  IonSelectOption,
  type InputCustomEvent,
  type SelectCustomEvent,
} from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { AvailableLanguages } from 'shared';
import { ACCOUNT_LIMITS } from './account-limits';
import { FieldMap, FormErrors } from './form-errors';
import { PasswordField } from './password-field';
import { SessionStore } from './session-store';

type Field = 'email' | 'password' | 'form';

// `account.language` cannot happen from the select, whose options are always the available
// languages: it lands on the form banner like the other codes with no field of their own.
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
  imports: [IonInput, IonSelect, IonSelectOption, PasswordField, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sign-up-page.html',
  styleUrl: './account-form.scss',
})
export class SignUpPage {
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);

  private readonly emailField = viewChild.required<IonInput>('emailField');
  private readonly passwordField = viewChild.required<PasswordField>('passwordField');
  private readonly formBanner = viewChild<ElementRef<HTMLElement>>('formBanner');

  readonly returnUrl = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
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

  protected onEmailInput(event: InputCustomEvent): void {
    this.email.set(event.detail.value ?? '');
  }

  protected onLanguageChange(event: SelectCustomEvent<string>): void {
    this.language.set(event.detail.value);
  }

  protected errorText(field: Field): string | undefined {
    const error = this.form.of(field);
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : undefined;
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending()) {
      return;
    }
    this.form.clear();
    this.pending.set(true);
    try {
      await this.session.signUp(this.email(), this.password(), this.language());
    } catch (error) {
      await this.focus(this.form.set(error));
    } finally {
      this.pending.set(false);
    }
  }

  private async focus(field: Field): Promise<void> {
    if (field === 'email') {
      await this.emailField().setFocus();
    } else if (field === 'password') {
      await this.passwordField().focus();
    } else {
      this.formBanner()?.nativeElement.focus();
    }
  }
}
