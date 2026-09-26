import { TestBed } from '@angular/core/testing';
import { FakeAdminClient, useFakeAdminClient } from '../testing/fake-admin-client';
import { SupportApi } from './support-api';

describe('SupportApi', () => {
  let client: FakeAdminClient;
  let api: SupportApi;

  beforeEach(() => {
    client = useFakeAdminClient();
    client.request.mockResolvedValue(undefined);
    api = TestBed.inject(SupportApi);
  });

  it('reads the queue by status, category and assignee', async () => {
    await api.queue({ status: 'new', category: 'data_protection', assignee: 'adm-1' }, 'c2');

    expect(client.request).toHaveBeenCalledWith('get', '/api/admin/v1/support-requests', {
      query: { status: 'new', category: 'data_protection', assignee: 'adm-1', cursor: 'c2' },
    });
  });

  it('reads a thread, changes its status or assignee, and posts to it', async () => {
    await api.thread('r1');
    await api.update('r1', { status: 'resolved' });
    await api.post('r1', 'Fixed in 1.2', false);

    expect(client.request.mock.calls).toEqual([
      ['get', '/api/admin/v1/support-requests/{id}', { path: { id: 'r1' } }],
      [
        'patch',
        '/api/admin/v1/support-requests/{id}',
        { path: { id: 'r1' }, body: { status: 'resolved' } },
      ],
      [
        'post',
        '/api/admin/v1/support-requests/{id}/messages',
        { path: { id: 'r1' }, body: { body: 'Fixed in 1.2', internal: false } },
      ],
    ]);
  });

  it('downloads the screenshot', async () => {
    const blob = new Blob(['png']);
    client.download.mockResolvedValue({ blob, fileName: null });

    expect(await api.screenshot('r1')).toBe(blob);
    expect(client.download).toHaveBeenCalledWith('/api/admin/v1/support-requests/{id}/screenshot', {
      id: 'r1',
    });
  });
});
