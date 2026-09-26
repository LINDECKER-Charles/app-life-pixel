import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  Injector,
  input,
  viewChild,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { StatusBanner } from '../../ui/status-banner/status-banner';
import type { FormErrorEntry } from './form-errors';

/**
 * The errors of a refused form, above its fields (design-system/docs/accessibility.md, "Forms,
 * validation and authentication"): an alert, announced once, that links each message to its
 * field — `targets` gives the id of each field's control. A message of the form as a whole, such
 * as `auth.csrf`, has no link. Place it inside the form: it looks for the fields there.
 */
@Component({
  selector: 'lp-error-summary',
  imports: [StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (entries().length > 0) {
      <lp-status-banner #summary variant="danger" tabindex="-1">
        <p>{{ 'auth.form.summary' | transloco }}</p>
        <ul class="list">
          @for (entry of entries(); track entry.field) {
            <li>
              @if (targets()[entry.field]; as target) {
                <a [href]="'#' + target" (click)="focusField($event, target)">
                  {{ entry.error.key | transloco: entry.error.params }}
                </a>
              } @else {
                {{ entry.error.key | transloco: entry.error.params }}
              }
            </li>
          }
        </ul>
      </lp-status-banner>
    }
  `,
  styles: `
    :host {
      display: block;
    }
    /* Without errors it takes no gap in the form's stack. */
    :host(:empty) {
      display: none;
    }
    .list {
      padding-inline-start: var(--lp-space-4);
    }
    a {
      color: inherit;
    }
  `,
})
export class ErrorSummary {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly injector = inject(Injector);
  private readonly summary = viewChild('summary', { read: ElementRef<HTMLElement> });

  /** The errors to list, in the order of the fields. */
  readonly entries = input.required<readonly FormErrorEntry[]>();
  /** The id of each field's control, by field. */
  readonly targets = input<Readonly<Record<string, string>>>({});

  /**
   * Once the errors are drawn, moves the focus to the first invalid field, or to the summary
   * when only the form as a whole was refused.
   */
  focusFirstError(): void {
    afterNextRender(() => this.focusNow(), { injector: this.injector });
  }

  protected focusField(event: Event, id: string): void {
    event.preventDefault();
    this.control(id)?.focus();
  }

  private focusNow(): void {
    const first = this.entries().find((entry) => this.targets()[entry.field]);
    const control = first ? this.control(this.targets()[first.field]) : null;
    (control ?? this.summary()?.nativeElement)?.focus();
  }

  /** The control `id` of this form: another page kept by the outlet may reuse the id. */
  private control(id: string): HTMLElement | null {
    const host = this.host.nativeElement;
    const scope = host.closest('form') ?? host.parentElement ?? host;
    return scope.querySelector<HTMLElement>(`[id="${id}"]`);
  }
}
