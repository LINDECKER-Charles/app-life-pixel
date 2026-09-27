import type { UserStatus } from '../core/admin-types';

/** The name of each account status. */
export const USER_STATUS_KEYS: Readonly<Record<UserStatus, string>> = {
  active: 'admin.users.status_active',
  suspended: 'admin.users.status_suspended',
};
