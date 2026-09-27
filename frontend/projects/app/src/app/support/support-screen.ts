import { DestroyRef, inject, Injectable } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { type ActivatedRouteSnapshot, NavigationEnd, Router } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { filter } from 'rxjs';
import { LIFE_PIXEL_CLIENT, type SupportRequestContext } from 'shared';

/** The screen a request is sent from when the app opened on support directly. */
const FIRST_SCREEN = '/editor';

/** `/support/:requestId` and `/support`: the pages a request is sent from, never its subject. */
function isSupport(template: string): boolean {
  return template === '/support' || template.startsWith('/support/');
}

/** The route template of the state under `root`, `/editor/:animationId`: never an id. */
export function routeTemplate(root: ActivatedRouteSnapshot): string {
  const segments: string[] = [];
  for (let route: ActivatedRouteSnapshot | null = root; route; route = route.firstChild) {
    const path = route.routeConfig?.path;
    if (path) {
      segments.push(path);
    }
  }
  return `/${segments.join('/')}`;
}

/**
 * The context a support request carries without asking (support-admin.md, H9): the app's
 * version, platform and language, and the route template of the last screen visited before
 * support. The shell calls `start()` once, when it is created: the screen already shown counts.
 */
@Injectable({ providedIn: 'root' })
export class SupportScreen {
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);
  private readonly client = inject(LIFE_PIXEL_CLIENT);
  private readonly transloco = inject(TranslocoService);
  private screen = FIRST_SCREEN;
  private started = false;

  start(): void {
    if (this.started) {
      return;
    }
    this.started = true;
    if (this.router.navigated) {
      this.remember(routeTemplate(this.router.routerState.snapshot.root));
    }
    this.router.events
      .pipe(
        filter((event) => event instanceof NavigationEnd),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(() => this.remember(routeTemplate(this.router.routerState.snapshot.root)));
  }

  /** The context of a request sent now. */
  context(): SupportRequestContext {
    const [platform, appVersion] = this.client.split('/');
    return { appVersion, platform, language: this.transloco.getActiveLang(), screen: this.screen };
  }

  private remember(template: string): void {
    if (!isSupport(template)) {
      this.screen = template;
    }
  }
}
