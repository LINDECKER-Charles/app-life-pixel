import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  signal,
  viewChild,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonInput, type InputCustomEvent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AuthApi } from 'shared';
import { ACCOUNT_LIMITS } from './account-limits';
import { FieldMap, FormErrors } from './form-errors';

type Field = 'form';

const FIELD_BY_CODE: FieldMap<Field> = {};

/**
 * Asks for a link setting a new password: the answer is the same, and as quick, whether an
 * active account has the address or not — no field says otherwise (accounts.md, H7).
 */
@Component({
  selector: 'lp-reset-password-page',
  imports: [IonInput, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './reset-password-page.html',
  styleUrl: './account-form.scss',
})
export class ResetPasswordPage {
  private readonly authApi = inject(AuthApi);

  private readonly formBanner = viewChild<ElementRef<HTMLElement>>('formBanner');

  protected readonly limits = ACCOUNT_LIMITS;
  protected readonly email = signal('');
  protected readonly pending = signal(false);
  protected readonly sent = signal(false);
  protected readonly form = new FormErrors<Field>(FIELD_BY_CODE, 'form');

  protected onEmailInput(event: InputCustomEvent): void {
    this.email.set(event.detail.value ?? '');
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending()) {
      return;
    }
    this.form.clear();
    this.pending.set(true);
    try {
      await this.authApi.requestPasswordReset(this.email());
      this.sent.set(true);
    } catch (error) {
      this.form.set(error);
      this.formBanner()?.nativeElement.focus();
    } finally {
      this.pending.set(false);
    }
  }
}
