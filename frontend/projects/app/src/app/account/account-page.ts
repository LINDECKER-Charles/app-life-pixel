import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { Router } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { AccountApi, ApiProblem, AuthApi, AvailableLanguages } from 'shared';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { AccountDeletion } from './deletion/account-deletion';
import { SessionStore } from './session-store';

/**
 * The signed-in account (accounts.md, H7): its address and verification, its storage against the
 * quota, its language, a data export, sign-out and, apart from them, deletion. `requireAccount`
 * keeps this page for a signed-in visitor.
 */
@Component({
  selector: 'lp-account-page',
  imports: [AccountDeletion, Icon, IonContent, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './account-page.html',
  styleUrl: './account-page.scss',
})
export class AccountPage {
  private readonly session = inject(SessionStore);
  private readonly authApi = inject(AuthApi);
  protected readonly accountApi = inject(AccountApi);
  private readonly router = inject(Router);

  protected readonly account = this.session.account;
  protected readonly languages = inject(AvailableLanguages).languages;

  protected readonly resendPending = signal(false);
  protected readonly resendSent = signal(false);
  protected readonly resendError = signal<ApiProblem | undefined>(undefined);

  protected readonly languagePending = signal(false);
  protected readonly languageError = signal<ApiProblem | undefined>(undefined);

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
      this.resendError.set(problem(error));
    } finally {
      this.resendPending.set(false);
    }
  }

  protected async changeLanguage(language: string): Promise<void> {
    this.languagePending.set(true);
    this.languageError.set(undefined);
    try {
      const account = await this.accountApi.updateLanguage(language);
      this.session.setAccount(account);
    } catch (error) {
      this.languageError.set(problem(error));
    } finally {
      this.languagePending.set(false);
    }
  }

  protected async signOut(): Promise<void> {
    await this.session.signOut();
    await this.router.navigateByUrl('/editor');
  }
}

/** `error` as a problem to show; anything but an `ApiProblem` is rethrown. */
function problem(error: unknown): ApiProblem {
  if (!(error instanceof ApiProblem)) {
    throw error;
  }
  return error;
}
