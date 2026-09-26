import { TestBed } from '@angular/core/testing';
import { FakeAdminClient, useFakeAdminClient } from '../testing/fake-admin-client';
import { MonitoringApi } from './monitoring-api';

const VIEW = { env: 'production', from: 'now-6h', to: 'now' };

describe('MonitoringApi', () => {
  let client: FakeAdminClient;
  let api: MonitoringApi;

  beforeEach(() => {
    client = useFakeAdminClient();
    api = TestBed.inject(MonitoringApi);
  });

  it('reads an environment’s tiles over the range', async () => {
    client.request.mockResolvedValue({ tiles: [] });

    await api.overview(VIEW);

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/monitoring/overview', {
      query: { env: 'production', from: 'now-6h', to: 'now' },
    });
  });

  it('reads a panel by its name, never by a query', async () => {
    client.request.mockResolvedValue({ series: [], previous: [] });

    await api.series(VIEW, 'latency');

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/monitoring/series', {
      query: { env: 'production', panel: 'latency', from: 'now-6h', to: 'now' },
    });
  });

  it('reads the log lines by level and text', async () => {
    client.request.mockResolvedValue({ lines: [{ message: 'boom' }] });

    const lines = await api.logs(VIEW, { level: 'error', q: 'timeout' });

    expect(lines).toEqual([{ message: 'boom' }]);
    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/logs', {
      query: { env: 'production', from: 'now-6h', to: 'now', level: 'error', q: 'timeout' },
    });
  });

  it('reads the alerts and a request’s Grafana link', async () => {
    client.request.mockResolvedValueOnce({ alerts: ['a'] });
    client.request.mockResolvedValueOnce({ url: 'https://grafana.example/explore' });

    expect(await api.alerts('staging')).toEqual(['a']);
    expect(await api.grafanaLink(VIEW, 'req-1')).toBe('https://grafana.example/explore');
    expect(client.request).toHaveBeenLastCalledWith('get', '/api/admin/v1/links/grafana', {
      query: { env: 'production', requestId: 'req-1', from: 'now-6h', to: 'now' },
    });
  });

  it('reads the product metrics of this console’s environment', async () => {
    client.request.mockResolvedValue({ support: { openByStatus: [] } });

    await api.productMetrics();

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/metrics/product');
  });
});
