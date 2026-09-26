import type { SupportCategory, SupportStatus } from '../core/admin-types';

/** The name of each category of request. */
export const SUPPORT_CATEGORY_KEYS: Readonly<Record<SupportCategory, string>> = {
  bug: 'admin.support.category.bug',
  account: 'admin.support.category.account',
  billing: 'admin.support.category.billing',
  data_protection: 'admin.support.category.data_protection',
  abuse: 'admin.support.category.abuse',
  other: 'admin.support.category.other',
};

/** The name of each status: new → in progress → waiting for the user → resolved → closed. */
export const SUPPORT_STATUS_KEYS: Readonly<Record<SupportStatus, string>> = {
  new: 'admin.support.status.new',
  in_progress: 'admin.support.status.in_progress',
  waiting_for_user: 'admin.support.status.waiting_for_user',
  resolved: 'admin.support.status.resolved',
  closed: 'admin.support.status.closed',
};
