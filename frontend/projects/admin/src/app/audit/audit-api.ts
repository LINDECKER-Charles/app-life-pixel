import { inject, Injectable } from '@angular/core';
import { AdminClient } from '../core/admin-client';
import type { AuditPage } from '../core/admin-types';

/** What the audit log is searched by. */
export interface AuditSearch {
  readonly adminId?: string;
  readonly action?: string;
  readonly targetId?: string;
}

/** The actions the audit log records (support-admin.md, H10). */
export const AUDIT_ACTIONS: readonly string[] = [
  'user.suspend',
  'user.reactivate',
  'user.export',
  'user.delete',
  'support.update',
  'support.reply',
  'support.note',
];

/** The audit log of this console's environment: append-only, newest first. */
@Injectable({ providedIn: 'root' })
export class AuditApi {
  private readonly client = inject(AdminClient);

  search(search: AuditSearch, cursor?: string): Promise<AuditPage> {
    return this.client.request('get', '/api/admin/v1/audit-log', {
      query: { ...search, cursor },
    });
  }
}
