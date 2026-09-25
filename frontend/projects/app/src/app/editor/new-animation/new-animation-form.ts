import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  input,
  linkedSignal,
  signal,
} from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import type { Limits } from '../../engine/engine-types';
import { NewAnimationFlow } from './new-animation-flow';
import { defaultSide, isValidSide, isValidTitle, sideBounds } from './new-animation-values';

/** The new-animation dialog's form: a title and a size, 32 × 32 by default, within the limits. */
@Component({
  selector: 'lp-new-animation-form',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './new-animation-form.html',
  styleUrl: './new-animation-form.scss',
})
export class NewAnimationForm {
  private readonly flow = inject(NewAnimationFlow);

  readonly limits = input.required<Limits>();

  protected readonly bounds = computed(() => sideBounds(this.limits()));
  protected readonly title = signal(inject(TranslocoService).translate('editor.new.default_title'));
  protected readonly width = linkedSignal(() => defaultSide(this.bounds()));
  protected readonly height = linkedSignal(() => defaultSide(this.bounds()));
  protected readonly isTitleValid = computed(() => isValidTitle(this.title(), this.limits()));
  protected readonly isWidthValid = computed(() => isValidSide(this.width(), this.bounds()));
  protected readonly isHeightValid = computed(() => isValidSide(this.height(), this.bounds()));
  protected readonly isValid = computed(
    () => this.isTitleValid() && this.isWidthValid() && this.isHeightValid(),
  );

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!this.isValid()) return;
    await this.flow.create({
      title: this.title().trim(),
      width: this.width(),
      height: this.height(),
    });
  }

  protected cancel(): void {
    this.flow.close();
  }
}
