import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  untracked,
} from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { AuditEntry } from '../core/admin-types';
import type { CsvColumn } from '../core/csv';
import { FileSaver } from '../core/file-saver';
import { formatDateTime } from '../core/format';
import { PagedList } from '../core/paged-list';
import { SessionStore } from '../core/session-store';
import { TableSort } from '../core/table-sort';
import { EmptyState } from '../ui/empty-state';
import { SortButton } from '../ui/sort-button';
import { AUDIT_ACTIONS, AuditApi, AuditSearch } from './audit-api';

type AuditKey = 'at' | 'adminEmail' | 'action' | 'targetType' | 'targetId' | 'reason';

/** Every admin action — who, what, when, why, before and after — searchable (admin-console.md). */
@Component({
  selector: 'lp-audit-page',
  imports: [TranslocoPipe, EmptyState, SortButton],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './audit-page.html',
})
export class AuditPage {
  private readonly api = inject(AuditApi);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);
  private readonly environment = inject(SessionStore).environment;

  /** Bound from the query: `?adminId=…&action=user.suspend&targetId=…`. */
  readonly adminId = input<string>();
  readonly action = input<string>();
  readonly targetId = input<string>();

  protected readonly actions = AUDIT_ACTIONS;
  protected readonly columns: readonly { key: AuditKey; labelKey: string }[] = [
    { key: 'at', labelKey: 'admin.audit.at' },
    { key: 'adminEmail', labelKey: 'admin.audit.admin' },
    { key: 'action', labelKey: 'admin.audit.action' },
    { key: 'targetType', labelKey: 'admin.audit.target_type' },
    { key: 'targetId', labelKey: 'admin.audit.target' },
    { key: 'reason', labelKey: 'admin.audit.reason' },
  ];
  protected readonly search = computed<AuditSearch>(() => ({
    adminId: this.adminId()?.trim() || undefined,
    action: this.action() || undefined,
    targetId: this.targetId()?.trim() || undefined,
  }));
  protected readonly list = new PagedList<AuditEntry, AuditSearch>((search, cursor) =>
    this.api.search(search, cursor),
  );
  protected readonly table = new TableSort<AuditEntry, AuditKey>(
    this.list.items,
    { key: 'at', direction: 'descending' },
    (entry, key) => entry[key],
  );

  constructor() {
    effect(() => {
      const search = this.search();
      untracked(() => void this.list.reload(search));
    });
  }

  protected date(iso: string): string {
    return formatDateTime(iso, this.transloco.getActiveLang());
  }

  protected json(value: unknown): string {
    return value === null || value === undefined ? '' : JSON.stringify(value, null, 2);
  }

  protected submit(event: SubmitEvent): void {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const value = (name: string): string | null => String(form.get(name) ?? '').trim() || null;
    void this.router.navigate([], {
      queryParams: {
        adminId: value('adminId'),
        action: value('action'),
        targetId: value('targetId'),
      },
      queryParamsHandling: 'merge',
    });
  }

  protected exportCsv(): void {
    const header = (key: string): string => this.transloco.translate(key);
    const columns: CsvColumn<AuditEntry>[] = [
      { header: header('admin.audit.id'), value: (entry) => entry.id },
      ...this.columns.map((column) => ({
        header: header(column.labelKey),
        value: (entry: AuditEntry) => entry[column.key],
      })),
      { header: header('admin.audit.before'), value: (entry) => this.json(entry.before) },
      { header: header('admin.audit.after'), value: (entry) => this.json(entry.after) },
    ];
    this.saver.saveCsv(this.table.rows(), columns, {
      table: 'audit-log',
      environment: this.environment(),
    });
  }
}
