import { NgComponentOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { PlatformService } from '../../platform/platform';
import { Icon } from '../../ui/icon/icon';
import type { IconName } from '../../ui/icon/icon-paths';
import { ACCOUNT_MENU_SLOT } from '../account-menu-slot';

/** A link of the header: its path, the i18n key of its text, its icon and where it exists. */
interface NavigationLink {
  readonly path: string;
  readonly label: string;
  readonly icon: IconName;
  readonly isHostedOnly: boolean;
}

/** Help leads to support (H9's Help → Contact), a hosted-only route (desktop.md, T2). */
const NAVIGATION_LINKS: readonly NavigationLink[] = [
  { path: '/editor', label: 'shell.nav.editor', icon: 'pencil', isHostedOnly: false },
  { path: '/library', label: 'shell.nav.library', icon: 'library', isHostedOnly: false },
  { path: '/settings', label: 'shell.nav.settings', icon: 'settings', isHostedOnly: false },
  { path: '/support', label: 'shell.nav.help', icon: 'help', isHostedOnly: true },
];

/** Life Pixel's mark, copied from design-system/assets/ by tools/copy-design-assets.mjs. */
const MARK_SOURCE = '/design-system/mark.svg';

/**
 * The app's header: the mark and the name, the links to its areas, and the account menu H7
 * provides. A link whose route this platform cannot serve is left out: the desktop has no Help,
 * since support is hosted only (design-system/docs/journeys.md).
 */
@Component({
  selector: 'lp-app-header',
  imports: [Icon, NgComponentOutlet, RouterLink, RouterLinkActive, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app-header.html',
  styleUrl: './app-header.scss',
})
export class AppHeader {
  private readonly isDesktop = inject(PlatformService).desktop;

  protected readonly markSource = MARK_SOURCE;
  protected readonly links = NAVIGATION_LINKS.filter(
    (link) => !(link.isHostedOnly && this.isDesktop),
  );
  protected readonly accountMenu = inject(ACCOUNT_MENU_SLOT, { optional: true });
}
