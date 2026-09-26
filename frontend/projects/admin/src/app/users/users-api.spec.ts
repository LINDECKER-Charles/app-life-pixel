import { TestBed } from '@angular/core/testing';
import { FakeAdminClient, useFakeAdminClient } from '../testing/fake-admin-client';
import { UsersApi } from './users-api';

describe('UsersApi', () => {
  let client: FakeAdminClient;
  let api: UsersApi;

  beforeEach(() => {
    client = useFakeAdminClient();
    client.request.mockResolvedValue(undefined);
    api = TestBed.inject(UsersApi);
  });

  it('searches the users by address or id, page after page', async () => {
    await api.search({ q: 'lee', status: 'suspended', cursor: 'c1' });

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/users', {
      query: { q: 'lee', status: 'suspended', cursor: 'c1' },
    });
  });

  it('reads a user', async () => {
    await api.get('u1');

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/users/{id}', {
      path: { id: 'u1' },
    });
  });

  it('sends the reason of every change', async () => {
    await api.suspend('u1', 'Spam');
    await api.reactivate('u1', 'Appeal accepted');
    await api.delete('u1', 'Erasure request');

    expect(client.request.mock.calls).toEqual([
      [
        'post',
        '/api/admin/v1/users/{id}/suspend',
        { path: { id: 'u1' }, body: { reason: 'Spam' } },
      ],
      [
        'post',
        '/api/admin/v1/users/{id}/reactivate',
        { path: { id: 'u1' }, body: { reason: 'Appeal accepted' } },
      ],
      [
        'delete',
        '/api/admin/v1/users/{id}',
        { path: { id: 'u1' }, body: { reason: 'Erasure request' } },
      ],
    ]);
  });

  it('downloads a user’s export', async () => {
    const file = { blob: new Blob(['zip']), fileName: 'export.zip' };
    client.download.mockResolvedValue(file);

    expect(await api.export('u1')).toBe(file);
    expect(client.download).toHaveBeenCalledWith('/api/admin/v1/users/{id}/export', { id: 'u1' });
  });
});
