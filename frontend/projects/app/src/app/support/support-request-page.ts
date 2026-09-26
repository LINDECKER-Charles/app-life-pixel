import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { ApiProblem, SupportApi, type SupportRequestThread } from 'shared';
import { ErrorSummary } from '../account/form/error-summary';
import { fieldError } from '../account/form/field-error';
import { type FieldCheck, FormErrors } from '../account/form/form-errors';
import { SessionStore } from '../account/session-store';
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
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
 * while it is open, checked when sent. The team's internal notes never reach the app.
 */
@Component({
  selector: 'lp-support-request-page',
  imports: [
    ErrorSummary,
    Icon,
    IonContent,
    RouterLink,
    StatusBanner,
    SupportSignedOut,
    TranslocoPipe,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './support-request-page.html',
  styleUrl: './support.scss',
})
export class SupportRequestPage {
  private readonly api = inject(SupportApi);
  private readonly transloco = inject(TranslocoService);
  private readonly summary = viewChild(ErrorSummary);

  /** Bound from the route's `:requestId`. */
  readonly requestId = input.required<string>();

  protected readonly account = inject(SessionStore).account;
  protected readonly categoryLabels = CATEGORY_LABELS;
  protected readonly statusLabels = STATUS_LABELS;
  protected readonly maxChars = SUPPORT_LIMITS.messageMaxChars;
  protected readonly targets = { body: 'support-reply' };

  protected readonly thread = signal<SupportRequestThread | undefined>(undefined);
  protected readonly loadError = signal<ApiProblem | undefined>(undefined);
  protected readonly reply = signal('');
  protected readonly pending = signal(false);
  protected readonly errors = new FormErrors<ReplyField>(
    { 'support.message_length': 'body' },
    'form',
  );
  protected readonly length = computed(() => messageLength(this.reply()));

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

  protected onReply(event: Event): void {
    this.reply.set((event.target as HTMLTextAreaElement).value);
    this.errors.recheck('body', this.check()[1]);
  }

  protected async send(event: SubmitEvent, thread: SupportRequestThread): Promise<void> {
    event.preventDefault();
    if (this.pending()) {
      return;
    }
    if (!this.errors.check([this.check()])) {
      this.summary()?.focusFirstError();
      return;
    }
    this.pending.set(true);
    try {
      await this.api.reply(thread.id, this.reply());
      this.reply.set('');
      await this.load(thread.id);
    } catch (error) {
      this.errors.set(error);
      this.summary()?.focusFirstError();
    } finally {
      this.pending.set(false);
    }
  }

  private check(): FieldCheck<ReplyField> {
    const length = this.length();
    const max = this.maxChars;
    return [
      'body',
      fieldError(length > 0 && length <= max, 'errors.support.message_length', { min: 1, max }),
    ];
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
