import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { Icon } from '../../ui/icon/icon';
import { Tooltip } from '../../ui/tooltip/tooltip';
import { ImportImage } from './import-image';

/** "Import image": a hidden file input behind a button, and the size error, if any. */
@Component({
  selector: 'lp-import-image-button',
  imports: [Icon, Tooltip, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button
      type="button"
      class="lp-icon-button"
      [attr.aria-label]="'tools.import_image' | transloco"
      [disabled]="!engine.document()"
      [lpTooltip]="'tools.import_image' | transloco"
      (click)="fileInput.click()"
    >
      <lp-icon name="import-image" size="large" />
    </button>
    <input
      #fileInput
      class="lp-visually-hidden"
      type="file"
      accept="image/png"
      tabindex="-1"
      aria-hidden="true"
      [disabled]="!engine.document()"
      [attr.aria-label]="'tools.import_image' | transloco"
      (change)="onChange(fileInput)"
    />
    @if (importImage.error(); as error) {
      <p class="hint" role="alert">{{ 'errors.import.image_too_large' | transloco: error }}</p>
    }
  `,
  styles: `
    /* The button is one cell of the tool rail's grid; an error spans the whole row under it. */
    :host {
      display: contents;
    }
    .hint {
      grid-column: 1 / -1;
      margin: var(--lp-space-1) 0 0;
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-danger);
    }
  `,
})
export class ImportImageButton {
  protected readonly engine = inject(EngineStore);
  protected readonly importImage = inject(ImportImage);

  protected async onChange(input: HTMLInputElement): Promise<void> {
    const file = input.files?.[0];
    input.value = '';
    if (file) await this.importImage.importFile(file);
  }
}
