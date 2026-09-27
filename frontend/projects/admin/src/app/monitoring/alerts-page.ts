import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  signal,
  untracked,
} from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import type { Alert } from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatDateTime } from '../core/format';
import { asProblem } from '../core/problem';
import { TableSort } from '../core/table-sort';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { EnvironmentName } from '../ui/environment-name';
import { SortButton } from '../ui/sort-button';
import { MonitoringApi } from './monitoring-api';

type AlertKey = 'name' | 'severity' | 'state' | 'startsAt';

/** The alerts firing in an environment, as Alertmanager reports them (admin-console.md). */
@Component({
  selector: 'lp-alerts-page',
  imports: [TranslocoPipe, EmptyState, EnvironmentName, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './alerts-page.html',
})
export class AlertsPage {
  private readonly api = inject(MonitoringApi);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private generation = 0;

  protected readonly state = inject(ViewStateStore).state;
  protected readonly alerts = signal<readonly Alert[]>([]);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly loading = signal(false);
  protected readonly table = new TableSort<Alert, AlertKey>(
    this.alerts,
    { key: 'startsAt', direction: 'descending' },
    (alert, key) => alert[key],
  );

  constructor() {
    effect(() => {
      const env = this.state().env;
      untracked(() => void this.load(env));
    });
  }

  protected time(iso: string): string {
    return formatDateTime(iso, this.transloco.getActiveLang());
  }

  protected exportCsv(): void {
    const header = (key: string): string => this.transloco.translate(key);
    const columns: CsvColumn<Alert>[] = [
      { header: header('admin.alerts.name'), value: (alert) => alert.name },
      { header: header('admin.alerts.severity'), value: (alert) => alert.severity },
      { header: header('admin.alerts.state'), value: (alert) => alert.state },
      { header: header('admin.alerts.since'), value: (alert) => alert.startsAt },
      { header: header('admin.alerts.summary'), value: (alert) => alert.summary },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: 'alerts',
      environment: this.state().env,
    });
  }

  private async load(env: string): Promise<void> {
    const generation = ++this.generation;
    this.loading.set(true);
    let alerts: readonly Alert[] = [];
    let problem: ApiProblem | null = null;
    try {
      alerts = await this.api.alerts(env);
    } catch (error: unknown) {
      problem = asProblem(error);
    }
    if (generation === this.generation) {
      this.alerts.set(alerts);
      this.problem.set(problem);
      this.loading.set(false);
    }
  }
}
