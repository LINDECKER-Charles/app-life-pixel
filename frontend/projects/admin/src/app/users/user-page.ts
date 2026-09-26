import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import type { AdminUserDetail } from '../core/admin-types';
import { FileSaver } from '../core/file-saver';
import { formatBytes, formatDateTime } from '../core/format';
import { asProblem } from '../core/problem';
import { SessionStore } from '../core/session-store';
import { ViewStateStore } from '../core/view-state-store';
import { ConfirmAction } from '../ui/confirm-action';
import { EmptyState } from '../ui/empty-state';
import { SUPPORT_CATEGORY_KEYS, SUPPORT_STATUS_KEYS } from '../support/support-labels';
import { availableActions, confirmationOf, USER_ACTIONS, UserAction } from './user-actions';
import { USER_STATUS_KEYS } from './user-labels';
import { UsersApi } from './users-api';

/**
 * One account (admin-console.md): its profile, plan, storage, activity, sessions, support
 * requests and tokens, and the actions on it, each confirmed by naming the environment and the
 * account.
 */
@Component({
  selector: 'lp-user-page',
  imports: [RouterLink, TranslocoPipe, ConfirmAction, EmptyState],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './user-page.html',
  styleUrl: './user-page.scss',
})
export class UserPage {
  private readonly api = inject(UsersApi);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly saver = inject(FileSaver);

  /** Bound from the route's `:id`. */
  readonly id = input.required<string>();

  protected readonly environment = inject(SessionStore).environment;
  protected readonly params = inject(ViewStateStore).params;
  protected readonly actions = USER_ACTIONS;
  protected readonly statusKeys = USER_STATUS_KEYS;
  protected readonly categoryKeys = SUPPORT_CATEGORY_KEYS;
  protected readonly supportStatusKeys = SUPPORT_STATUS_KEYS;
  protected readonly user = signal<AdminUserDetail | null>(null);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly pending = signal<UserAction | null>(null);
  protected readonly running = signal(false);
  protected readonly actionProblem = signal<ApiProblem | null>(null);
  protected readonly notice = signal<string | null>(null);
  protected readonly available = computed(() => {
    const user = this.user();
    return user ? availableActions(user) : [];
  });
  protected readonly confirmation = computed(() => {
    const [action, user] = [this.pending(), this.user()];
    return action && user ? confirmationOf(action, user, this.environment()) : null;
  });

  constructor() {
    effect(() => {
      const id = this.id();
      untracked(() => void this.load(id));
    });
  }

  protected date(iso: string | null | undefined): string {
    return iso ? formatDateTime(iso, this.transloco.getActiveLang()) : '';
  }

  protected bytes(value: number): string {
    return formatBytes(value, this.transloco.getActiveLang());
  }

  protected ask(action: UserAction): void {
    this.actionProblem.set(null);
    this.notice.set(null);
    this.pending.set(action);
  }

  protected cancel(): void {
    this.pending.set(null);
  }

  protected async confirm(reason: string): Promise<void> {
    const [action, user] = [this.pending(), this.user()];
    if (!action || !user) {
      return;
    }
    this.running.set(true);
    try {
      await this.run(action, user, reason);
      this.pending.set(null);
      this.notice.set(USER_ACTIONS[action].doneKey);
    } catch (error: unknown) {
      this.actionProblem.set(asProblem(error));
    } finally {
      this.running.set(false);
    }
  }

  private async run(action: UserAction, user: AdminUserDetail, reason: string): Promise<void> {
    if (action === 'export') {
      const file = await this.api.export(user.id);
      this.saver.save(file.blob, file.fileName ?? `life-pixel-export-${user.id}.zip`);
      return;
    }
    if (action === 'delete') {
      await this.api.delete(user.id, reason);
      await this.router.navigate(['/users'], { queryParams: this.params() });
      return;
    }
    await (action === 'suspend'
      ? this.api.suspend(user.id, reason)
      : this.api.reactivate(user.id, reason));
    await this.load(user.id);
  }

  private async load(id: string): Promise<void> {
    try {
      this.user.set(await this.api.get(id));
      this.problem.set(null);
    } catch (error: unknown) {
      this.user.set(null);
      this.problem.set(asProblem(error));
    }
  }
}
