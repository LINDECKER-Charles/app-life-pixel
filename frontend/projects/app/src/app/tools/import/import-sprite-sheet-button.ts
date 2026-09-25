import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { ImportSpriteSheetDialog } from './import-sprite-sheet-dialog';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';

/** "Import sprite sheet": a hidden file input behind a button, the dialog, and the size error. */
@Component({
  selector: 'lp-import-sprite-sheet-button',
  imports: [ImportSpriteSheetDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" [disabled]="!engine.document()" (click)="fileInput.click()">
      {{ 'tools.import_sprite_sheet' | transloco }}
    </button>
    <input
      #fileInput
      class="lp-visually-hidden"
      type="file"
      accept="image/png"
      (change)="onChange(fileInput)"
    />
    @if (flow.error(); as error) {
      <p class="hint" role="alert">{{ 'errors.import.image_too_large' | transloco: error }}</p>
    }
    <lp-import-sprite-sheet-dialog />
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
export class ImportSpriteSheetButton {
  protected readonly engine = inject(EngineStore);
  protected readonly flow = inject(ImportSpriteSheetFlow);

  protected async onChange(input: HTMLInputElement): Promise<void> {
    const file = input.files?.[0];
    input.value = '';
    if (file) await this.flow.pickFile(file);
  }
}
