import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  input,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { ApiProblem } from 'shared';
import { problemMessage } from '../core/problem';
import { EnvironmentName } from './environment-name';

/** The limits of a reason, as the admin API checks them once trimmed (`admin.reason_length`). */
export const REASON_LIMITS = { min: 1, max: 1000 } as const;

/** What a confirmation asks about: the action's keys, where, on what, and whether it needs a reason. */
export interface ConfirmRequest {
  /** The action's title, such as "Suspend this account". */
  readonly titleKey: string;
  /** What it does, with `{ target, environment }`. */
  readonly messageKey: string;
  /** The confirm button's label, with `{ environment }`. */
  readonly confirmKey: string;
  readonly environment: string;
  /** What the action changes, as the admin reads it: an address, a request's id. */
  readonly target: string;
  readonly needsReason: boolean;
}

/**
 * The confirmation of a dangerous action (admin-console.md): a modal dialog naming the
 * environment and the target, asking for the reason the audit log keeps.
 */
@Component({
  selector: 'lp-confirm-action',
  imports: [TranslocoPipe, EnvironmentName],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './confirm-action.html',
  styleUrl: './confirm-action.scss',
})
export class ConfirmAction {
  readonly request = input.required<ConfirmRequest>();
  readonly pending = input(false);
  readonly problem = input<ApiProblem | null>(null);
  /** The admin confirmed, with the reason, trimmed; empty when none is asked. */
  readonly confirmed = output<string>();
  readonly cancelled = output<void>();

  private readonly dialog = viewChild.required<ElementRef<HTMLDialogElement>>('dialog');

  protected readonly limits = REASON_LIMITS;
  protected readonly reason = signal('');
  protected readonly reasonLength = computed(() => this.reason().trim().length);
  protected readonly ready = computed(() => {
    if (this.pending()) {
      return false;
    }
    const length = this.reasonLength();
    return !this.request().needsReason || (length >= this.limits.min && length <= this.limits.max);
  });
  protected readonly error = computed(() => {
    const problem = this.problem();
    return problem ? problemMessage(problem) : null;
  });

  constructor() {
    afterNextRender(() => {
      const dialog = this.dialog().nativeElement;
      if (typeof dialog.showModal === 'function') {
        dialog.showModal();
      } else {
        dialog.setAttribute('open', '');
      }
    });
  }

  protected onReason(event: Event): void {
    this.reason.set((event.target as HTMLTextAreaElement).value);
  }

  protected submit(event: SubmitEvent): void {
    event.preventDefault();
    if (this.ready()) {
      this.confirmed.emit(this.reason().trim());
    }
  }

  /** Escape, or the cancel button: the dialog closes and nothing happens. */
  protected cancel(event?: Event): void {
    event?.preventDefault();
    this.cancelled.emit();
  }
}
