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
import { IonInput, type InputCustomEvent } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { ACCOUNT_LIMITS } from './account-limits';
import { FieldMap, FormErrors } from './form-errors';
import { PasswordField } from './password-field';
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
  imports: [IonInput, PasswordField, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sign-in-page.html',
  styleUrl: './account-form.scss',
})
export class SignInPage {
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);

  private readonly emailField = viewChild.required<IonInput>('emailField');
  private readonly passwordField = viewChild.required<PasswordField>('passwordField');
  private readonly formBanner = viewChild<ElementRef<HTMLElement>>('formBanner');

  readonly returnUrl = input<string>();

  protected readonly limits = ACCOUNT_LIMITS;
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

  protected onEmailInput(event: InputCustomEvent): void {
    this.email.set(event.detail.value ?? '');
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
      await this.session.signIn(this.email(), this.password());
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
