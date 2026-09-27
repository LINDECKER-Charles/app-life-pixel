import { TestBed } from '@angular/core/testing';
import { useFakeAdminClient } from '../testing/fake-admin-client';
import { AuditApi } from './audit-api';

describe('AuditApi', () => {
  it('searches the audit log by admin, action and target', async () => {
    const client = useFakeAdminClient();
    client.request.mockResolvedValue({ items: [] });

    await TestBed.inject(AuditApi).search({ action: 'user.delete', targetId: 'u1' }, 'c3');

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/audit-log', {
      query: { action: 'user.delete', targetId: 'u1', cursor: 'c3' },
    });
  });
});
