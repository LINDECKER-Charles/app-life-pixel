import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  input,
  linkedSignal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { DEFAULT_COLOR, isValidHex8, joinHex8, splitHex8 } from './palette-color';
import { PaletteEntryFlow, type PaletteEntryMode } from './palette-entry-flow';

/**
 * The add/edit form: a colour input, an alpha slider (0 to 255) and a `#rrggbbaa` field, the three
 * always in sync through one source of truth, the hex value (editor.md, U4).
 */
@Component({
  selector: 'lp-palette-entry-form',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './palette-entry-form.html',
  styleUrl: './palette-entry-form.scss',
})
export class PaletteEntryForm {
  private readonly flow = inject(PaletteEntryFlow);

  readonly mode = input.required<PaletteEntryMode>();

  protected readonly hex = linkedSignal<PaletteEntryMode, string>({
    source: this.mode,
    computation: (mode) => (mode.kind === 'edit' ? mode.color : DEFAULT_COLOR),
  });
  protected readonly rgb = computed(() => splitHex8(this.hex()).rgb);
  protected readonly alpha = computed(() => splitHex8(this.hex()).alpha);
  protected readonly isValid = computed(() => isValidHex8(this.hex()));
  protected readonly submitLabel = computed(() =>
    this.mode().kind === 'edit' ? 'palette.dialog.edit_submit' : 'palette.dialog.add_submit',
  );

  protected setRgb(rgb: string): void {
    this.hex.set(joinHex8(rgb, this.alpha()));
  }

  protected setAlpha(alpha: number): void {
    this.hex.set(joinHex8(this.rgb(), alpha));
  }

  protected setHex(value: string): void {
    this.hex.set(value.toLowerCase());
  }

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!this.isValid()) return;
    await this.flow.submit(this.hex());
  }

  protected cancel(): void {
    this.flow.close();
  }
}
