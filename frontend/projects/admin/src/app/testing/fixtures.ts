import type {
  AdminUser,
  AdminUserDetail,
  Alert,
  AuditEntry,
  LogLine,
  Overview,
  PanelSeries,
  SupportRequest,
  SupportThread,
} from '../core/admin-types';

const DAY_MS = 86_400_000;

/** An ISO date `days` before now. */
export function daysAgo(days: number): string {
  return new Date(Date.now() - days * DAY_MS).toISOString();
}

export const OVERVIEW: Overview = {
  environment: 'staging',
  from: 0,
  to: 1,
  tiles: [
    { panel: 'up', value: 1, previous: 1 },
    { panel: 'errors', value: 0.02, previous: 0.01 },
    { panel: 'latency', value: 0.25, previous: 0.2 },
  ],
};

export const PANEL_SERIES: PanelSeries = {
  environment: 'staging',
  panel: 'latency',
  from: 100,
  to: 300,
  stepSeconds: 100,
  series: [{ name: 'p95', labels: {}, times: [100, 200], values: [0.2, 0.3] }],
  previous: [{ name: 'p95', labels: {}, times: [100, 200], values: [0.1, 0.4] }],
};

export const LOG_LINE: LogLine = {
  time: '2026-09-26T08:00:00Z',
  level: 'error',
  message: 'upstream timed out',
  requestId: 'req-42',
  fields: { route: '/api/v1/library' },
};

export const ALERT: Alert = {
  fingerprint: 'f1',
  name: 'HighErrorRate',
  severity: 'critical',
  state: 'firing',
  startsAt: '2026-09-26T07:00:00Z',
  summary: 'More than 5% of requests fail',
  labels: {},
};

export const USER: AdminUser = {
  id: 'u1',
  email: 'lee@example.com',
  emailVerified: true,
  plan: 'free',
  status: 'active',
  storageUsedBytes: 1_500_000,
  createdAt: '2026-01-02T10:00:00Z',
  lastSeenAt: '2026-09-25T10:00:00Z',
};

export const USER_DETAIL: AdminUserDetail = {
  ...USER,
  language: 'en',
  projectCount: 3,
  animationCount: 7,
  events: [{ name: 'export.completed', count: 4 }],
  sessions: [
    {
      createdAt: '2026-09-20T10:00:00Z',
      lastSeenAt: '2026-09-25T10:00:00Z',
      expiresAt: '2026-10-20T10:00:00Z',
    },
  ],
  supportRequests: [],
  tokens: [],
};

export const REQUEST: SupportRequest = {
  id: 'r1',
  accountId: 'u1',
  email: 'lee@example.com',
  category: 'data_protection',
  status: 'new',
  createdAt: daysAgo(10),
  updatedAt: daysAgo(1),
  ageSeconds: 10 * 86_400,
  hasScreenshot: true,
  assignedTo: null,
  firstResponseAt: null,
  resolvedAt: null,
};

export const THREAD: SupportThread = {
  request: REQUEST,
  hasScreenshot: true,
  context: { appVersion: '1.2.0', platform: 'web', language: 'en', screen: '/library' },
  messages: [
    {
      id: 'm1',
      author: 'user',
      body: 'Please send me all my data.',
      createdAt: REQUEST.createdAt,
      internal: false,
    },
    {
      id: 'm2',
      author: 'team',
      adminId: 'adm-1',
      body: 'Identity checked.',
      createdAt: REQUEST.updatedAt,
      internal: true,
    },
  ],
};

export const AUDIT_ENTRY: AuditEntry = {
  id: 7,
  at: '2026-09-26T09:00:00Z',
  adminId: 'adm-1',
  adminEmail: 'ops@example.com',
  action: 'user.suspend',
  targetType: 'user',
  targetId: 'u1',
  reason: 'Spam',
  before: null,
  after: null,
};
