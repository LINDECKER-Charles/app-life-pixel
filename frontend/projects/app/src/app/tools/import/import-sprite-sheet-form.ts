import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  input,
  linkedSignal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { Limits } from '../../engine/engine-types';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';
import { cellSideBounds, durationBounds, isValidSide } from './import-sprite-sheet-values';

/** The sprite-sheet dialog's form: the cell size and the frames' duration, within the limits. */
@Component({
  selector: 'lp-import-sprite-sheet-form',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './import-sprite-sheet-form.html',
  styleUrl: './import-sprite-sheet-form.scss',
})
export class ImportSpriteSheetForm {
  private readonly flow = inject(ImportSpriteSheetFlow);

  readonly limits = input.required<Limits>();

  protected readonly cellBounds = computed(() => cellSideBounds(this.limits()));
  protected readonly durationRange = computed(() => durationBounds(this.limits()));
  protected readonly cellWidth = linkedSignal(() => this.cellBounds().min);
  protected readonly cellHeight = linkedSignal(() => this.cellBounds().min);
  protected readonly durationMs = linkedSignal(() => this.limits().defaultFrameDurationMs);
  protected readonly isCellWidthValid = computed(() =>
    isValidSide(this.cellWidth(), this.cellBounds()),
  );
  protected readonly isCellHeightValid = computed(() =>
    isValidSide(this.cellHeight(), this.cellBounds()),
  );
  protected readonly isDurationValid = computed(() =>
    isValidSide(this.durationMs(), this.durationRange()),
  );
  protected readonly isValid = computed(
    () => this.isCellWidthValid() && this.isCellHeightValid() && this.isDurationValid(),
  );

  protected async submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!this.isValid()) return;
    await this.flow.submit({
      cellWidth: this.cellWidth(),
      cellHeight: this.cellHeight(),
      durationMs: this.durationMs(),
    });
  }

  protected cancel(): void {
    this.flow.close();
  }
}
