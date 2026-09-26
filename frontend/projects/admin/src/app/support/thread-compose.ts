import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { asProblem, problemMessage } from '../core/problem';
import { SupportApi } from './support-api';

/** A message's length, as the admin API checks it once trimmed (`support.message_length`). */
export const MESSAGE_LIMITS = { min: 1, max: 5000 } as const;

/**
 * The team's answer to a request: a reply, emailed to the person and visible in the app, or an
 * internal note they never see — chosen explicitly each time.
 */
@Component({
  selector: 'lp-thread-compose',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './thread-compose.html',
})
export class ThreadCompose {
  private readonly api = inject(SupportApi);

  readonly requestId = input.required<string>();
  /** The address a reply is emailed to. */
  readonly email = input.required<string>();
  /** Whether a reply is still possible: a closed request takes only notes. */
  readonly closed = input(false);
  /** A message was posted; `true` for an internal note. */
  readonly posted = output<boolean>();

  protected readonly limits = MESSAGE_LIMITS;
  protected readonly kind = signal<'reply' | 'note'>('reply');
  protected readonly body = signal('');
  protected readonly pending = signal(false);
  protected readonly problem = signal<ApiProblem | null>(null);
  protected readonly length = computed(() => this.body().trim().length);
  protected readonly internal = computed(() => this.closed() || this.kind() === 'note');
  protected readonly ready = computed(
    () => !this.pending() && this.length() >= this.limits.min && this.length() <= this.limits.max,
  );
  protected readonly error = computed(() => {
    const problem = this.problem();
    return problem ? problemMessage(problem) : null;
  });

  protected onKind(event: Event): void {
    this.kind.set((event.target as HTMLInputElement).value === 'note' ? 'note' : 'reply');
  }

  protected onBody(event: Event): void {
    this.body.set((event.target as HTMLTextAreaElement).value);
  }

  protected async send(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!this.ready()) {
      return;
    }
    const internal = this.internal();
    this.pending.set(true);
    this.problem.set(null);
    try {
      await this.api.post(this.requestId(), this.body().trim(), internal);
      this.body.set('');
      this.posted.emit(internal);
    } catch (error: unknown) {
      this.problem.set(asProblem(error));
    } finally {
      this.pending.set(false);
    }
  }
}
