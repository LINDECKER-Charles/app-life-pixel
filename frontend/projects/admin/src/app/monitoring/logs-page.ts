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
import { Router, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { LOG_LEVELS, type LogLevel, type LogLine } from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatDateTime } from '../core/format';
import { asProblem } from '../core/problem';
import { TableSort } from '../core/table-sort';
import type { ViewState } from '../core/view-state';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { EnvironmentName } from '../ui/environment-name';
import { SortButton } from '../ui/sort-button';
import { LogFilters, MonitoringApi } from './monitoring-api';

type LogKey = 'time' | 'level' | 'message' | 'requestId';

const LEVEL_KEYS: Readonly<Record<LogLevel, string>> = {
  error: 'admin.logs.level_error',
  warn: 'admin.logs.level_warn',
  info: 'admin.logs.level_info',
  debug: 'admin.logs.level_debug',
  trace: 'admin.logs.level_trace',
};

function isLevel(value: string | undefined): value is LogLevel {
  return LOG_LEVELS.some((level) => level === value);
}

/**
 * The logs of an environment over the range, filtered by level and text (admin-console.md): each
 * line's request id opens that request's lines and its Grafana trace. Every filter is in the URL.
 */
@Component({
  selector: 'lp-logs-page',
  imports: [RouterLink, TranslocoPipe, EmptyState, EnvironmentName, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './logs-page.html',
  styleUrl: './logs-page.scss',
})
export class LogsPage {
  private readonly api = inject(MonitoringApi);
  private readonly view = inject(ViewStateStore);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private generation = 0;

  /** Bound from the query: `?level=error&q=timeout&request=<id>`. */
  readonly level = input<string>();
  readonly q = input<string>();
  readonly request = input<string>();

  protected readonly levels = LOG_LEVELS;
  protected readonly levelKeys = LEVEL_KEYS;
  protected readonly state = this.view.state;
  protected readonly filters = computed<LogFilters>(() => {
    const level = this.level();
    return { level: isLevel(level) ? level : undefined, q: this.q() || undefined };
  });
  protected readonly lines = signal<readonly LogLine[]>([]);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly loading = signal(false);
  protected readonly grafana = signal<string | null>(null);
  protected readonly grafanaProblem = signal<ApiProblem | null>(null);
  protected readonly table = new TableSort<LogLine, LogKey>(
    this.lines,
    { key: 'time', direction: 'descending' },
    (line, key) => line[key],
  );

  constructor() {
    effect(() => {
      const [state, filters] = [this.state(), this.filters()];
      untracked(() => void this.load(state, filters));
    });
    effect(() => {
      const [state, request] = [this.state(), this.request()];
      untracked(() => void this.loadGrafana(state, request));
    });
  }

  protected time(iso: string): string {
    return formatDateTime(iso, this.transloco.getActiveLang());
  }

  protected fieldsOf(line: LogLine): [string, string][] {
    return Object.entries(line.fields);
  }

  protected search(event: SubmitEvent): void {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const level = String(form.get('level') ?? '');
    const q = String(form.get('q') ?? '').trim();
    void this.router.navigate([], {
      queryParams: { level: level || null, q: q || null, request: null },
      queryParamsHandling: 'merge',
    });
  }

  protected exportCsv(): void {
    const header = (key: string): string => this.transloco.translate(key);
    const columns: CsvColumn<LogLine>[] = [
      { header: header('admin.logs.time'), value: (line) => line.time },
      { header: header('admin.logs.level'), value: (line) => line.level },
      { header: header('admin.logs.message'), value: (line) => line.message },
      { header: header('admin.logs.request_id'), value: (line) => line.requestId },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: 'logs',
      environment: this.state().env,
    });
  }

  private async load(state: ViewState, filters: LogFilters): Promise<void> {
    const generation = ++this.generation;
    this.loading.set(true);
    let lines: readonly LogLine[] = [];
    let problem: ApiProblem | null = null;
    try {
      lines = await this.api.logs(state, filters);
    } catch (error: unknown) {
      problem = asProblem(error);
    }
    if (generation === this.generation) {
      this.lines.set(lines);
      this.problem.set(problem);
      this.loading.set(false);
    }
  }

  private async loadGrafana(state: ViewState, request: string | undefined): Promise<void> {
    this.grafana.set(null);
    this.grafanaProblem.set(null);
    if (!request) {
      return;
    }
    try {
      this.grafana.set(await this.api.grafanaLink(state, request));
    } catch (error: unknown) {
      this.grafanaProblem.set(asProblem(error));
    }
  }
}
