import { ComponentFixture } from '@angular/core/testing';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { LOG_LINE } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { LogsPage } from './logs-page';
import { MonitoringApi } from './monitoring-api';

function fakeApi(): Record<'logs' | 'grafanaLink', ReturnType<typeof vi.fn>> {
  return {
    logs: vi.fn().mockResolvedValue([LOG_LINE]),
    grafanaLink: vi.fn().mockResolvedValue('https://grafana.example/explore?left=req-42'),
  };
}

async function openLogs(
  api = fakeApi(),
  inputs: Record<string, unknown> = {},
): Promise<ComponentFixture<LogsPage>> {
  return openPage(LogsPage, {
    url: '/logs?env=production&from=now-1h',
    inputs,
    providers: [{ provide: MonitoringApi, useValue: api }],
  });
}

describe('LogsPage', () => {
  it('reads the lines of the environment and the range, by level and text', async () => {
    const api = fakeApi();
    const fixture = await openLogs(api, { level: 'error', q: 'timeout' });

    await settled(() => expect(fixture.nativeElement.querySelectorAll('tbody tr').length).toBe(1));
    expect(api.logs).toHaveBeenCalledWith(
      { env: 'production', from: 'now-1h', to: 'now' },
      { level: 'error', q: 'timeout' },
    );
    expect(textOf(fixture.nativeElement.querySelector('h2#logs-lines'))).toBe('1 line');
  });

  it('opens a request’s own lines from its id', async () => {
    const fixture = await openLogs();
    await settled(() => expect(fixture.nativeElement.querySelectorAll('tbody tr').length).toBe(1));

    const link = fixture.nativeElement.querySelector('a[aria-label^="Open the logs"]');
    expect(link.getAttribute('aria-label')).toBe('Open the logs of request req-42');
    expect(link.getAttribute('href')).toBe(
      '/logs?env=production&from=now-1h&q=req-42&request=req-42',
    );
  });

  it('links a request to its trace in Grafana', async () => {
    const api = fakeApi();
    const fixture = await openLogs(api, { request: 'req-42', q: 'req-42' });

    await settled(() =>
      expect(fixture.nativeElement.querySelector('a[target="_blank"]')?.getAttribute('href')).toBe(
        'https://grafana.example/explore?left=req-42',
      ),
    );
    expect(api.grafanaLink).toHaveBeenCalledWith(
      { env: 'production', from: 'now-1h', to: 'now' },
      'req-42',
    );
  });

  it('says why there is no line', async () => {
    const api = fakeApi();
    api.logs.mockRejectedValue(
      new ApiProblem(503, 'monitoring.not_configured', { source: 'loki' }),
    );
    const fixture = await openLogs(api);

    await settled(() =>
      expect(fixture.nativeElement.querySelector('[data-reason="not_configured"]')).not.toBeNull(),
    );
  });

  it('says there is nothing when no line matches', async () => {
    const api = fakeApi();
    api.logs.mockResolvedValue([]);
    const fixture = await openLogs(api);

    await settled(() =>
      expect(textOf(fixture.nativeElement.querySelector('[data-reason="nothing"]'))).toBe(
        'No log line matches these filters over this period.',
      ),
    );
  });

  it('passes axe', async () => {
    const fixture = await openLogs(fakeApi(), { request: 'req-42' });
    await settled(() =>
      expect(fixture.nativeElement.querySelector('a[target="_blank"]')).not.toBeNull(),
    );

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
