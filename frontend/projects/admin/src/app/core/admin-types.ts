import type { components } from 'admin-api';

type Schemas = components['schemas'];

/** The admin signed in, the CSRF token, and the environments (support-admin.md, H11). */
export type ConsoleSession = Schemas['ConsoleSession'];
/** What signing in sends. */
export type SignInRequest = Schemas['SignInRequest'];

/** A panel of the monitoring views. */
export type Panel = Schemas['Panel'];
/** The tiles of an environment. */
export type Overview = Schemas['Overview'];
/** A tile: a panel's value at the end of the range, and one range earlier. */
export type Tile = Schemas['Tile'];
/** A panel's series, and the previous period's moved onto the range. */
export type PanelSeries = Schemas['PanelSeries'];
/** One series of a panel. */
export type Series = Schemas['Series'];
/** A log level the logs filter by. */
export type LogLevel = Schemas['LogLevel'];
/** One log line. */
export type LogLine = Schemas['LogLine'];
/** A firing alert. */
export type Alert = Schemas['Alert'];
/** The product metrics of a period, and the support queue's measures. */
export type ProductMetrics = Schemas['ProductMetricsBody'];

/** Whether an account may sign in. */
export type UserStatus = Schemas['AdminUserStatus'];
/** A user as the search lists them. */
export type AdminUser = Schemas['AdminUser'];
/** A user in detail. */
export type AdminUserDetail = Schemas['AdminUserDetail'];
/** A page of users. */
export type UserPage = Schemas['Page_AdminUser'];

/** What a support request is about. */
export type SupportCategory = Schemas['SupportCategory'];
/** Where a support request stands. */
export type SupportStatus = Schemas['SupportRequestStatus'];
/** A request as the team sees it, without its messages. */
export type SupportRequest = Schemas['AdminSupportRequest'];
/** A request with its context and every message, internal notes included. */
export type SupportThread = Schemas['AdminSupportThread'];
/** A message of a request. */
export type SupportMessage = Schemas['AdminSupportMessage'];
/** A page of the queue. */
export type SupportPage = Schemas['Page_AdminSupportRequest'];
/** A change of a request's status or assignee. */
export type SupportPatch = Schemas['RequestPatch'];

/** An entry of the audit log. */
export type AuditEntry = Schemas['AuditEntryBody'];
/** A page of the audit log. */
export type AuditPage = Schemas['Page_AuditEntryBody'];

export const SUPPORT_STATUSES: readonly SupportStatus[] = [
  'new',
  'in_progress',
  'waiting_for_user',
  'resolved',
  'closed',
];

export const SUPPORT_CATEGORIES: readonly SupportCategory[] = [
  'bug',
  'account',
  'billing',
  'data_protection',
  'abuse',
  'other',
];

export const LOG_LEVELS: readonly LogLevel[] = ['error', 'warn', 'info', 'debug', 'trace'];
