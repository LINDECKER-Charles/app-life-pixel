import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { SessionStore } from '../core/session-store';
import { ViewStateStore } from '../core/view-state-store';
import { EnvironmentBanner } from './environment-banner';
import { ViewControls } from './view-controls';

/** A section of the console, in the order of the navigation. */
interface NavItem {
  readonly path: string;
  readonly labelKey: string;
  readonly exact: boolean;
}

const NAV: readonly NavItem[] = [
  { path: '/', labelKey: 'admin.nav.overview', exact: true },
  { path: '/monitoring/requests', labelKey: 'admin.nav.monitoring', exact: false },
  { path: '/logs', labelKey: 'admin.nav.logs', exact: false },
  { path: '/alerts', labelKey: 'admin.nav.alerts', exact: false },
  { path: '/users', labelKey: 'admin.nav.users', exact: false },
  { path: '/support', labelKey: 'admin.nav.support', exact: false },
  { path: '/audit', labelKey: 'admin.nav.audit', exact: false },
];

/**
 * The console's header: the environment banner, the navigation — each link carrying the
 * environment and the range —, the shared selectors and signing out.
 */
@Component({
  selector: 'lp-app-header',
  imports: [RouterLink, RouterLinkActive, TranslocoPipe, EnvironmentBanner, ViewControls],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app-header.html',
  styleUrl: './app-header.scss',
})
export class AppHeader {
  private readonly session = inject(SessionStore);

  protected readonly nav = NAV;
  protected readonly params = inject(ViewStateStore).params;
  protected readonly environment = this.session.environment;
  protected readonly admin = this.session.admin;

  /** Signed out, even when the server cannot be reached: the shell then goes to signing in. */
  protected signOut(): void {
    this.session.signOut().catch((error: unknown) => console.error('Signing out failed.', error));
  }
}
