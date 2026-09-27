import { ChangeDetectionStrategy, Component, computed, input, output, signal } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { Icon } from '../../ui/icon/icon';
import type { FormError } from './form-errors';

/**
 * A labelled password `<input>` (`.lp-field`), with a button that shows it in clear text; its hint
 * and its error are tied to it with `aria-describedby` (accounts.md, H7: "a show-password
 * toggle"). Paste and password managers work as on any field.
 */
@Component({
  selector: 'lp-password-field',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'lp-field' },
  template: `
    <label class="lp-field__label" [for]="fieldId()">{{ label() }}</label>
    @if (hint(); as hint) {
      <p class="lp-field__hint" [id]="id('hint')">{{ hint }}</p>
    }
    <div class="lp-field__control">
      <input
        #field
        class="lp-input"
        [id]="fieldId()"
        [type]="visible() ? 'text' : 'password'"
        [autocomplete]="autocomplete()"
        [attr.maxlength]="maxChars()"
        [value]="value()"
        [attr.aria-invalid]="error() ? 'true' : null"
        [attr.aria-describedby]="describedBy()"
        (input)="valueChange.emit(field.value)"
      />
      <button
        type="button"
        class="lp-button lp-button--secondary"
        [attr.aria-controls]="fieldId()"
        (click)="visible.set(!visible())"
      >
        {{ (visible() ? 'auth.password.hide' : 'auth.password.show') | transloco }}
      </button>
    </div>
    @if (error(); as error) {
      <p class="lp-field__error" [id]="id('error')">
        <lp-icon name="error" size="small" />{{ error.key | transloco: error.params }}
      </p>
    }
  `,
  // On a narrow screen the toggle goes under the field rather than squeezing it.
  styles: `
    .lp-field__control {
      flex-wrap: wrap;
    }
    .lp-input {
      flex-basis: 12rem;
    }
  `,
})
export class PasswordField {
  /** The `<input>`'s id, from the parent's id scope: its error summary links the field by it. */
  readonly fieldId = input.required<string>();
  readonly label = input.required<string>();
  readonly hint = input<string>();
  readonly autocomplete = input<'new-password' | 'current-password'>('current-password');
  readonly maxChars = input<number>();
  readonly value = input('');
  readonly error = input<FormError>();
  readonly valueChange = output<string>();

  protected readonly id = idScope('password-field');
  protected readonly visible = signal(false);

  protected readonly describedBy = computed(() => {
    const ids = [
      this.hint() ? this.id('hint') : undefined,
      this.error() ? this.id('error') : undefined,
    ].filter((id) => id !== undefined);
    return ids.length > 0 ? ids.join(' ') : null;
  });
}
