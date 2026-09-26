import {
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import {
  SUPPORT_STATUSES,
  type SupportPatch,
  type SupportStatus,
  type SupportThread,
} from '../core/admin-types';
import { formatAge, formatDateTime } from '../core/format';
import { asProblem } from '../core/problem';
import { SessionStore } from '../core/session-store';
import { ViewStateStore } from '../core/view-state-store';
import { EmptyState } from '../ui/empty-state';
import { DeadlineNote } from './deadline-note';
import { SupportApi } from './support-api';
import { SUPPORT_CATEGORY_KEYS, SUPPORT_STATUS_KEYS } from './support-labels';
import { ThreadCompose } from './thread-compose';

/**
 * One request as the team handles it (admin-console.md): the thread with its internal notes, a
 * reply or a note, the status, the assignment, the screenshot, the context, and the legal
 * deadline of a data-protection request.
 */
@Component({
  selector: 'lp-support-thread-page',
  imports: [RouterLink, TranslocoPipe, DeadlineNote, EmptyState, ThreadCompose],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './support-thread-page.html',
  styleUrl: './support-thread-page.scss',
})
export class SupportThreadPage {
  private readonly api = inject(SupportApi);
  private readonly transloco = inject(TranslocoService);
  private readonly session = inject(SessionStore);

  /** Bound from the route's `:id`. */
  readonly id = input.required<string>();

  protected readonly params = inject(ViewStateStore).params;
  protected readonly adminId = computed(() => this.session.admin()?.id ?? null);
  protected readonly statuses = SUPPORT_STATUSES;
  protected readonly statusKeys = SUPPORT_STATUS_KEYS;
  protected readonly categoryKeys = SUPPORT_CATEGORY_KEYS;
  protected readonly thread = signal<SupportThread | null>(null);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly changeProblem = signal<ApiProblem | null>(null);
  protected readonly screenshot = signal<string | null>(null);
  protected readonly screenshotProblem = signal<ApiProblem | null>(null);
  protected readonly notice = signal<string | null>(null);

  constructor() {
    effect(() => {
      const id = this.id();
      untracked(() => void this.load(id));
    });
    inject(DestroyRef).onDestroy(() => this.dropScreenshot());
  }

  protected date(iso: string | null | undefined): string {
    return iso ? formatDateTime(iso, this.transloco.getActiveLang()) : '';
  }

  protected age(seconds: number): string {
    return formatAge(seconds, this.transloco.getActiveLang());
  }

  protected async setStatus(event: SubmitEvent, id: string): Promise<void> {
    event.preventDefault();
    const form = new FormData(event.target as HTMLFormElement);
    const status = String(form.get('status')) as SupportStatus;
    await this.change(id, { status }, 'admin.support.status_saved');
  }

  protected assign(id: string, adminId: string | null): Promise<void> {
    const done = adminId ? 'admin.support.assigned' : 'admin.support.unassigned';
    return this.change(id, { assignedTo: adminId }, done);
  }

  protected async showScreenshot(id: string): Promise<void> {
    this.screenshotProblem.set(null);
    try {
      const blob = await this.api.screenshot(id);
      this.dropScreenshot();
      this.screenshot.set(URL.createObjectURL(blob));
    } catch (error: unknown) {
      this.screenshotProblem.set(asProblem(error));
    }
  }

  /** A reply or a note was posted: the thread is read again, with its new status. */
  protected async posted(internal: boolean): Promise<void> {
    this.notice.set(internal ? 'admin.support.note_added' : 'admin.support.reply_sent');
    await this.load(this.id());
  }

  private async change(id: string, patch: SupportPatch, done: string): Promise<void> {
    this.changeProblem.set(null);
    this.notice.set(null);
    try {
      await this.api.update(id, patch);
      this.notice.set(done);
      await this.load(id);
    } catch (error: unknown) {
      this.changeProblem.set(asProblem(error));
    }
  }

  private async load(id: string): Promise<void> {
    try {
      this.thread.set(await this.api.thread(id));
      this.problem.set(null);
    } catch (error: unknown) {
      this.thread.set(null);
      this.problem.set(asProblem(error));
    }
  }

  private dropScreenshot(): void {
    const url = this.screenshot();
    if (url) {
      URL.revokeObjectURL(url);
      this.screenshot.set(null);
    }
  }
}
