import type { Alert } from '../core/admin-types';
import { EnvironmentReadings, overviewTiles } from './overview-tiles';

const ALERT: Alert = {
  fingerprint: 'f1',
  name: 'HighErrorRate',
  labels: {},
  startsAt: '2026-09-26T08:00:00Z',
  state: 'firing',
};

const READINGS: EnvironmentReadings = {
  env: 'staging',
  overview: {
    environment: 'staging',
    from: 0,
    to: 1,
    tiles: [
      { panel: 'up', value: 1, previous: 1 },
      { panel: 'errors', value: 0.02, previous: 0.01 },
      { panel: 'latency', value: 0.25, previous: null },
    ],
  },
  alerts: [ALERT, ALERT],
  support: {
    openByStatus: [
      { status: 'new', count: 3 },
      { status: 'in_progress', count: 2 },
    ],
  },
  isOwn: true,
};

describe('overview tiles', () => {
  it('shows health, errors, latency, traffic, storage, support and alerts, in that order', () => {
    const tiles = overviewTiles(READINGS, 'en');

    expect(tiles.map((tile) => tile.id)).toEqual([
      'up',
      'errors',
      'latency',
      'requests',
      'exports',
      'mcp',
      'storage',
      'support',
      'alerts',
    ]);
    expect(tiles.every((tile) => tile.env === 'staging')).toBe(true);
  });

  it('opens each tile’s detail', () => {
    const links = Object.fromEntries(
      overviewTiles(READINGS, 'en').map((tile) => [tile.id, tile.link]),
    );

    expect(links['latency']).toBe('/monitoring/latency');
    expect(links['support']).toBe('/support');
    expect(links['alerts']).toBe('/alerts');
  });

  it('gives each value and the previous period’s', () => {
    const [up, errors, latency, requests] = overviewTiles(READINGS, 'en');

    expect(up.value.key).toBe('admin.value.up');
    expect(errors.value.params).toEqual({ value: '2%' });
    expect(errors.previous?.params).toEqual({ value: '1%' });
    expect(latency.previous?.key).toBe('admin.value.none');
    expect(requests.value.key).toBe('admin.value.none');
    expect(requests.previous).toBeNull();
  });

  it('counts the open support requests and the firing alerts', () => {
    const tiles = overviewTiles(READINGS, 'en');

    expect(tiles.find((tile) => tile.id === 'support')?.value.params).toEqual({ value: '5' });
    expect(tiles.find((tile) => tile.id === 'alerts')?.value.params).toEqual({ value: '2' });
  });

  it('says where another environment’s support queue is', () => {
    const tiles = overviewTiles(
      { ...READINGS, env: 'production', isOwn: false, support: null, alerts: null },
      'en',
    );

    expect(tiles.find((tile) => tile.id === 'support')?.value.key).toBe(
      'admin.overview.support_elsewhere',
    );
    expect(tiles.find((tile) => tile.id === 'alerts')?.value.key).toBe('admin.value.none');
  });
});
