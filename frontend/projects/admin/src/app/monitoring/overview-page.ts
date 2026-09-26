import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { asProblem } from '../core/problem';
import { SessionStore } from '../core/session-store';
import type { ViewState } from '../core/view-state';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { EnvironmentName } from '../ui/environment-name';
import { MonitoringApi } from './monitoring-api';
import { EnvironmentReadings, overviewTiles } from './overview-tiles';

/** One environment's section: what was read, and why a part is missing. */
interface Section {
  readonly readings: EnvironmentReadings;
  readonly overviewProblem: ApiProblem | null;
  readonly alertsProblem: ApiProblem | null;
}

async function settle<T>(read: Promise<T>): Promise<[T | null, ApiProblem | null]> {
  try {
    return [await read, null];
  } catch (error: unknown) {
    return [null, asProblem(error)];
  }
}

/**
 * "Is everything fine?" (admin-console.md): for each environment the monitoring shows, the tiles
 * of its health, errors, latency, traffic, storage, open support requests and firing alerts,
 * each opening its detail.
 */
@Component({
  selector: 'lp-overview-page',
  imports: [RouterLink, TranslocoPipe, EmptyState, EnvironmentName],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './overview-page.html',
  styleUrl: './overview-page.scss',
})
export class OverviewPage {
  private readonly api = inject(MonitoringApi);
  private readonly session = inject(SessionStore);
  private readonly view = inject(ViewStateStore);
  private readonly transloco = inject(TranslocoService);
  private generation = 0;

  protected readonly params = this.view.params;
  protected readonly sections = signal<readonly Section[]>([]);
  protected readonly loading = signal(false);
  protected readonly tiles = computed(() =>
    this.sections().map((section) => ({
      ...section,
      tiles: overviewTiles(section.readings, this.transloco.getActiveLang()),
    })),
  );

  constructor() {
    effect(() => {
      const environments = this.environments();
      const state = this.view.state();
      untracked(() => void this.load(environments, state));
    });
  }

  private environments(): readonly string[] {
    const monitored = this.session.monitoredEnvironments();
    return monitored.length > 0 ? monitored : [this.session.environment()];
  }

  private async load(environments: readonly string[], state: ViewState): Promise<void> {
    const generation = ++this.generation;
    this.loading.set(true);
    const [metrics] = await settle(this.api.productMetrics());
    const sections = await Promise.all(
      environments.map((env) => this.section({ ...state, env }, metrics?.support ?? null)),
    );
    if (generation === this.generation) {
      this.sections.set(sections);
      this.loading.set(false);
    }
  }

  private async section(
    state: ViewState,
    support: EnvironmentReadings['support'],
  ): Promise<Section> {
    const isOwn = state.env === this.session.environment();
    const [[overview, overviewProblem], [alerts, alertsProblem]] = await Promise.all([
      settle(this.api.overview(state)),
      settle(this.api.alerts(state.env)),
    ]);
    return {
      readings: { env: state.env, overview, alerts, support: isOwn ? support : null, isOwn },
      overviewProblem,
      alertsProblem,
    };
  }
}
