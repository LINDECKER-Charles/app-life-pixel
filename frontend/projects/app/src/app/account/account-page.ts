import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  signal,
  viewChild,
} from '@angular/core';
import { Router } from '@angular/router';
import {
  IonContent,
  IonInput,
  IonSelect,
  IonSelectOption,
  type InputCustomEvent,
  type SelectCustomEvent,
} from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { AccountApi, ApiProblem, AuthApi, AvailableLanguages } from 'shared';
import { FieldMap, FormErrors } from './form-errors';
import { PasswordField } from './password-field';
import { SessionStore } from './session-store';

type DeletionField = 'password' | 'form';

const DELETION_FIELD_BY_CODE: FieldMap<DeletionField> = {
  'auth.current_password': 'password',
  'auth.csrf': 'form',
};

/**
 * The signed-in account (accounts.md, H7): its address and verification, its storage against the
 * quota, its language, a data export, deletion, and sign-out. `requireAccount` keeps this page for
 * a signed-in visitor.
 */
@Component({
  selector: 'lp-account-page',
  imports: [IonContent, IonInput, IonSelect, IonSelectOption, PasswordField, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './account-page.html',
  styleUrl: './account-page.scss',
})
export class AccountPage {
  private readonly session = inject(SessionStore);
  private readonly authApi = inject(AuthApi);
  protected readonly accountApi = inject(AccountApi);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);

  private readonly deletionPasswordField = viewChild<PasswordField>('deletionPasswordField');
  private readonly deletionBanner = viewChild<ElementRef<HTMLElement>>('deletionBanner');

  protected readonly account = this.session.account;
  protected readonly languages = inject(AvailableLanguages).languages;

  protected readonly resendPending = signal(false);
  protected readonly resendSent = signal(false);
  protected readonly resendError = signal<ApiProblem | undefined>(undefined);

  protected readonly languagePending = signal(false);
  protected readonly languageError = signal<ApiProblem | undefined>(undefined);

  protected readonly deleting = signal(false);
  protected readonly deletionPassword = signal('');
  protected readonly deletionConfirmation = signal('');
  protected readonly deletionPending = signal(false);
  protected readonly deletion = new FormErrors<DeletionField>(DELETION_FIELD_BY_CODE, 'form');

  protected deletionErrorText(field: DeletionField): string | undefined {
    const error = this.deletion.of(field);
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : undefined;
  }

  protected async resendVerification(): Promise<void> {
    if (this.resendPending()) {
      return;
    }
    this.resendPending.set(true);
    this.resendError.set(undefined);
    try {
      await this.authApi.resendVerificationEmail();
      this.resendSent.set(true);
    } catch (error) {
      if (!(error instanceof ApiProblem)) {
        throw error;
      }
      this.resendError.set(error);
    } finally {
      this.resendPending.set(false);
    }
  }

  protected async changeLanguage(event: SelectCustomEvent<string>): Promise<void> {
    this.languagePending.set(true);
    this.languageError.set(undefined);
    try {
      const account = await this.accountApi.updateLanguage(event.detail.value);
      this.session.setAccount(account);
    } catch (error) {
      if (!(error instanceof ApiProblem)) {
        throw error;
      }
      this.languageError.set(error);
    } finally {
      this.languagePending.set(false);
    }
  }

  protected startDeletion(): void {
    this.deleting.set(true);
    this.deletionPassword.set('');
    this.deletionConfirmation.set('');
    this.deletion.clear();
  }

  protected cancelDeletion(): void {
    this.deleting.set(false);
  }

  protected onDeletionConfirmationInput(event: InputCustomEvent): void {
    this.deletionConfirmation.set(event.detail.value ?? '');
  }

  protected deletionConfirmed(account: { email: string }): boolean {
    return this.deletionConfirmation() === account.email;
  }

  protected async confirmDeletion(event: SubmitEvent, account: { email: string }): Promise<void> {
    event.preventDefault();
    if (this.deletionPending() || !this.deletionConfirmed(account)) {
      return;
    }
    this.deletion.clear();
    this.deletionPending.set(true);
    try {
      await this.accountApi.delete(this.deletionPassword());
      this.session.handleUnauthenticated();
      await this.router.navigateByUrl('/editor');
    } catch (error) {
      const field = this.deletion.set(error);
      if (field === 'password') {
        await this.deletionPasswordField()?.focus();
      } else {
        this.deletionBanner()?.nativeElement.focus();
      }
    } finally {
      this.deletionPending.set(false);
    }
  }

  protected async signOut(): Promise<void> {
    await this.session.signOut();
    await this.router.navigateByUrl('/editor');
  }
}
