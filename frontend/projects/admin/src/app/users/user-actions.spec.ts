import { availableActions, confirmationOf } from './user-actions';

describe('user actions', () => {
  it('offers suspending an active account and reactivating a suspended one', () => {
    expect(availableActions({ status: 'active' })).toEqual(['suspend', 'export', 'delete']);
    expect(availableActions({ status: 'suspended' })).toEqual(['reactivate', 'export', 'delete']);
  });

  it('confirms an action by naming the environment and the account', () => {
    expect(confirmationOf('delete', { email: 'lee@example.com' }, 'production')).toEqual({
      titleKey: 'admin.user.delete.title',
      messageKey: 'admin.user.delete.message',
      confirmKey: 'admin.user.delete.confirm',
      environment: 'production',
      target: 'lee@example.com',
      needsReason: true,
    });
  });

  it('asks a reason for every change the audit log records', () => {
    const user = { email: 'lee@example.com' };

    expect(confirmationOf('suspend', user, 'staging').needsReason).toBe(true);
    expect(confirmationOf('reactivate', user, 'staging').needsReason).toBe(true);
    expect(confirmationOf('export', user, 'staging').needsReason).toBe(true);
  });
});
