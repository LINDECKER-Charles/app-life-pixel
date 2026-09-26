import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  Injector,
  input,
  signal,
  viewChild,
} from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { AccountApi } from 'shared';
import { ErrorSummary } from '../form/error-summary';
import { fieldError } from '../form/field-error';
import { type FieldCheck, type FieldMap, FormErrors } from '../form/form-errors';
import { PasswordField } from '../form/password-field';
import { SessionStore } from '../session-store';

type Field = 'password' | 'form';

const FIELD_BY_CODE: FieldMap<Field> = {
  'auth.current_password': 'password',
  'auth.csrf': 'form',
};

/**
 * The account page's deletion, kept apart from sign-out and plain (design-system/docs/patterns.md,
 * "Destructive actions"): its consequences, then the password and the address typed again
 * (accounts.md, H7). The final button stays disabled until the address matches.
 */
@Component({
  selector: 'lp-account-deletion',
  imports: [ErrorSummary, PasswordField, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './account-deletion.html',
})
export class AccountDeletion {
  private readonly accountApi = inject(AccountApi);
  private readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly injector = inject(Injector);
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly summary = viewChild(ErrorSummary);
  private readonly startButton = viewChild<ElementRef<HTMLButtonElement>>('startButton');

  /** The account to delete: the confirmation names its address. */
  readonly account = input.required<{ readonly email: string }>();

  protected readonly targets = { password: 'account-delete-password' };
  protected readonly deleting = signal(false);
  protected readonly password = signal('');
  protected readonly confirmation = signal('');
  protected readonly pending = signal(false);
  protected readonly errors = new FormErrors<Field>(FIELD_BY_CODE, 'form');
  protected readonly confirmed = computed(() => this.confirmation() === this.account().email);

  /** Opens the form, its password field focused: the button that opened it is gone. */
  protected start(): void {
    this.deleting.set(true);
    this.password.set('');
    this.confirmation.set('');
    this.errors.clear();
    this.afterRender(() => this.host.nativeElement.querySelector('input')?.focus());
  }

  /** Closes the form, the focus back on the button that opens it. */
  protected cancel(): void {
    this.deleting.set(false);
    this.afterRender(() => this.startButton()?.nativeElement.focus());
  }

  protected onPassword(value: string): void {
    this.password.set(value);
    this.errors.recheck('password', this.check()[1]);
  }

  protected async confirm(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (this.pending() || !this.confirmed()) {
      return;
    }
    if (!this.errors.check([this.check()])) {
      this.summary()?.focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      await this.accountApi.delete(this.password());
      this.session.handleUnauthenticated();
      await this.router.navigateByUrl('/editor');
    } catch (error) {
      this.errors.set(error);
      this.summary()?.focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private check(): FieldCheck<Field> {
    return ['password', fieldError(this.password() !== '', 'auth.form.password_required')];
  }

  private afterRender(action: () => void): void {
    afterNextRender(action, { injector: this.injector });
  }
}
