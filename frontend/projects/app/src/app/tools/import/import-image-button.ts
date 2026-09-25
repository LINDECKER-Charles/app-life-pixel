import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { ImportImage } from './import-image';

/** "Import image": a hidden file input behind a button, and the size error, if any. */
@Component({
  selector: 'lp-import-image-button',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" [disabled]="!engine.document()" (click)="fileInput.click()">
      {{ 'tools.import_image' | transloco }}
    </button>
    <input
      #fileInput
      class="lp-visually-hidden"
      type="file"
      accept="image/png"
      (change)="onChange(fileInput)"
    />
    @if (importImage.error(); as error) {
      <p class="hint" role="alert">{{ 'errors.import.image_too_large' | transloco: error }}</p>
    }
  `,
  styles: `
    button {
      padding: var(--lp-space-2) var(--lp-space-3);
      font: inherit;
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-text);
      background: transparent;
      border: 1px solid var(--lp-color-border);
      border-radius: var(--lp-radius-medium);
      cursor: pointer;

      &:disabled {
        cursor: not-allowed;
        opacity: 0.6;
      }
    }
    .hint {
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
