import { inject, Injectable } from '@angular/core';
import { AdminClient } from '../core/admin-client';
import type {
  SupportCategory,
  SupportMessage,
  SupportPage,
  SupportPatch,
  SupportRequest,
  SupportStatus,
  SupportThread,
} from '../core/admin-types';

/** What the queue filters by. */
export interface QueueFilters {
  readonly status?: SupportStatus;
  readonly category?: SupportCategory;
  /** An admin id. */
  readonly assignee?: string;
}

/**
 * The support queue of this console's environment, through the relayed internal admin API
 * (support-admin.md, H10): the threads with their internal notes, replies emailed to the person.
 */
@Injectable({ providedIn: 'root' })
export class SupportApi {
  private readonly client = inject(AdminClient);

  queue(filters: QueueFilters, cursor?: string): Promise<SupportPage> {
    return this.client.request('get', '/api/admin/v1/support-requests', {
      query: { ...filters, cursor },
    });
  }

  thread(id: string): Promise<SupportThread> {
    return this.client.request('get', '/api/admin/v1/support-requests/{id}', { path: { id } });
  }

  update(id: string, patch: SupportPatch): Promise<SupportRequest> {
    return this.client.request('patch', '/api/admin/v1/support-requests/{id}', {
      path: { id },
      body: patch,
    });
  }

  /** A reply, emailed to the person, or an internal note they never see. */
  post(id: string, body: string, internal: boolean): Promise<SupportMessage> {
    return this.client.request('post', '/api/admin/v1/support-requests/{id}/messages', {
      path: { id },
      body: { body, internal },
    });
  }

  async screenshot(id: string): Promise<Blob> {
    const file = await this.client.download('/api/admin/v1/support-requests/{id}/screenshot', {
      id,
    });
    return file.blob;
  }
}
