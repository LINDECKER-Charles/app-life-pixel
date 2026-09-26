import { ChangeDetectionStrategy, Component, effect, inject } from '@angular/core';
import { Router, RouterOutlet } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { SessionStore } from './core/session-store';
import { AppHeader } from './shell/app-header';

const SIGN_IN = '/sign-in';

/**
 * The console's shell: a skip link, the header once signed in, and the page. A session that
 * ends — signed out elsewhere, or expired — goes back to signing in, then to the same page.
 */
@Component({
  selector: 'lp-root',
  imports: [RouterOutlet, TranslocoPipe, AppHeader],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  private readonly router = inject(Router);

  protected readonly signedIn = inject(SessionStore).signedIn;

  constructor() {
    effect(() => {
      const url = this.router.url;
      if (!this.signedIn() && this.router.navigated && !url.startsWith(SIGN_IN)) {
        void this.router.navigate([SIGN_IN], { queryParams: { returnUrl: url } });
      }
    });
  }
}
