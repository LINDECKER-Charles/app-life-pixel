import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { ALERT, OVERVIEW } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { MonitoringApi } from './monitoring-api';
import { OverviewPage } from './overview-page';

function fakeApi(): Record<'overview' | 'alerts' | 'productMetrics', ReturnType<typeof vi.fn>> {
  return {
    overview: vi.fn().mockResolvedValue(OVERVIEW),
    alerts: vi.fn().mockResolvedValue([ALERT]),
    productMetrics: vi.fn().mockResolvedValue({
      support: { openByStatus: [{ status: 'new', count: 4 }] },
    }),
  };
}

async function openOverview(api = fakeApi(), url = '/?from=now-7d'): Promise<HTMLElement> {
  const fixture = await openPage(OverviewPage, {
    url,
    providers: [{ provide: MonitoringApi, useValue: api }],
  });
  await settled(() => expect(fixture.nativeElement.querySelectorAll('section').length).toBe(2));
  return fixture.nativeElement;
}

describe('OverviewPage', () => {
  it('shows the tiles of every monitored environment over the range', async () => {
    const api = fakeApi();
    const page = await openOverview(api);
    const [staging, production] = Array.from(page.querySelectorAll('section'));

    expect(textOf(staging.querySelector('h2'))).toBe('Staging');
    expect(textOf(production.querySelector('h2'))).toBe('Production');
    expect(staging.querySelectorAll('.tile').length).toBe(9);
    expect(api.overview).toHaveBeenCalledWith({ env: 'production', from: 'now-7d', to: 'now' });
  });

  it('opens each tile’s detail in its environment and the same range', async () => {
    const page = await openOverview();
    const production = page.querySelectorAll('section')[1];
    const latency = production.querySelector('a[href^="/monitoring/latency"]');

    expect(latency?.getAttribute('href')).toBe(
      '/monitoring/latency?env=production&from=now-7d&to=now',
    );
    expect(textOf(latency)).toContain('p95 latency');
    expect(textOf(latency)).toContain('Previous period: 200 ms');
  });

  it('counts this environment’s open requests, and says where another’s are', async () => {
    const page = await openOverview();
    const [staging, production] = Array.from(page.querySelectorAll('section'));

    expect(textOf(staging.querySelector('a[href^="/support"]'))).toContain('4');
    expect(textOf(production.querySelector('a[href^="/support"]'))).toContain(
      'Open that environment’s console',
    );
    expect(textOf(staging.querySelector('a[href^="/alerts"]'))).toContain('1');
  });

  it('says why an environment has no tiles', async () => {
    const api = fakeApi();
    api.overview.mockRejectedValue(
      new ApiProblem(503, 'monitoring.not_configured', { source: 'prometheus' }),
    );
    const page = await openOverview(api);

    const empty = page.querySelector('[data-reason="not_configured"]');
    expect(textOf(empty)).toContain('This source is not configured on the admin server.');
  });

  it('passes axe', async () => {
    const page = await openOverview();

    expect(await seriousViolations(page)).toEqual([]);
  });
});
