import { DOCUMENT } from '@angular/common';
import { ChangeDetectionStrategy, Component, effect, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { NavigationEnd, Router, RouterLink } from '@angular/router';
import { AlertController } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { filter, map, startWith } from 'rxjs';
import { Icon } from '../ui/icon/icon';
import { MenuButton } from '../ui/menu/menu-button';
import { SessionStore } from './session-store';

/**
 * The header's account menu (accounts.md, H7): a sign-in link for a visitor, or a labelled menu
 * naming the address, with the account, the library and signing out. Also shows the blocking
 * dialog a `426` opens, since it is mounted on every page (`ACCOUNT_MENU_SLOT`).
 */
@Component({
  selector: 'lp-account-menu',
  imports: [Icon, MenuButton, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (session.account(); as account) {
      <lp-menu-button [label]="'shell.account_menu.label' | transloco">
        <span lpMenuTrigger class="trigger">
          <lp-icon name="account" />
          <span class="lp-visually-hidden">{{ 'shell.account_menu.label' | transloco }}</span>
          <span class="email">{{ account.email }}</span>
        </span>
        <p lpMenuHeader class="lp-menu__header">
          {{ 'shell.account_menu.signed_in_as' | transloco: { email: account.email } }}
        </p>
        <a role="menuitem" class="lp-menu__item" routerLink="/account">
          <lp-icon name="account" />{{ 'shell.account_menu.account' | transloco }}
        </a>
        <a role="menuitem" class="lp-menu__item" routerLink="/library">
          <lp-icon name="library" />{{ 'shell.account_menu.library' | transloco }}
        </a>
        <hr class="lp-menu__separator" />
        <button role="menuitem" class="lp-menu__item" type="button" (click)="signOut()">
          <lp-icon name="sign-out" />{{ 'shell.account_menu.sign_out' | transloco }}
        </button>
      </lp-menu-button>
    } @else {
      <a
        class="lp-button lp-button--secondary lp-button--compact"
        routerLink="/sign-in"
        [queryParams]="returnUrlParams()"
      >
        {{ 'shell.account_menu.sign_in' | transloco }}
      </a>
    }
  `,
  styles: `
    .trigger {
      display: inline-flex;
      gap: var(--lp-space-2);
      align-items: center;
      min-width: 0;
    }
    .email {
      max-width: min(16rem, 30vw);
      overflow: hidden;
      font-weight: var(--lp-font-weight-semibold);
      text-overflow: ellipsis;
      white-space: nowrap;
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
