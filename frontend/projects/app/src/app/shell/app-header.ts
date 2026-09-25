import { NgComponentOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { IonHeader, IonToolbar } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { ACCOUNT_MENU_SLOT } from './account-menu-slot';

/** A link of the header: its path and the i18n key of its text. */
interface NavigationLink {
  readonly path: string;
  readonly label: string;
}

/** Help leads to support (H9's Help → Contact). */
const NAVIGATION_LINKS: readonly NavigationLink[] = [
  { path: '/editor', label: 'shell.nav.editor' },
  { path: '/library', label: 'shell.nav.library' },
  { path: '/settings', label: 'shell.nav.settings' },
  { path: '/support', label: 'shell.nav.help' },
];

/** The app's header: its name, the links to its areas, and the account menu H7 provides. */
@Component({
  selector: 'lp-app-header',
  imports: [IonHeader, IonToolbar, NgComponentOutlet, RouterLink, RouterLinkActive, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app-header.html',
  styleUrl: './app-header.scss',
})
export class AppHeader {
  protected readonly links = NAVIGATION_LINKS;
  protected readonly accountMenu = inject(ACCOUNT_MENU_SLOT, { optional: true });
}
