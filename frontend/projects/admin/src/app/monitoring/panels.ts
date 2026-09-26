import type { Panel } from '../core/admin-types';

/** How a panel's values read: each unit has its own formatting. */
export type PanelUnit = 'health' | 'rate' | 'ratio' | 'seconds' | 'count' | 'bytes' | 'cores';

/** A panel as the console shows it: its short name, the question its chart answers, its unit. */
export interface PanelInfo {
  readonly panel: Panel;
  readonly nameKey: string;
  readonly questionKey: string;
  readonly unit: PanelUnit;
}

/** Every panel of the admin server (support-admin.md, H11), in the order the views list them. */
export const PANELS: readonly PanelInfo[] = [
  {
    panel: 'up',
    nameKey: 'admin.panel.up.name',
    questionKey: 'admin.panel.up.question',
    unit: 'health',
  },
  {
    panel: 'errors',
    nameKey: 'admin.panel.errors.name',
    questionKey: 'admin.panel.errors.question',
    unit: 'ratio',
  },
  {
    panel: 'latency',
    nameKey: 'admin.panel.latency.name',
    questionKey: 'admin.panel.latency.question',
    unit: 'seconds',
  },
  {
    panel: 'requests',
    nameKey: 'admin.panel.requests.name',
    questionKey: 'admin.panel.requests.question',
    unit: 'rate',
  },
  {
    panel: 'slow_routes',
    nameKey: 'admin.panel.slow_routes.name',
    questionKey: 'admin.panel.slow_routes.question',
    unit: 'seconds',
  },
  {
    panel: 'exports',
    nameKey: 'admin.panel.exports.name',
    questionKey: 'admin.panel.exports.question',
    unit: 'count',
  },
  {
    panel: 'mcp',
    nameKey: 'admin.panel.mcp.name',
    questionKey: 'admin.panel.mcp.question',
    unit: 'count',
  },
  {
    panel: 'auth',
    nameKey: 'admin.panel.auth.name',
    questionKey: 'admin.panel.auth.question',
    unit: 'count',
  },
  {
    panel: 'storage',
    nameKey: 'admin.panel.storage.name',
    questionKey: 'admin.panel.storage.question',
    unit: 'bytes',
  },
  {
    panel: 'quota',
    nameKey: 'admin.panel.quota.name',
    questionKey: 'admin.panel.quota.question',
    unit: 'count',
  },
  {
    panel: 'memory',
    nameKey: 'admin.panel.memory.name',
    questionKey: 'admin.panel.memory.question',
    unit: 'bytes',
  },
  {
    panel: 'cpu',
    nameKey: 'admin.panel.cpu.name',
    questionKey: 'admin.panel.cpu.question',
    unit: 'cores',
  },
];

/** The panel of a route's `:panel`, or `null` for a name the admin server does not know. */
export function panelInfo(panel: string): PanelInfo | null {
  return PANELS.find((info) => info.panel === panel) ?? null;
}
