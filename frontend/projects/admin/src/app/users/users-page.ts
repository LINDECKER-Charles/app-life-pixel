import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  untracked,
} from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { AdminUser, UserStatus } from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatBytes, formatDate } from '../core/format';
import { PagedList } from '../core/paged-list';
import { SessionStore } from '../core/session-store';
import { TableSort } from '../core/table-sort';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { SortButton } from '../ui/sort-button';
import { UserSearch, UsersApi } from './users-api';
import { USER_STATUS_KEYS } from './user-labels';

type UserKey = 'email' | 'plan' | 'status' | 'storageUsedBytes' | 'createdAt' | 'lastSeenAt';

function isStatus(value: string | undefined): value is UserStatus {
  return value === 'active' || value === 'suspended';
}

/** The users found by address or id, page after page, in a sortable table (admin-console.md). */
@Component({
  selector: 'lp-users-page',
  imports: [RouterLink, TranslocoPipe, EmptyState, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './users-page.html',
})
export class UsersPage {
  private readonly api = inject(UsersApi);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private readonly environment = inject(SessionStore).environment;

  /** Bound from the query: `?q=lee&status=suspended`. */
  readonly q = input<string>();
  readonly status = input<string>();

  protected readonly params = inject(ViewStateStore).params;
  protected readonly statusKeys = USER_STATUS_KEYS;
  protected readonly columns: readonly { key: UserKey; labelKey: string }[] = [
    { key: 'email', labelKey: 'admin.users.email' },
    { key: 'plan', labelKey: 'admin.users.plan' },
    { key: 'status', labelKey: 'admin.users.status' },
    { key: 'storageUsedBytes', labelKey: 'admin.users.storage' },
    { key: 'createdAt', labelKey: 'admin.users.created' },
    { key: 'lastSeenAt', labelKey: 'admin.users.last_seen' },
  ];
  protected readonly search = computed<UserSearch>(() => {
    const status = this.status();
    return { q: this.q()?.trim() || undefined, status: isStatus(status) ? status : undefined };
  });
  protected readonly list = new PagedList<AdminUser, UserSearch>((search, cursor) =>
    this.api.search({ ...search, cursor }),
  );
  protected readonly table = new TableSort<AdminUser, UserKey>(
    this.list.items,
    { key: 'createdAt', direction: 'descending' },
    (user, key) => user[key],
  );

  constructor() {
    effect(() => {
      const search = this.search();
      untracked(() => void this.list.reload(search));
    });
  }

  protected date(iso: string | null | undefined): string {
    return iso ? formatDate(iso, this.transloco.getActiveLang()) : '';
  }

  protected bytes(value: number): string {
    return formatBytes(value, this.transloco.getActiveLang());
  }

  protected submit(event: SubmitEvent): void {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const q = String(form.get('q') ?? '').trim();
    const status = String(form.get('status') ?? '');
    void this.router.navigate([], {
      queryParams: { q: q || null, status: status || null },
      queryParamsHandling: 'merge',
    });
  }

  protected more(): void {
    void this.list.more();
  }

  protected exportCsv(): void {
    const header = (key: string): string => this.transloco.translate(key);
    const columns: CsvColumn<AdminUser>[] = [
      { header: header('admin.users.id'), value: (user) => user.id },
      { header: header('admin.users.email'), value: (user) => user.email },
      { header: header('admin.users.email_verified'), value: (user) => user.emailVerified },
      { header: header('admin.users.plan'), value: (user) => user.plan },
      { header: header('admin.users.status'), value: (user) => user.status },
      { header: header('admin.users.storage'), value: (user) => user.storageUsedBytes },
      { header: header('admin.users.created'), value: (user) => user.createdAt },
      { header: header('admin.users.last_seen'), value: (user) => user.lastSeenAt },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: 'users',
      environment: this.environment(),
    });
  }
}
