import { inject, Injectable } from '@angular/core';
import { AdminClient, DownloadedFile } from '../core/admin-client';
import type { AdminUserDetail, UserPage, UserStatus } from '../core/admin-types';

/** What the users view searches by. */
export interface UserSearch {
  readonly q?: string;
  readonly status?: UserStatus;
  readonly cursor?: string;
}

/**
 * The users of this console's environment, through the relayed internal admin API
 * (support-admin.md, H10): each change carries its reason, kept in the audit log.
 */
@Injectable({ providedIn: 'root' })
export class UsersApi {
  private readonly client = inject(AdminClient);

  search(search: UserSearch): Promise<UserPage> {
    return this.client.request('get', '/api/admin/v1/users', {
      query: { q: search.q, status: search.status, cursor: search.cursor },
    });
  }

  get(id: string): Promise<AdminUserDetail> {
    return this.client.request('get', '/api/admin/v1/users/{id}', { path: { id } });
  }

  /** Suspends the account: it may not sign in, and its sessions end. */
  async suspend(id: string, reason: string): Promise<void> {
    await this.client.request('post', '/api/admin/v1/users/{id}/suspend', {
      path: { id },
      body: { reason },
    });
  }

  async reactivate(id: string, reason: string): Promise<void> {
    await this.client.request('post', '/api/admin/v1/users/{id}/reactivate', {
      path: { id },
      body: { reason },
    });
  }

  /** The account's data, as the zip its owner's export gives (right of access), with `reason`. */
  export(id: string, reason: string): Promise<DownloadedFile> {
    return this.client.downloadPost('/api/admin/v1/users/{id}/export', {
      path: { id },
      body: { reason },
    });
  }

  /** Erases the account, its documents included (right to erasure). */
  async delete(id: string, reason: string): Promise<void> {
    await this.client.request('delete', '/api/admin/v1/users/{id}', {
      path: { id },
      body: { reason },
    });
  }
}
