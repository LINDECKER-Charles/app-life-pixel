import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import type { PanelSeries } from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatValue } from '../core/format';
import { asProblem } from '../core/problem';
import type { ViewState } from '../core/view-state';
import { ViewStateStore } from '../core/view-state-store';
import { TableSort } from '../core/table-sort';
import { EmptyState } from '../ui/empty-state';
import { EnvironmentName } from '../ui/environment-name';
import { SortButton } from '../ui/sort-button';
import { alignSeries, hasPoints, LineSummary, summarize } from './chart-data';
import { MonitoringApi } from './monitoring-api';
import { PanelChart } from './panel-chart';
import { PanelInfo, panelInfo, PANELS } from './panels';

type SummaryKey = 'label' | 'period' | 'last' | 'min' | 'max';

/**
 * A panel's chart over the range, with the previous period dashed beside it (admin-console.md),
 * titled by the question it answers, and its values as a table for keyboards, screen readers
 * and CSV.
 */
@Component({
  selector: 'lp-panel-page',
  imports: [RouterLink, TranslocoPipe, EmptyState, EnvironmentName, PanelChart, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './panel-page.html',
  styleUrl: './monitoring.scss',
})
export class PanelPage {
  private readonly api = inject(MonitoringApi);
  private readonly view = inject(ViewStateStore);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private generation = 0;

  /** Bound from the route's `:panel`. */
  readonly panel = input.required<string>();

  protected readonly panels = PANELS;
  protected readonly columns: readonly { key: SummaryKey; labelKey: string }[] = [
    { key: 'label', labelKey: 'admin.chart.series' },
    { key: 'period', labelKey: 'admin.chart.period' },
    { key: 'last', labelKey: 'admin.chart.last' },
    { key: 'min', labelKey: 'admin.chart.min' },
    { key: 'max', labelKey: 'admin.chart.max' },
  ];
  protected readonly state = this.view.state;
  protected readonly params = this.view.params;
  protected readonly info = computed(() => panelInfo(this.panel()));
  protected readonly series = signal<PanelSeries | null>(null);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly loading = signal(false);
  protected readonly data = computed(() => {
    const series = this.series();
    return series ? alignSeries(series) : null;
  });
  protected readonly empty = computed(() => {
    const data = this.data();
    return data === null || !hasPoints(data);
  });
  protected readonly rows = computed(() => this.data()?.lines.map(summarize) ?? []);
  protected readonly table = new TableSort<LineSummary, SummaryKey>(
    this.rows,
    { key: 'label', direction: 'ascending' },
    (row, key) => (key === 'period' ? row.previous : row[key]),
  );
  protected readonly format = computed(() => {
    const unit = this.info()?.unit ?? 'count';
    return (value: number): string => this.text(formatValue(value, unit, this.language()));
  });

  constructor() {
    effect(() => {
      const info = this.info();
      const state = this.state();
      untracked(() => void (info ? this.load(info, state) : this.series.set(null)));
    });
  }

  protected value(value: number | null): string {
    return this.text(formatValue(value, this.info()?.unit ?? 'count', this.language()));
  }

  protected exportCsv(info: PanelInfo): void {
    const period = (row: LineSummary): string =>
      this.transloco.translate(row.previous ? 'admin.chart.previous' : 'admin.chart.current');
    const columns: CsvColumn<LineSummary>[] = [
      { header: this.transloco.translate('admin.chart.series'), value: (row) => row.label },
      { header: this.transloco.translate('admin.chart.period'), value: period },
      { header: this.transloco.translate('admin.chart.last'), value: (row) => row.last },
      { header: this.transloco.translate('admin.chart.min'), value: (row) => row.min },
      { header: this.transloco.translate('admin.chart.max'), value: (row) => row.max },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: `panel-${info.panel}`,
      environment: this.state().env,
    });
  }

  private language(): string {
    return this.transloco.getActiveLang();
  }

  private text(message: { key: string; params: Readonly<Record<string, unknown>> }): string {
    return this.transloco.translate(message.key, message.params);
  }

  private async load(info: PanelInfo, state: ViewState): Promise<void> {
    const generation = ++this.generation;
    this.loading.set(true);
    let series: PanelSeries | null = null;
    let problem: ApiProblem | null = null;
    try {
      series = await this.api.series(state, info.panel);
    } catch (error: unknown) {
      problem = asProblem(error);
    }
    if (generation === this.generation) {
      this.series.set(series);
      this.problem.set(problem);
      this.loading.set(false);
    }
  }
}
