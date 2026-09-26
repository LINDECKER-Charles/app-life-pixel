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
import {
  SUPPORT_CATEGORIES,
  SUPPORT_STATUSES,
  type SupportCategory,
  type SupportRequest,
  type SupportStatus,
} from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatAge, formatDate } from '../core/format';
import { PagedList } from '../core/paged-list';
import { SessionStore } from '../core/session-store';
import { TableSort } from '../core/table-sort';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { SortButton } from '../ui/sort-button';
import { DeadlineNote } from './deadline-note';
import { legalDeadline } from './legal-deadline';
import { QueueFilters, SupportApi } from './support-api';
import { SUPPORT_CATEGORY_KEYS, SUPPORT_STATUS_KEYS } from './support-labels';

type QueueKey = 'category' | 'email' | 'status' | 'ageSeconds' | 'updatedAt' | 'deadline';

function oneOf<T extends string>(values: readonly T[], value: string | undefined): T | undefined {
  return values.find((candidate) => candidate === value);
}

/** The support queue, filtered and sorted (admin-console.md); every filter is in the URL. */
@Component({
  selector: 'lp-support-queue-page',
  imports: [RouterLink, TranslocoPipe, DeadlineNote, EmptyState, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './support-queue-page.html',
})
export class SupportQueuePage {
  private readonly api = inject(SupportApi);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private readonly session = inject(SessionStore);

  /** Bound from the query: `?status=new&category=bug&assignee=me`. */
  readonly status = input<string>();
  readonly category = input<string>();
  readonly assignee = input<string>();

  protected readonly params = inject(ViewStateStore).params;
  protected readonly statuses = SUPPORT_STATUSES;
  protected readonly categories = SUPPORT_CATEGORIES;
  protected readonly statusKeys = SUPPORT_STATUS_KEYS;
  protected readonly categoryKeys = SUPPORT_CATEGORY_KEYS;
  protected readonly columns: readonly { key: QueueKey; labelKey: string }[] = [
    { key: 'category', labelKey: 'admin.support.category_label' },
    { key: 'email', labelKey: 'admin.support.email' },
    { key: 'status', labelKey: 'admin.support.status_label' },
    { key: 'ageSeconds', labelKey: 'admin.support.age' },
    { key: 'updatedAt', labelKey: 'admin.support.updated' },
    { key: 'deadline', labelKey: 'admin.support.deadline_label' },
  ];
  protected readonly adminId = computed(() => this.session.admin()?.id ?? null);
  protected readonly filters = computed<QueueFilters>(() => ({
    status: oneOf<SupportStatus>(SUPPORT_STATUSES, this.status()),
    category: oneOf<SupportCategory>(SUPPORT_CATEGORIES, this.category()),
    assignee: this.assignee() === 'me' ? (this.adminId() ?? undefined) : undefined,
  }));
  protected readonly list = new PagedList<SupportRequest, QueueFilters>((filters, cursor) =>
    this.api.queue(filters, cursor),
  );
  protected readonly table = new TableSort<SupportRequest, QueueKey>(
    this.list.items,
    { key: 'ageSeconds', direction: 'descending' },
    (request, key) =>
      key === 'deadline'
        ? (legalDeadline(request, new Date())?.deadline.getTime() ?? null)
        : request[key],
  );

  constructor() {
    effect(() => {
      const filters = this.filters();
      untracked(() => void this.list.reload(filters));
    });
  }

  protected age(seconds: number): string {
    return formatAge(seconds, this.transloco.getActiveLang());
  }

  protected date(iso: string): string {
    return formatDate(iso, this.transloco.getActiveLang());
  }

  protected submit(event: SubmitEvent): void {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const value = (name: string): string | null => String(form.get(name) ?? '') || null;
    void this.router.navigate([], {
      queryParams: {
        status: value('status'),
        category: value('category'),
        assignee: value('assignee'),
      },
      queryParamsHandling: 'merge',
    });
  }

  protected exportCsv(): void {
    const header = (key: string): string => this.transloco.translate(key);
    const columns: CsvColumn<SupportRequest>[] = [
      { header: header('admin.support.id'), value: (request) => request.id },
      { header: header('admin.support.category_label'), value: (request) => request.category },
      { header: header('admin.support.email'), value: (request) => request.email },
      { header: header('admin.support.status_label'), value: (request) => request.status },
      { header: header('admin.support.age'), value: (request) => request.ageSeconds },
      { header: header('admin.support.updated'), value: (request) => request.updatedAt },
      { header: header('admin.support.assignee'), value: (request) => request.assignedTo },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: 'support',
      environment: this.session.environment(),
    });
  }
}
