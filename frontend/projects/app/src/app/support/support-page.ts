import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import {
  ApiProblem,
  SupportApi,
  type SupportRequestSummary,
  type SupportRequestThread,
} from 'shared';
import { SessionStore } from '../account/session-store';
import { NewSupportRequest } from './new-support-request';
import { CATEGORY_LABELS, formatDate, STATUS_LABELS } from './support-limits';
import { SupportSignedOut } from './support-signed-out';

/**
 * Help → Contact (support-admin.md, H9): the signed-in account's requests, most recently updated
 * first, and a new one. A signed-out visitor reads why support needs an account.
 */
@Component({
  selector: 'lp-support-page',
  imports: [NewSupportRequest, RouterLink, SupportSignedOut, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './support-page.html',
  styleUrl: './support.scss',
})
export class SupportPage {
  private readonly api = inject(SupportApi);
  private readonly transloco = inject(TranslocoService);

  protected readonly account = inject(SessionStore).account;
  protected readonly categoryLabels = CATEGORY_LABELS;
  protected readonly statusLabels = STATUS_LABELS;

  protected readonly requests = signal<readonly SupportRequestSummary[]>([]);
  protected readonly nextCursor = signal<string | null | undefined>(undefined);
  protected readonly loaded = signal(false);
  protected readonly loading = signal(false);
  protected readonly loadError = signal<ApiProblem | undefined>(undefined);

  constructor() {
    effect(() => {
      if (this.account()) {
        untracked(() => void this.load());
      }
    });
  }

  protected date(iso: string): string {
    return formatDate(iso, this.transloco.getActiveLang());
  }

  /** Shows a request just sent first: it is the most recently updated. */
  protected onSent(thread: SupportRequestThread): void {
    const summary: SupportRequestSummary = {
      id: thread.id,
      category: thread.category,
      status: thread.status,
      hasScreenshot: thread.hasScreenshot,
      createdAt: thread.createdAt,
      updatedAt: thread.updatedAt,
    };
    this.requests.update((requests) => [summary, ...requests]);
  }

  /** The next page, or the first when `cursor` is `undefined`. */
  protected async load(cursor?: string): Promise<void> {
    this.loading.set(true);
    this.loadError.set(undefined);
    try {
      const page = await this.api.list(cursor);
      this.requests.update((requests) => (cursor ? [...requests, ...page.items] : page.items));
      this.nextCursor.set(page.nextCursor);
      this.loaded.set(true);
    } catch (error) {
      if (!(error instanceof ApiProblem)) {
        throw error;
      }
      this.loadError.set(error);
    } finally {
      this.loading.set(false);
    }
  }
}
