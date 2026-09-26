import { inject, Injectable } from '@angular/core';
import { AdminClient } from '../core/admin-client';
import type {
  Alert,
  LogLevel,
  LogLine,
  Overview,
  Panel,
  PanelSeries,
  ProductMetrics,
} from '../core/admin-types';
import type { ViewState } from '../core/view-state';

/** What the logs view filters by, beyond the environment and the range. */
export interface LogFilters {
  readonly level?: LogLevel;
  readonly q?: string;
}

/**
 * The monitoring routes of the admin server (support-admin.md, H11): the browser names a panel,
 * never a query. A source not configured throws `monitoring.not_configured`, one that fails
 * `monitoring.unavailable`.
 */
@Injectable({ providedIn: 'root' })
export class MonitoringApi {
  private readonly client = inject(AdminClient);

  overview(view: ViewState): Promise<Overview> {
    return this.client.request('get', '/api/admin/v1/monitoring/overview', {
      query: { env: view.env, from: view.from, to: view.to },
    });
  }

  series(view: ViewState, panel: Panel): Promise<PanelSeries> {
    return this.client.request('get', '/api/admin/v1/monitoring/series', {
      query: { env: view.env, panel, from: view.from, to: view.to },
    });
  }

  async logs(view: ViewState, filters: LogFilters): Promise<readonly LogLine[]> {
    const answer = await this.client.request('get', '/api/admin/v1/logs', {
      query: { env: view.env, from: view.from, to: view.to, level: filters.level, q: filters.q },
    });
    return answer.lines;
  }

  async alerts(env: string): Promise<readonly Alert[]> {
    const answer = await this.client.request('get', '/api/admin/v1/alerts', { query: { env } });
    return answer.alerts;
  }

  /** The Grafana Explore link to a request's logs and trace. */
  async grafanaLink(view: ViewState, requestId: string): Promise<string> {
    const answer = await this.client.request('get', '/api/admin/v1/links/grafana', {
      query: { env: view.env, requestId, from: view.from, to: view.to },
    });
    return answer.url;
  }

  /** The product metrics and the support queue's measures of this console's environment. */
  productMetrics(): Promise<ProductMetrics> {
    return this.client.request('get', '/api/admin/v1/metrics/product');
  }
}
