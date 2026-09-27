import type { AdminUser } from '../core/admin-types';
import type { ConfirmRequest } from '../ui/confirm-action';

/** What an admin may do to an account (admin-console.md, "Administration"). */
export type UserAction = 'suspend' | 'reactivate' | 'export' | 'delete';

interface ActionKeys {
  readonly labelKey: string;
  readonly titleKey: string;
  readonly messageKey: string;
  readonly confirmKey: string;
  readonly doneKey: string;
  /** Whether the admin API records a reason for it: every action does, export included (H10). */
  readonly needsReason: boolean;
}

export const USER_ACTIONS: Readonly<Record<UserAction, ActionKeys>> = {
  suspend: {
    labelKey: 'admin.user.suspend.label',
    titleKey: 'admin.user.suspend.title',
    messageKey: 'admin.user.suspend.message',
    confirmKey: 'admin.user.suspend.confirm',
    doneKey: 'admin.user.suspend.done',
    needsReason: true,
  },
  reactivate: {
    labelKey: 'admin.user.reactivate.label',
    titleKey: 'admin.user.reactivate.title',
    messageKey: 'admin.user.reactivate.message',
    confirmKey: 'admin.user.reactivate.confirm',
    doneKey: 'admin.user.reactivate.done',
    needsReason: true,
  },
  export: {
    labelKey: 'admin.user.export.label',
    titleKey: 'admin.user.export.title',
    messageKey: 'admin.user.export.message',
    confirmKey: 'admin.user.export.confirm',
    doneKey: 'admin.user.export.done',
    needsReason: true,
  },
  delete: {
    labelKey: 'admin.user.delete.label',
    titleKey: 'admin.user.delete.title',
    messageKey: 'admin.user.delete.message',
    confirmKey: 'admin.user.delete.confirm',
    doneKey: 'admin.user.delete.done',
    needsReason: true,
  },
};

/** The actions an account offers: suspending an active one, reactivating a suspended one. */
export function availableActions(user: Pick<AdminUser, 'status'>): UserAction[] {
  return [user.status === 'active' ? 'suspend' : 'reactivate', 'export', 'delete'];
}

/** The confirmation of `action` on `user`, naming the environment and the account's address. */
export function confirmationOf(
  action: UserAction,
  user: Pick<AdminUser, 'email'>,
  environment: string,
): ConfirmRequest {
  const keys = USER_ACTIONS[action];
  return {
    titleKey: keys.titleKey,
    messageKey: keys.messageKey,
    confirmKey: keys.confirmKey,
    environment,
    target: user.email,
    needsReason: keys.needsReason,
  };
}
