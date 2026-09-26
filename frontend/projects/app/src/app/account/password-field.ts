import {
  ChangeDetectionStrategy,
  Component,
  input,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { IonInput, type InputCustomEvent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';

/**
 * A labelled `ion-input` of `type="password"`, with a toggle that shows it in clear text: the
 * toggle's own label, translated, doubles as the accessible name (accounts.md, H7: "a
 * show-password toggle").
 */
@Component({
  selector: 'lp-password-field',
  imports: [IonInput, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-input
      #field
      [label]="label()"
      labelPlacement="stacked"
      [type]="visible() ? 'text' : 'password'"
      [autocomplete]="autocomplete()"
      [attr.maxlength]="maxChars()"
      [value]="value()"
      [errorText]="errorText()"
      [class.ion-invalid]="!!errorText()"
      [class.ion-touched]="!!errorText()"
      (ionInput)="onInput($event)"
    >
      <button type="button" slot="end" class="toggle" (click)="visible.set(!visible())">
        {{ (visible() ? 'auth.password.hide' : 'auth.password.show') | transloco }}
      </button>
    </ion-input>
  `,
  styles: `
    .toggle {
      background: none;
      border: none;
      color: var(--lp-color-accent, inherit);
      cursor: pointer;
      font: inherit;
    }
  `,
})
export class PasswordField {
  private readonly fieldRef = viewChild.required<IonInput>('field');

  readonly label = input.required<string>();
  readonly autocomplete = input<'new-password' | 'current-password'>('current-password');
  readonly maxChars = input<number>();
  readonly value = input('');
  readonly errorText = input<string | undefined>();
  readonly valueChange = output<string>();

  protected readonly visible = signal(false);

  protected onInput(event: InputCustomEvent): void {
    this.valueChange.emit(event.detail.value ?? '');
  }

  async focus(): Promise<void> {
    await this.fieldRef().setFocus();
  }
}
