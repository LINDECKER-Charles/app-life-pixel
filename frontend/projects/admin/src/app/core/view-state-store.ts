import { computed, inject, Injectable } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { NavigationEnd, Params, Router } from '@angular/router';
import { filter, map } from 'rxjs';
import { SessionStore } from './session-store';
import { readViewState, ViewState, viewParams } from './view-state';

/**
 * The environment and the time range of the URL, shared by every view: a change navigates, so
 * that the address bar always holds a link to what is on screen.
 */
@Injectable({ providedIn: 'root' })
export class ViewStateStore {
  private readonly router = inject(Router);
  private readonly session = inject(SessionStore);
  private readonly query = toSignal(
    this.router.events.pipe(
      filter((event) => event instanceof NavigationEnd),
      map(() => this.currentQuery()),
    ),
    { initialValue: this.currentQuery() },
  );

  readonly state = computed<ViewState>(() =>
    readViewState(this.query(), this.session.monitoredEnvironments(), this.session.environment()),
  );
  /** The query parameters every link between views carries. */
  readonly params = computed(() => viewParams(this.state()));

  /** Moves the current view to another environment or range, keeping its other parameters. */
  update(change: Partial<ViewState>): Promise<boolean> {
    return this.router.navigate([], { queryParams: change, queryParamsHandling: 'merge' });
  }

  private currentQuery(): Params {
    return this.router.routerState.snapshot.root.queryParams;
  }
}
