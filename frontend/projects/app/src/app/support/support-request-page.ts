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
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { ApiProblem, SupportApi, type SupportRequestThread } from 'shared';
import { FormErrors } from '../account/form-errors';
import { SessionStore } from '../account/session-store';
import {
  CATEGORY_LABELS,
  formatDate,
  messageLength,
  STATUS_LABELS,
  SUPPORT_LIMITS,
} from './support-limits';
import { SupportSignedOut } from './support-signed-out';

type ReplyField = 'body' | 'form';

/**
 * One request's thread (support-admin.md, H9): its messages, oldest first, and a reply field
 * while it is open. The team's internal notes never reach the app.
 */
@Component({
  selector: 'lp-support-request-page',
  imports: [IonContent, RouterLink, SupportSignedOut, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './support-request-page.html',
  styleUrl: './support.scss',
})
export class SupportRequestPage {
  private readonly api = inject(SupportApi);
  private readonly transloco = inject(TranslocoService);

  /** Bound from the route's `:requestId`. */
  readonly requestId = input.required<string>();

  protected readonly account = inject(SessionStore).account;
  protected readonly categoryLabels = CATEGORY_LABELS;
  protected readonly statusLabels = STATUS_LABELS;
  protected readonly maxChars = SUPPORT_LIMITS.messageMaxChars;

  protected readonly thread = signal<SupportRequestThread | undefined>(undefined);
  protected readonly loadError = signal<ApiProblem | undefined>(undefined);
  protected readonly reply = signal('');
  protected readonly pending = signal(false);
  protected readonly errors = new FormErrors<ReplyField>(
    { 'support.message_length': 'body' },
    'form',
  );
  protected readonly length = computed(() => messageLength(this.reply()));
  protected readonly ready = computed(
    () => this.length() > 0 && this.length() <= this.maxChars && !this.pending(),
  );

  constructor() {
    effect(() => {
      const requestId = this.requestId();
      if (this.account()) {
        untracked(() => void this.load(requestId));
      }
    });
  }

  protected date(iso: string): string {
    return formatDate(iso, this.transloco.getActiveLang());
  }

  protected errorText(field: ReplyField): string | undefined {
    const error = this.errors.of(field);
    return error ? this.transloco.translate(`errors.${error.code}`, error.params) : undefined;
  }

  protected onReply(event: Event): void {
    this.reply.set((event.target as HTMLTextAreaElement).value);
  }

  protected async send(event: SubmitEvent, thread: SupportRequestThread): Promise<void> {
    event.preventDefault();
    if (!this.ready()) {
      return;
    }
    this.errors.clear();
    this.pending.set(true);
    try {
      await this.api.reply(thread.id, this.reply());
      this.reply.set('');
      await this.load(thread.id);
    } catch (error) {
      this.errors.set(error);
    } finally {
      this.pending.set(false);
    }
  }

  private async load(requestId: string): Promise<void> {
    this.loadError.set(undefined);
    try {
      this.thread.set(await this.api.get(requestId));
    } catch (error) {
      if (!(error instanceof ApiProblem)) {
        throw error;
      }
      this.thread.set(undefined);
      this.loadError.set(error);
    }
  }
}
