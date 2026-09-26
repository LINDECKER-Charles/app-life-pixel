import type { Alert, Overview, Panel, ProductMetrics } from '../core/admin-types';
import { formatValue } from '../core/format';
import type { Message } from '../core/problem';
import { panelInfo } from './panels';

/** A tile of the overview: its name, its value, the previous period's, and where it leads. */
export interface TileView {
  readonly id: string;
  readonly nameKey: string;
  readonly value: Message;
  readonly previous: Message | null;
  /** The detail it opens: a panel, the alerts, or the support queue. */
  readonly link: string;
  /** The environment its detail shows. */
  readonly env: string;
}

/** What the overview read for one environment; `null` while unread or when it failed. */
export interface EnvironmentReadings {
  readonly env: string;
  readonly overview: Overview | null;
  readonly alerts: readonly Alert[] | null;
  /** The support queue's measures: only this console's own environment has them. */
  readonly support: ProductMetrics['support'] | null;
  readonly isOwn: boolean;
}

/** The panels of the tiles, in the order of admin-console.md's overview. */
export const TILE_PANELS: readonly Panel[] = [
  'up',
  'errors',
  'latency',
  'requests',
  'exports',
  'mcp',
  'storage',
];

const NO_VALUE: Message = { key: 'admin.value.none', params: {} };

function countOf(value: number, language: string): Message {
  return formatValue(value, 'count', language);
}

function panelTile(readings: EnvironmentReadings, panel: Panel, language: string): TileView {
  const info = panelInfo(panel);
  const unit = info?.unit ?? 'count';
  const tile = readings.overview?.tiles.find((candidate) => candidate.panel === panel);
  return {
    id: panel,
    nameKey: info?.nameKey ?? panel,
    value: formatValue(tile?.value, unit, language),
    previous: tile ? formatValue(tile.previous, unit, language) : null,
    link: `/monitoring/${panel}`,
    env: readings.env,
  };
}

function supportTile(readings: EnvironmentReadings, language: string): TileView {
  const open = readings.support?.openByStatus.reduce((sum, status) => sum + status.count, 0);
  let value = NO_VALUE;
  if (!readings.isOwn) {
    value = { key: 'admin.overview.support_elsewhere', params: {} };
  } else if (open !== undefined) {
    value = countOf(open, language);
  }
  return {
    id: 'support',
    nameKey: 'admin.overview.tile.support',
    value,
    previous: null,
    link: '/support',
    env: readings.env,
  };
}

function alertsTile(readings: EnvironmentReadings, language: string): TileView {
  return {
    id: 'alerts',
    nameKey: 'admin.overview.tile.alerts',
    value: readings.alerts === null ? NO_VALUE : countOf(readings.alerts.length, language),
    previous: null,
    link: '/alerts',
    env: readings.env,
  };
}

/** The tiles of one environment: health, errors, latency, traffic, storage, support, alerts. */
export function overviewTiles(readings: EnvironmentReadings, language: string): TileView[] {
  return [
    ...TILE_PANELS.map((panel) => panelTile(readings, panel, language)),
    supportTile(readings, language),
    alertsTile(readings, language),
  ];
}
