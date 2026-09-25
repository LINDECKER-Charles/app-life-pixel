import { ChangeDetectionStrategy, Component, computed, inject, input } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { Limits } from '../engine/engine-types';
import { ExportFlow } from './export-flow';
import { clampScale, scaleBounds } from './export-options-values';

/** The export dialog's options: the tag and the scale, bounded by the limits, re-exporting. */
@Component({
  selector: 'lp-export-options-form',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './export-options-form.html',
  styleUrl: './export-options-form.scss',
})
export class ExportOptionsForm {
  protected readonly flow = inject(ExportFlow);

  readonly limits = input.required<Limits>();

  protected readonly bounds = computed(() => scaleBounds(this.limits()));
  protected readonly tags = computed(() => this.flow.document()?.tags ?? []);
  protected readonly tag = this.flow.tag;
  protected readonly scale = this.flow.scale;

  protected onTagChange(value: string): void {
    void this.flow.setTag(value === '' ? null : value);
  }

  protected onScaleChange(field: HTMLInputElement): void {
    const clamped = clampScale(field.valueAsNumber, this.bounds());
    field.value = String(clamped);
    void this.flow.setScale(clamped);
  }
}
