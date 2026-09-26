import { DOCUMENT } from '@angular/common';
import { ChangeDetectionStrategy, Component, effect, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { NavigationEnd, Router, RouterLink } from '@angular/router';
import { AlertController } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { filter, map, startWith } from 'rxjs';
import { SessionStore } from './session-store';

/**
 * The header's account menu (accounts.md, H7): a sign-in link for a visitor, or the address with
 * links to the account and the library and a sign-out button. Also shows the blocking dialog a
 * `426` opens, since it is mounted on every page (`ACCOUNT_MENU_SLOT`).
 */
@Component({
  selector: 'lp-account-menu',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (session.account(); as account) {
      <nav class="menu" [attr.aria-label]="'shell.account_menu.label' | transloco">
        <span class="email">{{ account.email }}</span>
        <a routerLink="/account">{{ 'shell.account_menu.account' | transloco }}</a>
        <a routerLink="/library">{{ 'shell.account_menu.library' | transloco }}</a>
        <button type="button" (click)="signOut()">
          {{ 'shell.account_menu.sign_out' | transloco }}
        </button>
      </nav>
    } @else {
      <a routerLink="/sign-in" [queryParams]="returnUrlParams()">
        {{ 'shell.account_menu.sign_in' | transloco }}
      </a>
    }
  `,
  styles: `
    .menu {
      display: flex;
      align-items: center;
      gap: var(--lp-space-3);
    }
    button {
      background: none;
      border: none;
      color: inherit;
      cursor: pointer;
      font: inherit;
    }
  `,
})
export class AccountMenu {
  protected readonly session = inject(SessionStore);
  private readonly router = inject(Router);
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);
  private readonly document = inject(DOCUMENT);

  // OnPush and mounted for the whole session (`ACCOUNT_MENU_SLOT`): a signal is what lets this
  // pick up a navigation that happens elsewhere, so `returnUrl` always names the current page.
  protected readonly returnUrlParams = toSignal(
    this.router.events.pipe(
      filter((event) => event instanceof NavigationEnd),
      map((event) => (event.url && event.url !== '/' ? { returnUrl: event.url } : {})),
      startWith(this.router.url && this.router.url !== '/' ? { returnUrl: this.router.url } : {}),
    ),
    { initialValue: {} },
  );

  constructor() {
    effect(() => {
      if (this.session.reloadRequired()) {
        void this.openReloadDialog();
      }
    });
  }

  protected async signOut(): Promise<void> {
    await this.session.signOut();
    await this.router.navigateByUrl('/editor');
  }

  private async openReloadDialog(): Promise<void> {
    const translate = (key: string): string => this.transloco.translate(key);
    const alert = await this.alerts.create({
      header: translate('shell.update_required.title'),
      message: translate('shell.update_required.message'),
      backdropDismiss: false,
      buttons: [
        {
          text: translate('shell.update_required.reload'),
          handler: () => {
            this.document.defaultView?.location.reload();
            return false;
          },
        },
      ],
    });
    await alert.present();
  }
}
